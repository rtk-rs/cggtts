//! Common View Planification table

use crate::prelude::CommonViewPeriod;

use hifitime::{
    errors::HifitimeError,
    prelude::{Duration, Epoch, TimeScale, Unit},
};

use crate::scheduler::period::{BIPM_REFERENCE_MJD, BIPM_SETUP_DURATION_SECONDS};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("an integral number of cv-periods must fit within a day")]
    UnalignedCvPeriod,
}

/// [CommonViewCalendar] is a serie of evenly spaced [CommonViewPeriod]s.
#[derive(Debug, Clone, PartialEq)]
pub struct CommonViewCalendar {
    /// Reference [Epoch]. In historical CGGTTS, this is
    /// MJD 50_722 + 2'.
    reference_epoch: Epoch,
    /// Reference [Epoch] rounded to midnight that day
    reference_epoch_mjd_midnight: u32,
    /// Holds the possible number of nanoseconds offset from Reference [Epoch] midnight
    reference_epoch_midnight_offset_nanos: i128,
    /// Number of integral [CommonViewPeriod]s per day
    periods_per_day: u16,
    /// Abitrary Daily offset. In historical CGGTTS, this is -4'
    /// to remain aligned with GPS sideral period.
    daily_offset: Duration,
    /// [CommonViewPeriod] specifications.
    period: CommonViewPeriod,
}

impl CommonViewCalendar {
    /// Returns "now" (system time) expressed in [TimeScale::UTC].
    fn now_utc() -> Result<Epoch, HifitimeError> {
        Ok(Epoch::now()?.to_time_scale(TimeScale::UTC))
    }

    /// Design a new [CommonViewCalendar] (planification table).
    ///
    /// ## Input
    /// - reference_epoch: reference [Epoch] used in the scheduling process.  
    /// In historical CGGTTS, this is MJD 50_722 +2'.
    ///
    /// - period: [CommonViewPeriod] specifications.
    /// The total [CommonViewPeriod] must be a perfect multiple of a day,
    /// we do not support a fractional number of daily periods.
    pub fn new(reference_epoch: Epoch, period: CommonViewPeriod) -> Result<Self, Error> {
        let total_duration = period.total_duration().to_seconds();
        let one_day = Duration::from_days(1.0).to_seconds();

        let r = one_day / total_duration;
        let periods_per_day = r.floor() as u16;

        if r.fract() == 0.0 {
            let reference_mjd_midnight = reference_epoch.round(1.0 * Unit::Day);

            let reference_epoch_midnight_offset_nanos =
                (reference_epoch - reference_mjd_midnight).total_nanoseconds();

            Ok(Self {
                period,
                periods_per_day,
                reference_epoch,
                daily_offset: Duration::ZERO,
                reference_epoch_mjd_midnight: {
                    reference_mjd_midnight.to_mjd_utc_days().floor() as u32
                },
                reference_epoch_midnight_offset_nanos,
            })
        } else {
            Err(Error::UnalignedCvPeriod)
        }
    }

    /// Builds the standardized [CommonViewCalendar]
    /// following the CGGTTS historical specifications:
    ///
    /// - aligned to 50_722 MJD + 2'
    /// - -4' daily offset to remain aligned to GPS sideral time
    /// - 16' periods made of 13' data collection following
    /// a 3' warmup phase that is repeated at the beginning of each period.
    ///
    /// ```
    /// use cggtts::prelude::CommonViewCalendar;
    ///
    /// let bipm_calendar = CommonViewCalendar::bipm();
    /// assert_eq!(bipm_calendar.periods_per_day(), 90);
    /// ```
    pub fn bipm() -> Self {
        Self::bipm_unaliged_gps_sideral().with_daily_offset(Duration::from_seconds(-240.0))
    }

    /// Creates the standardized [CommonViewCalendar]
    /// following the CGGTTS historical specifications, except that
    /// it is not aligned to GPS sideral time.
    ///
    /// - aligned to 50_722 MJD + 2'
    /// - 16' periods made of 13' data collection following
    /// a 3' warmup phase that is repeated at the beginning of each period.
    ///
    /// ```
    /// use cggtts::prelude::CommonViewCalendar;
    ///
    /// let bipm_calendar = CommonViewCalendar::bipm_unaliged_gps_sideral();
    /// assert_eq!(bipm_calendar.periods_per_day(), 90);
    /// ```
    pub fn bipm_unaliged_gps_sideral() -> Self {
        let period = CommonViewPeriod::bipm_common_view_period();

        let reference_epoch = Epoch::from_mjd_utc(BIPM_REFERENCE_MJD as f64) + 120.0 * Unit::Second;

        Self::new(reference_epoch, period).unwrap()
    }

    /// Returns the total number of complete [CommonViewPeriod]s per day.
    pub const fn periods_per_day(&self) -> u16 {
        self.periods_per_day
    }

    /// Returns the total duration from the [CommonViewPeriod] specifications.
    pub fn total_period_duration(&self) -> Duration {
        self.period.total_duration()
    }

    /// Add a Daily offset to this [CommonViewCalendar].
    /// A default [CommonViewCalendar] is calculated from the original reference point.
    /// It is possible to add (either positive or negative) offset to apply each day.
    /// Strict CGGTTS uses a -4' offset to remain aligned to GPS sideral time.
    pub fn with_daily_offset(&self, offset: Duration) -> Self {
        let mut s = self.clone();
        s.daily_offset = offset;
        s
    }

    // Returns offset (in nanoseconds) of the i-th CV period starting point
    // for that MJD.
    fn period_start_offset_nanos(&self, mjd: u32, ith: u16) -> i128 {
        // compute offset to reference epoch
        let total_duration_nanos = self.period.total_duration().total_nanoseconds();

        // period offset
        let mut offset_nanos = ith as i128 * total_duration_nanos;

        // offset from midnight (if any)
        offset_nanos += self.reference_epoch_midnight_offset_nanos;

        // daily shift (if any)
        if self.has_daily_offset() {
            let days_offset = (mjd as i128 - self.reference_epoch_mjd_midnight as i128);
            offset_nanos += self.daily_offset.total_nanoseconds() * days_offset;
        }

        offset_nanos
    }

    /// Returns true if a daily offset is defined
    fn has_daily_offset(&self) -> bool {
        self.daily_offset != Duration::ZERO
    }

    // Returns daily offset (in nanoseconds) of the very first period of that MJD.
    fn first_start_offset_nanos(&self, mjd: u32) -> i128 {
        self.period_start_offset_nanos(mjd, 0)
    }

    // /// Next track start time, compared to given Epoch.
    // pub fn next_track_start(&self, t: Epoch) -> Epoch {
    //     let utc_t = match t.time_scale {
    //         TimeScale::UTC => t,
    //         _ => Epoch::from_utc_duration(t.to_utc_duration()),
    //     };

    //     let trk_duration = self.trk_duration;
    //     let mjd = utc_t.to_mjd_utc_days();
    //     let mjd_u = mjd.floor() as u32;

    //     let mjd_next = Epoch::from_mjd_utc((mjd_u + 1) as f64);
    //     let time_to_midnight = mjd_next - utc_t;

    //     match time_to_midnight < trk_duration {
    //         true => {
    //             /*
    //              * if we're in the last track of the day,
    //              * we need to consider next day (MJD+1)
    //              */
    //             let offset_nanos = Self::t0_offset_nanos(mjd_u + 1, trk_duration);
    //             Epoch::from_mjd_utc((mjd_u + 1) as f64)
    //                 + Duration::from_nanoseconds(offset_nanos as f64)
    //         },
    //         false => {
    //             let offset_nanos = Self::t0_offset_nanos(mjd_u, trk_duration);

    //             // determine track number this "t" contributes to
    //             let day_offset_nanos =
    //                 (utc_t - Epoch::from_mjd_utc(mjd_u as f64)).total_nanoseconds() - offset_nanos;
    //             let i = (day_offset_nanos as f64 / trk_duration.total_nanoseconds() as f64).ceil();

    //             let mut e = Epoch::from_mjd_utc(mjd_u as f64)
    //                 + Duration::from_nanoseconds(offset_nanos as f64);

    //             // on first track of day: we only have the day nanos offset
    //             if i > 0.0 {
    //                 // add ith track offset
    //                 e += Duration::from_nanoseconds(i * trk_duration.total_nanoseconds() as f64);
    //             }
    //             e
    //         },
    //     }
    // }

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
            period::{CommonViewPeriod, BIPM_REFERENCE_MJD},
        },
    };

    #[test]
    fn test_bipm() {
        const MJD0_OFFSET_NANOS: i128 = 120_000_000_000;
        const DAILY_OFFSET_NANOS: i128 = -240_000_000_000;
        const PERIOD_DURATION_NANOS_128: i128 = 960_000_000_000;

        let calendar = CommonViewCalendar::bipm();

        assert_eq!(calendar.periods_per_day, 90);
        assert_eq!(calendar.reference_epoch_mjd_midnight, BIPM_REFERENCE_MJD);
        assert_eq!(
            calendar.reference_epoch_midnight_offset_nanos,
            MJD0_OFFSET_NANOS
        );

        // MJD (ith=0): 2' to midnight (like the infamous song)
        assert_eq!(
            calendar.first_start_offset_nanos(BIPM_REFERENCE_MJD),
            MJD0_OFFSET_NANOS
        );

        // Test all tracks for MJD0
        for ith_track in 0..90 {
            assert_eq!(
                calendar.period_start_offset_nanos(BIPM_REFERENCE_MJD, ith_track),
                MJD0_OFFSET_NANOS + ith_track as i128 * PERIOD_DURATION_NANOS_128,
                "failed for MJD0 | track={}",
                ith_track
            );
        }

        // Test for MJD-1..-89 (days prior MJD0)
        for mjd_offset in 1..90 {
            let daily_offset_nanos = MJD0_OFFSET_NANOS - mjd_offset as i128 * DAILY_OFFSET_NANOS;
            let mjd = BIPM_REFERENCE_MJD - mjd_offset;

            // Test all periods for that day
            for i in 0..90 {
                assert_eq!(
                    calendar.period_start_offset_nanos(mjd, i),
                    daily_offset_nanos + i as i128 * PERIOD_DURATION_NANOS_128,
                    "failed for MJD0 -{} | period={}",
                    mjd_offset,
                    i,
                );
            }
        }

        // Test for MJD+1..+89 (days after MJD0)
        for mjd_offset in 1..90 {
            let daily_offset_nanos = MJD0_OFFSET_NANOS + mjd_offset as i128 * DAILY_OFFSET_NANOS;
            let mjd = BIPM_REFERENCE_MJD + mjd_offset;

            // Test all periods for that day
            for i in 0..90 {
                assert_eq!(
                    calendar.period_start_offset_nanos(mjd, i),
                    daily_offset_nanos + i as i128 * PERIOD_DURATION_NANOS_128,
                    "failed for MJD0 +{} | period={}",
                    mjd_offset,
                    i,
                );
            }
        }

        // test a few verified values
        // (50721, 6 * 60 * 1_000_000_000),
        // (50722, 2 * 60 * 1_000_000_000),
        // (50723, 14 * 60 * 1_000_000_000),
        // (50724, 10 * 60 * 1_000_000_000),
        // (59507, 14 * 60 * 1_000_000_000),
        // (59508, 10 * 60 * 1_000_000_000),
        // (59509, 6 * 60 * 1_000_000_000),
        // (59510, 2 * 60 * 1_000_000_000),
    }

    #[test]
    fn test_bipm_not_sideral_gps() {
        const MJD0_OFFSET_NANOS: i128 = 120_000_000_000;
        const PERIOD_DURATION_NANOS_128: i128 = 960_000_000_000;

        let calendar = CommonViewCalendar::bipm_unaliged_gps_sideral();

        assert_eq!(calendar.periods_per_day, 90);
        assert_eq!(calendar.reference_epoch_mjd_midnight, BIPM_REFERENCE_MJD);
        assert_eq!(
            calendar.reference_epoch_midnight_offset_nanos,
            MJD0_OFFSET_NANOS
        );

        // MJD0 (ith=0): 2' to midnight (like the infamous song)
        assert_eq!(
            calendar.first_start_offset_nanos(BIPM_REFERENCE_MJD),
            MJD0_OFFSET_NANOS
        );

        // Test all Periods for MJD0
        for i in 0..90 {
            assert_eq!(
                calendar.period_start_offset_nanos(BIPM_REFERENCE_MJD, i),
                MJD0_OFFSET_NANOS + i as i128 * PERIOD_DURATION_NANOS_128,
                "failed for MJD0 | period={}",
                i
            );
        }

        // Test for MJD-1..-89 (days prior MJD0)
        for mjd_offset in 1..90 {
            let daily_offset_nanos = 120_000_000_000;

            let mjd = BIPM_REFERENCE_MJD - mjd_offset;

            // Test all Periods for that day
            for i in 0..90 {
                assert_eq!(
                    calendar.period_start_offset_nanos(mjd, i),
                    daily_offset_nanos + i as i128 * PERIOD_DURATION_NANOS_128,
                    "failed for MJD -{} | period={}",
                    mjd_offset,
                    i,
                );
            }
        }

        // Test for MJD+1..89 (days after MJD0)
        for mjd_offset in 1..90 {
            let daily_offset_nanos = 120_000_000_000;

            let mjd = BIPM_REFERENCE_MJD + mjd_offset;

            // Test all tracks for that day
            for i in 0..90 {
                assert_eq!(
                    calendar.period_start_offset_nanos(mjd, i),
                    daily_offset_nanos + i as i128 * PERIOD_DURATION_NANOS_128,
                    "failed for MJD +{} | period={}",
                    mjd_offset,
                    i,
                );
            }
        }
    }
}
