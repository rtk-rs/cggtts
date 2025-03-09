//! Satellite track fitting
use polyfit_rs::polyfit_rs::polyfit;
use thiserror::Error;

use crate::prelude::{Duration, Epoch, FittedData, IonosphericData, TrackData, SV};

use std::collections::BTreeMap;

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
    /// At least two symbols are required to fit
    #[error("track fitting requires at least 2 observations")]
    NotEnoughSymbols,
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

/// [SVTracker] is used to track an individual [SV].
#[derive(Default, Debug, Clone)]
pub struct SVTracker {
    /// SV being tracked
    sv: SV,
    /// Symbols counter
    size: usize,
    /// Sampling gap tolerance
    gap_tolerance: Option<Duration>,
    /// Internal buffer
    buffer: Vec<Observation>,
}

/// [Observation] you need to provide to attempt a CGGTTS fit.
#[derive(Debug, Default, Clone)]
pub struct Observation {
    /// Epoch of [Observation]
    pub epoch: Epoch,
    /// Satellite onboard clock offset to local clock
    pub refsv: f64,
    /// Satellite onboard clock offset to timescale
    pub refsys: f64,
    /// Modeled Tropospheric Delay in seconds of propagation delay
    pub mdtr: f64,
    /// Elevation in degrees
    pub elevation: f64,
    /// Azimuth in degrees
    pub azimuth: f64,
    /// Modeled Ionospheric Delay in seconds of propagation delay
    pub mdio: Option<f64>,
    /// Measured Ionospheric Delay in seconds of propagation delay
    pub msio: Option<f64>,
}

impl SVTracker {
    /// Creates (allocates) a new [SVTracker] for that particular satellite.
    ///
    /// ## Input
    /// - satellite: as [SV]
    /// - tolerance: sampling gap tolerance as [Duration] 
    pub fn new(sv: SV, gap_tolerance: Option<Duration>) -> Self {
        Self {
            sv,
            size: 0,
            gap_tolerance,
            last_epoch: None,
            buffer: Vec::with_capacity(16),
        }
    }

    /// Track fitting attempt.
    pub fn fit(
        &self,
        trk_duration: Duration,
        sampling_period: Duration,
        trk_midpoint: Epoch,
    ) -> Result<((f64, f64), TrackData, Option<IonosphericData>), FitError> {

        if self.size < 2 {
            return Err(FitError::NotEnoughSymbols);
        }

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

    /// Provide new [Observation] at t [Epoch] of observation.
    /// Although CGGTTS works in UTC internally, we accept any timescale here.
    /// Samples must be provided in chronological order.
    pub fn new_observation(&mut self, t: Epoch, data: Observation) {

        if let Some(past_t) = self.last_epoch {
            if let Some(tolerance) = self.gap_tolerance {
                if t > past_t {

                }
            }
        }

        self.buffer.insert(t, data);
        self.size += 1;
    }

    /// Reset and flush the internall buffer.
    /// You should call this any time the satellite being tracked goes out of sight.
    pub fn reset(&mut self) {
        self.size = 0;
        self.buffer.clear();
    }

    /// True if at least one measurement has been latched
    pub fn not_empty(&self) -> bool {
        self.size > 0
    }

}