//! Common View Planification table

use crate::prelude::{CommonViewPeriod, Duration, Epoch, TimeScale};
use hifitime::errors::HifitimeError;

use super::period::REFERENCE_MJD;

/// [CommonViewCalendar] is a serie of evenly spaced [CommonViewPeriod]s.
pub struct CommonViewCalendar {
    /// Start time, as [Epoch] which marks the beginning
    /// of the very first [CommonViewPeriod].
    start_time: Epoch,
    /// [CommonViewPeriod] specifications
    period: CommonViewPeriod,
    /// Indicates whether this is the first tracking in a MJD.
    is_t0: bool,
}

impl CommonViewCalendar {
    /// Returns "now" (system time) expressed in [TimeScale::UTC].
    fn now_utc() -> Result<Epoch, HifitimeError> {
        Ok(Epoch::now()?.to_time_scale(TimeScale::UTC))
    }

    /// Design a new [CommonViewCalendar] (planification table)
    /// starting "right now", as opposed to [Self::new].  
    ///
    /// NB: [CommonViewCalendar] does not allow partial [CommonViewPeriod]s,
    /// so the tracking will be active once the pending period is completed.  
    ///
    /// NB: whatever your [CommonViewPeriod] specifications might be, we still
    /// align to the original BIPM T0, set to MJD=50_722 midnight +2'.
    ///
    /// ## Input
    /// - period: [CommonViewPeriod] specifications
    pub fn now(period: CommonViewPeriod) -> Result<Self, HifitimeError> {
        let now = Self::now_utc()?;
        Ok(Self::new(now, period))
    }

    /// Creates a [CommonViewCalendar] (planification table) starting point in time,
    /// which can either be future (usual case) or in the past.  
    ///
    /// NB: [CommonViewCalendar] does not allow partial [CommonViewPeriod]s,
    /// so the tracking will be active once the pending period is completed.  
    ///
    /// NB: whatever your [CommonViewPeriod] specifications might be, we still
    /// align to the original BIPM T0, set to MJD=50_722 midnight +2'.
    ///
    /// ## Input
    /// - start_time: start time as [Epoch]
    /// - period: your [CommonViewPeriod] specifications
    pub fn new(start_time: Epoch, period: CommonViewPeriod) -> Self {
        let (start_time, is_t0) = period.next_period_start(start_time);
        Self {
            start_time,
            is_t0,
            period,
        }
    }

    /// Returns true if this [CommonViewCalendar] is currently deployed,
    /// meaning that we're past the starting point of the very first [CommonViewPeriod].
    pub fn is_currently_deployed(&self) -> Result<bool, HifitimeError> {
        let now = Self::now_utc()?;
        Ok(self.is_deployed_at(now))
    }

    /// Returns true if this [CommonViewCalendar] will be declared as deployed (see [Self::is_currently_deployed])
    /// at this particular point in time.
    pub fn is_deployed_at(&self, t: Epoch) -> bool {
        t >= self.start_time
    }

    /// Returns true if this [CommonViewCalendar] is currently deployed & active.
    /// Meaning, we're past the starting point of the very first [CommonViewPeriod]
    /// and we're inside an active phase of the ongoing [CommonViewPeriod].
    pub fn is_currently_active(&self) -> Result<bool, HifitimeError> {
        let now = Self::now_utc()?;
        if !self.is_deployed_at(now) {
            return Ok(false);
        }
        Ok(self.is_active_at(now))
    }

    /// Returns true if this [CommonViewCalendar] is declared as active
    /// (see [Self::is_currently_active]) at this particular point in time.
    pub fn is_active_at(&self, t: Epoch) -> bool {
        if !self.is_deployed_at(t) {
            return false;
        }
        false
    }

    /// Helper to determine how long until a next "synchronous" track.
    pub fn time_to_next_track(&self, now: Epoch) -> Duration {
        self.next_track_start(now) - now
    }

    // Offset (in nanoseconds) of the very first Track for that MJD (midnight)
    // if following the CGGTTS specs.
    pub(crate) fn t0_offset_nanos(mjd: u32, trk_duration: Duration) -> i128 {
        let tracking_nanos = trk_duration.total_nanoseconds();

        let mut offset_nanos = (REFERENCE_MJD - mjd) as i128;

        offset_nanos *= 2 * 1_000_000_000 * 60; // offset to midnight on reference MJD
        offset_nanos *= 4 * 1_000_000_000 * 60; // shift per day in CGGTTS specs
        offset_nanos = offset_nanos % trk_duration.total_nanoseconds();

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

    // /// Returns true if we're currently inside an observation period (active measurement).
    // /// To respect this [CommonViewCalendar] table, your measurement system should be active!
    // pub fn active_measurement(&self) -> Result<bool, HifitimeError> {
    //     let now = Self::now_utc()?;
    //     if now > self.start_time {
    //         // we're inside a cv-period
    //         Ok(false)
    //     } else {
    //         // not inside a cv-period
    //         Ok(false)
    //     }
    // }

    // /// Returns remaining [Duration] before beginning of next
    // /// [CommonViewPeriod]. `now` may be any [Epoch]
    // /// but is usually `now()` when actively tracking.
    // /// Although CGGTTS uses UTC strictly, we accept any timescale here.
    // pub fn time_to_next_period(now: Epoch) -> Duration {
    //     let (next_period_start, _) = Self::next_period_start(now);
    //     next_period_start - now
    // }

    // /// Offset of first track for any given MJD, expressed in nanoseconds
    // /// within that day.
    // fn first_track_offset_nanos(mjd: u32) -> i128 {
    //     if self.setup_duration != Duration::from_seconds(BIPM_SETUP_DURATION_SECONDS as f64)
    //         || self.tracking_duration
    //             != Duration::from_seconds(BIPM_TRACKING_DURATION_SECONDS as f64)
    //     {
    //         return 0i128;
    //     }

    //     let tracking_nanos = self.total_period().total_nanoseconds();

    //     let mjd_difference = REFERENCE_MJD as i128 - mjd as i128;

    //     let offset_nanos = (mjd_difference
    //         // this is the shift per day
    //         * 4 * 1_000_000_000 * 60
    //         // this was the offset on MJD reference
    //         + 2 * 1_000_000_000 * 60)
    //         % tracking_nanos;

    //     if offset_nanos < 0 {
    //         offset_nanos + tracking_nanos
    //     } else {
    //         offset_nanos
    //     }
    // }

    // /// Returns the date and time of the next [CommonViewPeriod] expressed as an [Epoch]
    // /// and a boolean indicating whether the next [CommonViewPeriod] is `t0`.
    // /// `now` may be any [Epoch]
    // /// but is usually `now()` when actively tracking.
    // /// Although CGGTTS uses UTC strictly, we accept any timescale here.
    // pub fn next_period_start(&self, now: Epoch) -> (Epoch, bool) {
    //     let total_period = self.total_period();
    //     let total_period_nanos = total_period.total_nanoseconds();

    //     let now_utc = match now.time_scale {
    //         TimeScale::UTC => now,
    //         _ => Epoch::from_utc_duration(now.to_utc_duration()),
    //     };

    //     let mjd_utc = (now_utc.to_mjd_utc_days()).floor() as u32;
    //     let today_midnight_utc = Epoch::from_mjd_utc(mjd_utc as f64);

    //     let today_t0_offset_nanos = self.first_track_offset_nanos(mjd_utc);
    //     let today_offset_nanos = (now_utc - today_midnight_utc).total_nanoseconds();

    //     let today_t0_utc = today_midnight_utc + (today_t0_offset_nanos as f64) * Unit::Nanosecond;

    //     if today_offset_nanos < today_t0_offset_nanos {
    //         // still within first track
    //         (today_t0_utc, true)
    //     } else {
    //         let ith_period = (((now_utc - today_t0_utc).total_nanoseconds() as f64)
    //             / total_period_nanos as f64)
    //             .ceil() as i128;

    //         let number_periods_per_day = (24 * 3600 * 1_000_000_000) / total_period_nanos;

    //         if ith_period >= number_periods_per_day {
    //             let tomorrow_t0_offset_nanos = self.first_track_offset_nanos(mjd_utc + 1);

    //             (
    //                 Epoch::from_mjd_utc((mjd_utc + 1) as f64)
    //                     + tomorrow_t0_offset_nanos as f64 * Unit::Nanosecond,
    //                 false,
    //             )
    //         } else {
    //             (
    //                 today_midnight_utc
    //                     + today_t0_offset_nanos as f64 * Unit::Nanosecond
    //                     + (ith_period * total_period_nanos) as f64 * Unit::Nanosecond,
    //                 false,
    //             )
    //         }
    //     }
    // }
}

#[cfg(test)]
mod test {

    use crate::{
        prelude::Epoch,
        scheduler::{
            calendar::CommonViewCalendar,
            period::{CommonViewPeriod, REFERENCE_MJD},
        },
    };

    #[test]
    fn test_13min_no_dead_time() {
        // D0 in the CGGTTS specs
        let period = CommonViewPeriod::default(); // 13'
        let start_time = Epoch::from_mjd_utc(REFERENCE_MJD as f64);
        let calendar = CommonViewCalendar::new(start_time, period).unwrap();

        // D-1 in the CGGTTS specs (verify we support that as well!)
    }

    #[test]
    fn test_bipm_13min_3min_dead_time() {
        // D0 in the CGGTTS specs
        let start_time = Epoch::from_mjd_utc(REFERENCE_MJD);
        let period = CommonViewPeriod::default();
        let calendar = CommonViewCalendar::new(start_time, period);
    }

    // (50721, 6 * 60 * 1_000_000_000),
    // (50722, 2 * 60 * 1_000_000_000),
    // (50723, 14 * 60 * 1_000_000_000),
    // (50724, 10 * 60 * 1_000_000_000),
    // (59507, 14 * 60 * 1_000_000_000),
    // (59508, 10 * 60 * 1_000_000_000),
    // (59509, 6 * 60 * 1_000_000_000),
    // (59510, 2 * 60 * 1_000_000_000),
}
