use crate::prelude::{Duration, Epoch, IonosphericData, TimeScale, TrackData};
use polyfit_rs::polyfit_rs::polyfit;
use std::collections::BTreeMap;
use thiserror::Error;

fn linear_reg_2d(i: (f64, f64), j: (f64, f64)) -> (f64, f64) {
    let (_, y_i) = i;
    let (x_j, y_j) = j;
    let a = y_j - y_i;
    let b = y_j - a * x_j;
    (a, b)
}

/// CGGTTS track formation errors
#[derive(Debug, Clone, Error)]
pub enum FitError {
    /// Linear regression failure. Either extreme values
    /// encountered or data gaps are present.
    #[error("linear regression failure")]
    LinearRegressionFailure,
    /// You can't fit a track if buffer contains gaps.
    /// CGGTTS requires steady sampling. On this error, you should
    /// reset the tracker.
    #[error("buffer contains gaps")]
    NonContiguousBuffer,
    /// Can only fit "complete" tracks
    #[error("missing measurements (data gaps)")]
    IncompleteTrackMissingMeasurements,
    /// Buffer should be centered on tracking midpoint
    #[error("not centered on midpoint")]
    NotCenteredOnTrackMidpoint,
}

/// SV Tracker is used to track a single SV and form a CGGTTS track.
#[derive(Default, Debug, Clone)]
pub struct SVTracker {
    /* internal buffer */
    buffer: BTreeMap<Epoch, FitData>,
}

/// FitData is a measurement to pass several times
/// to the SVTracker and try to form a track.
#[derive(Debug, Default, Clone)]
pub struct FitData {
    /// REFSV [s]
    pub refsv: f64,
    /// REFSYS [s]
    pub refsys: f64,
    /// MDTR Modeled Tropospheric Delay [s]
    pub mdtr: f64,
    /// SV elevation [°]
    pub elevation: f64,
    /// SV azimuth [°]
    pub azimuth: f64,
    /// MDIO Modeled Ionospheric Delay [s]
    pub mdio: Option<f64>,
    /// MSIO Measured Ionospheric Delay [s]
    pub msio: Option<f64>,
}

impl SVTracker {
    /* has msio data */
    fn has_msio(&self) -> bool {
        self.buffer
            .values()
            .filter(|data| data.msio.is_some())
            .count()
            > 0
    }
    /// Try to fit a track. You need to provide the ongoing IOE.
    pub fn fit(
        &self,
        ioe: u16,
        trk_duration: Duration,
        sampling_period: Duration,
        trk_midpoint: Epoch,
    ) -> Result<((f64, f64), TrackData, Option<IonosphericData>), FitError> {
        // verify tracking completion
        //  complete if we have enough measurements
        let expected_nb =
            (trk_duration.to_seconds() / sampling_period.to_seconds()).ceil() as usize;
        if self.buffer.len() < expected_nb {
            return Err(FitError::IncompleteTrackMissingMeasurements);
        }

        // verify tracking completion
        // complete if we're centered on midpoint
        let (first, _) = self.buffer.first_key_value().unwrap();

        let (last, _) = self.buffer.last_key_value().unwrap();

        if !((*first < trk_midpoint) && (*last > trk_midpoint)) {
            return Err(FitError::NotCenteredOnTrackMidpoint);
        }

        let t_xs: Vec<_> = self
            .buffer
            .keys()
            .map(|t| {
                t.to_duration_in_time_scale(t.time_scale)
                    .total_nanoseconds() as f64
                    * 1.0E-9
            })
            .collect();

        let t_mid_s = trk_midpoint
            .to_duration_in_time_scale(first.time_scale)
            .total_nanoseconds() as f64
            * 1.0E-9;

        let mut t_mid_index = 0;
        for (index, t) in self.buffer.keys().enumerate() {
            if *t < trk_midpoint {
                t_mid_index = index;
            }
        }

        /*
         * for the SV attitude at mid track:
         * we either use the direct measurement if one was latched @ that UTC epoch
         * or we fit ax+b fit between adjacent attitudes.
         */
        let elev = self.buffer.iter().find(|(t, _)| **t == trk_midpoint);

        let (elev, azi) = match elev {
            Some(elev) => {
                // UTC epoch exists
                let azi = self
                    .buffer
                    .iter()
                    .find(|(t, _)| **t == trk_midpoint)
                    .unwrap() // unfaillible @ this point
                    .1
                    .azimuth;
                (elev.1.elevation, azi)
            },
            None => {
                /* linear interpolation */
                let elev: Vec<_> = self.buffer.iter().map(|(_, fit)| fit.elevation).collect();
                let (a, b) = linear_reg_2d(
                    (t_xs[t_mid_index], elev[t_mid_index]),
                    (t_xs[t_mid_index + 1], elev[t_mid_index + 1]),
                );

                let elev = a * t_mid_s + b;

                let azi: Vec<_> = self.buffer.iter().map(|(_, fit)| fit.azimuth).collect();
                let (a, b) = linear_reg_2d(
                    (t_xs[t_mid_index], azi[t_mid_index]),
                    (t_xs[t_mid_index + 1], azi[t_mid_index + 1]),
                );

                let azi = a * t_mid_s + b;
                (elev, azi)
            },
        };

        let fit = polyfit(
            &t_xs,
            &self
                .buffer
                .values()
                .map(|f| f.refsv)
                .collect::<Vec<_>>()
                .as_slice(),
            1,
        )
        .map_err(|_| FitError::LinearRegressionFailure)?;

        let (srsv, srsv_b) = (fit[1], fit[0]);
        let refsv = srsv * t_mid_s + srsv_b;

        let fit = polyfit(
            &t_xs,
            &self
                .buffer
                .values()
                .map(|f| f.refsys)
                .collect::<Vec<_>>()
                .as_slice(),
            1,
        )
        .map_err(|_| FitError::LinearRegressionFailure)?;

        let (srsys, srsys_b) = (fit[1], fit[0]);
        let refsys = srsys * t_mid_s + srsys_b;

        let refsys_fit: Vec<_> = t_xs.iter().map(|t_s| srsys * t_s + srsys_b).collect();

        let mut dsg = 0.0_f64;
        for refsys_fit in refsys_fit {
            dsg += (refsys_fit - refsys).powi(2);
        }
        dsg = dsg.sqrt();

        let fit = polyfit(
            &t_xs,
            &self
                .buffer
                .values()
                .map(|f| f.mdtr)
                .collect::<Vec<_>>()
                .as_slice(),
            1,
        )
        .map_err(|_| FitError::LinearRegressionFailure)?;

        let (smdt, smdt_b) = (fit[1], fit[0]);
        let mdtr = smdt * t_mid_s + smdt_b;

        let fit = polyfit(
            &t_xs,
            &self
                .buffer
                .values()
                .map(|f| f.mdio.unwrap_or(0.0_f64))
                .collect::<Vec<_>>()
                .as_slice(),
            1,
        )
        .map_err(|_| FitError::LinearRegressionFailure)?;

        let (smdi, smdi_b) = (fit[1], fit[0]);
        let mdio = smdi * t_mid_s + smdi_b;

        let trk_data = TrackData {
            refsv,
            srsv,
            refsys,
            srsys,
            dsg,
            ioe,
            mdtr,
            smdt,
            mdio,
            smdi,
        };

        let iono_data = match self.has_msio() {
            false => None,
            true => {
                let fit = polyfit(
                    &t_xs,
                    &self
                        .buffer
                        .values()
                        .map(|f| f.msio.unwrap())
                        .collect::<Vec<_>>()
                        .as_slice(),
                    1,
                )
                .map_err(|_| FitError::LinearRegressionFailure)?;

                let (smsi, smsi_b) = (fit[1], fit[0]);
                let msio = smsi * t_mid_s + smsi_b;

                let mut isg = 0.0_f64;
                let msio_fit: Vec<_> = t_xs.iter().map(|t_s| smsi * t_s + smsi_b).collect();
                for msio_fit in msio_fit {
                    isg += (msio_fit - msio).powi(2);
                }
                isg = isg.sqrt();

                Some(IonosphericData { msio, smsi, isg })
            },
        };

        Ok(((elev, azi), trk_data, iono_data))
    }

    /// Latch a new measurement at given UTC Epoch.
    /// You can then use .fit() to try to fit a track.
    pub fn latch_measurement(&mut self, utc_t: Epoch, data: FitData) {
        if let Some((last_t, _)) = self.buffer.last_key_value() {
            assert!(
                utc_t > *last_t,
                "samples should be streamed in chronological order"
            );
        }
        self.buffer.insert(utc_t, data);
    }

    /// You should only form a track (.fit()) if no_gaps are present in the buffer.
    pub fn no_gaps(&self, sampling_period: Duration) -> bool {
        let mut prev = Option::<Epoch>::None;
        for t in self.buffer.keys() {
            if let Some(prev) = prev {
                let dt = *t - prev;
                if dt > sampling_period {
                    return false;
                }
            }
            prev = Some(*t);
        }
        true
    }

    /// Reset and flush previously latched measurements
    pub fn reset(&mut self) {
        self.buffer.clear();
    }

    /// True if at least one measurement has been latched
    pub fn not_empty(&self) -> bool {
        !self.buffer.is_empty()
    }
}

/// Scheduler used to form synchronous CGGTTS tracks.
#[derive(Debug, Clone)]
pub struct Scheduler {
    /// Tracking duration in use.
    pub trk_duration: Duration,
}

impl Default for Scheduler {
    fn default() -> Self {
        Self {
            trk_duration: Self::bipm_tracking_duration(),
        }
    }
}

impl Scheduler {
    /// Standard tracking duration [s]
    pub const BIPM_TRACKING_DURATION_SECONDS: u32 = 960;

    /// Returns standard tracking duration
    pub fn bipm_tracking_duration() -> Duration {
        Duration::from_seconds(Self::BIPM_TRACKING_DURATION_SECONDS as f64)
    }

    /// Generates a new Track Scheduler from a given (usually simply "now")
    /// datetime expressed as an Epoch.
    pub fn new(trk_duration: Duration) -> Self {
        Self { trk_duration }
    }

    /* track 0 offset within any MJD, expressed in nanos */
    pub(crate) fn t0_offset_nanos(mjd: u32, trk_duration: Duration) -> i128 {
        let tracking_nanos = trk_duration.total_nanoseconds();
        let offset_nanos = (
            (50_722 - mjd as i128)
            * 4 * 1_000_000_000 * 60  // shift per day
            + 2 * 1_000_000_000 * 60
            // offset on MJD=50722 reference
        ) % trk_duration.total_nanoseconds();
        if offset_nanos < 0 {
            offset_nanos + tracking_nanos
        } else {
            offset_nanos
        }
    }

    /// Next track start time, compared to given Epoch.
    pub fn next_track_start(&self, t: Epoch) -> Epoch {
        let utc_t = match t.time_scale {
            TimeScale::UTC => t,
            _ => Epoch::from_utc_duration(t.to_utc_duration()),
        };

        let trk_duration = self.trk_duration;
        let mjd = utc_t.to_mjd_utc_days();
        let mjd_u = mjd.floor() as u32;

        let mjd_next = Epoch::from_mjd_utc((mjd_u + 1) as f64);
        let time_to_midnight = mjd_next - utc_t;

        match time_to_midnight < trk_duration {
            true => {
                /*
                 * if we're in the last track of the day,
                 * we need to consider next day (MJD+1)
                 */
                let offset_nanos = Self::t0_offset_nanos(mjd_u + 1, trk_duration);
                Epoch::from_mjd_utc((mjd_u + 1) as f64)
                    + Duration::from_nanoseconds(offset_nanos as f64)
            },
            false => {
                let offset_nanos = Self::t0_offset_nanos(mjd_u, trk_duration);

                // determine track number this "t" contributes to
                let day_offset_nanos =
                    (utc_t - Epoch::from_mjd_utc(mjd_u as f64)).total_nanoseconds() - offset_nanos;
                let i = (day_offset_nanos as f64 / trk_duration.total_nanoseconds() as f64).ceil();

                let mut e = Epoch::from_mjd_utc(mjd_u as f64)
                    + Duration::from_nanoseconds(offset_nanos as f64);

                // on first track of day: we only have the day nanos offset
                if i > 0.0 {
                    // add ith track offset
                    e += Duration::from_nanoseconds(i * trk_duration.total_nanoseconds() as f64);
                }
                e
            },
        }
    }
    /// Helper to determine how long until a next "synchronous" track.
    pub fn time_to_next_track(&self, now: Epoch) -> Duration {
        self.next_track_start(now) - now
    }
}

#[cfg(test)]
mod test {
    use crate::track::Scheduler;
    use crate::{Duration, Epoch};
    #[test]
    fn t0_offset_minutes() {
        let duration = Duration::from_seconds(Scheduler::BIPM_TRACKING_DURATION_SECONDS as f64);
        for (mjd, expected) in vec![
            (50721, 6 * 60 * 1_000_000_000),
            (50722, 2 * 60 * 1_000_000_000),
            (50723, 14 * 60 * 1_000_000_000),
            (50724, 10 * 60 * 1_000_000_000),
            (59507, 14 * 60 * 1_000_000_000),
            (59508, 10 * 60 * 1_000_000_000),
            (59509, 6 * 60 * 1_000_000_000),
            (59510, 2 * 60 * 1_000_000_000),
        ] {
            assert_eq!(Scheduler::t0_offset_nanos(mjd, duration), expected);
        }
    }
    #[test]
    fn next_track_scheduler() {
        for (t, expected) in vec![
            // reference MJD
            (
                Epoch::from_mjd_utc(50722.0),
                Epoch::from_mjd_utc(50722.0) + Duration::from_seconds(120.0),
            ),
            // 1 sec into reference MJD
            (
                Epoch::from_mjd_utc(50722.0) + Duration::from_seconds(1.0),
                Epoch::from_mjd_utc(50722.0) + Duration::from_seconds(120.0),
            ),
            // 10 sec into reference MJD
            (
                Epoch::from_mjd_utc(50722.0) + Duration::from_seconds(1.0),
                Epoch::from_mjd_utc(50722.0) + Duration::from_seconds(120.0),
            ),
            // 1 sec before reference MJD
            (
                Epoch::from_mjd_utc(50722.0) - Duration::from_seconds(1.0),
                Epoch::from_mjd_utc(50722.0) + Duration::from_seconds(120.0),
            ),
            // 10 sec before reference MJD
            (
                Epoch::from_mjd_utc(50722.0) - Duration::from_seconds(10.0),
                Epoch::from_mjd_utc(50722.0) + Duration::from_seconds(120.0),
            ),
            // two tracks into reference MJD
            (
                Epoch::from_mjd_utc(50722.0)
                    + Duration::from_seconds(
                        2.0 * Scheduler::BIPM_TRACKING_DURATION_SECONDS as f64 + 120.0,
                    ),
                Epoch::from_mjd_utc(50722.0)
                    + Duration::from_seconds(
                        2.0 * Scheduler::BIPM_TRACKING_DURATION_SECONDS as f64 + 120.0,
                    ),
            ),
            // two tracks + 10sec into reference MJD
            (
                Epoch::from_mjd_utc(50722.0)
                    + Duration::from_seconds(
                        2.0 * Scheduler::BIPM_TRACKING_DURATION_SECONDS as f64 + 130.0,
                    ),
                Epoch::from_mjd_utc(50722.0)
                    + Duration::from_seconds(
                        3.0 * Scheduler::BIPM_TRACKING_DURATION_SECONDS as f64 + 120.0,
                    ),
            ),
            // two tracks + 950 sec into reference MJD
            (
                Epoch::from_mjd_utc(50722.0)
                    + Duration::from_seconds(
                        2.0 * Scheduler::BIPM_TRACKING_DURATION_SECONDS as f64 + 120.0 + 950.0,
                    ),
                Epoch::from_mjd_utc(50722.0)
                    + Duration::from_seconds(
                        3.0 * Scheduler::BIPM_TRACKING_DURATION_SECONDS as f64 + 120.0,
                    ),
            ),
            // MJD = 59_506
            (
                Epoch::from_mjd_utc(59506.0),
                Epoch::from_mjd_utc(59506.0) + Duration::from_seconds(2.0 * 60.0),
            ),
            // MJD = 59_507
            (
                Epoch::from_mjd_utc(59507.0),
                Epoch::from_mjd_utc(59507.0) + Duration::from_seconds(14.0 * 60.0),
            ),
            // MJD = 59_508
            (
                Epoch::from_mjd_utc(59508.0),
                Epoch::from_mjd_utc(59508.0) + Duration::from_seconds(10.0 * 60.0),
            ),
            // MJD = 59_509
            (
                Epoch::from_mjd_utc(59509.0),
                Epoch::from_mjd_utc(59509.0) + Duration::from_seconds(6.0 * 60.0),
            ),
        ] {
            let tracker = Scheduler::default();
            let next_track_start = tracker.next_track_start(t);
            println!("next track start: {:?}", next_track_start);
            let error_nanos = (next_track_start - expected).abs().total_nanoseconds();
            assert!(
                error_nanos < 10,
                "failed for {} with {} ns of error",
                t,
                error_nanos
            );
        }
    }
    #[test]
    fn verify_bipm_track_definition() {
        assert_eq!(
            Scheduler::bipm_tracking_duration(),
            Duration::from_seconds(Scheduler::BIPM_TRACKING_DURATION_SECONDS as f64)
        );
    }
}
