//! Common View Planification table
use crate::scheduler::period::{CommonViewPeriod, BIPM_REFERENCE_MJD};
use hifitime::prelude::{Duration, Epoch, TimeScale, Unit};
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

    /// Returns true if a daily offset is defined
    fn has_daily_offset(&self) -> bool {
        self.daily_offset != Duration::ZERO
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
            let days_offset = mjd as i128 - self.reference_epoch_mjd_midnight as i128;
            offset_nanos += self.daily_offset.total_nanoseconds() * days_offset as i128;
        }

        offset_nanos %= total_duration_nanos;

        if offset_nanos.is_negative() {
            offset_nanos += total_duration_nanos;
        }

        offset_nanos
    }

    // Returns daily offset (in nanoseconds) of the very first period of that MJD.
    fn first_start_offset_nanos(&self, mjd: u32) -> i128 {
        self.period_start_offset_nanos(mjd, 0)
    }

    /// Returns datetime (as [Epoch]) of next [CommonViewPeriod] after
    /// specified [Epoch]. Although CGGTTS is scheduled in and aligned
    /// to [TimeScale::UTC], we tolerate other timescales here.
    pub fn next_period_start_after(&self, t: Epoch) -> Epoch {
        let ts = t.time_scale;
        let utc = ts == TimeScale::UTC;
        let period_duration = self.period.total_duration();

        let t_utc = if utc {
            t
        } else {
            t.to_time_scale(TimeScale::UTC)
        };

        // determine time to next midnight
        let mjd = t_utc.to_mjd_utc_days().floor() as u32;

        let mjd_midnight = Epoch::from_mjd_utc(mjd as f64);
        let mjd_next_midnight = Epoch::from_mjd_utc((mjd + 1) as f64);
        let time_to_midnight = mjd_next_midnight - mjd_midnight;

        let t_utc = if time_to_midnight < period_duration {
            // we're inside the last track of that day
            // simply return T0 of MJD+1
            let t0_offset_nanos = self.first_start_offset_nanos(mjd + 1) as f64;
            Epoch::from_duration(t0_offset_nanos * Unit::Nanosecond, TimeScale::UTC)
        } else {
            // offset of first period of that day
            let offset_nanos = self.first_start_offset_nanos(mjd) as f64;
            let t0_utc = mjd_midnight + offset_nanos * Unit::Nanosecond;

            // determine track number this "t" contributes to
            let i = (t_utc - t0_utc).total_nanoseconds() / period_duration.total_nanoseconds();

            println!("ith={}", i);

            if t_utc < t0_utc {
                // on first track of day, we only have the daily offset
                t0_utc
            } else {
                t0_utc + ((i + 1) * period_duration.total_nanoseconds()) as f64 * Unit::Nanosecond
            }
        };

        if utc {
            t_utc
        } else {
            t_utc.to_time_scale(ts)
        }
    }

    /// Returns datetime (as [Epoch]) of next active data collection
    /// after specified [Epoch]. Although CGGTTS is scheduled in and aligned
    /// to [TimeScale::UTC], we tolerate other timescales here.
    pub fn next_data_collection_after(&self, t: Epoch) -> Epoch {
        let mut next_t = self.next_period_start_after(t);
        if self.period.setup_duration == Duration::ZERO {
            next_t += self.period.setup_duration;
        }
        next_t
    }

    /// Returns remaining time (as [Duration]) until start of next
    /// [CommonViewPeriod] after specified [Epoch].
    pub fn time_to_next_start(&self, t: Epoch) -> Duration {
        let next_start = self.next_period_start_after(t);
        t - next_start
    }

    /// Returns remaining time (as [Duration]) until start of next
    /// active data collection, after specified [Epoch].
    pub fn time_to_next_data_collection(&self, t: Epoch) -> Duration {
        let mut dt = self.time_to_next_start(t);
        if self.period.setup_duration == Duration::ZERO {
            dt += self.period.setup_duration;
        }
        dt
    }
}

#[cfg(test)]
mod test {

    use crate::{
        prelude::{Duration, Epoch},
        scheduler::{calendar::CommonViewCalendar, period::BIPM_REFERENCE_MJD},
    };

    #[test]
    fn test_bipm() {
        const MJD0_OFFSET_NANOS: i128 = 120_000_000_000;
        const DAILY_OFFSET_NANOS: i128 = -240_000_000_000;
        const PERIOD_DURATION_NANOS_128: i128 = 960_000_000_000;

        let calendar = CommonViewCalendar::bipm();

        assert_eq!(calendar.periods_per_day, 90);
        assert_eq!(calendar.daily_offset.to_seconds(), -240.0);
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
                (MJD0_OFFSET_NANOS + ith_track as i128 * PERIOD_DURATION_NANOS_128)
                    % PERIOD_DURATION_NANOS_128,
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
                    (daily_offset_nanos + i as i128 * PERIOD_DURATION_NANOS_128)
                        % PERIOD_DURATION_NANOS_128,
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
                let mut expected = (daily_offset_nanos - i as i128 * PERIOD_DURATION_NANOS_128)
                    % PERIOD_DURATION_NANOS_128;

                if expected.is_negative() {
                    expected += PERIOD_DURATION_NANOS_128;
                }

                assert_eq!(
                    calendar.period_start_offset_nanos(mjd, i),
                    expected,
                    "failed for MJD0 +{} | period={}",
                    mjd_offset,
                    i,
                );
            }
        }

        // test a few verified values
        for (mjd, t0_nanos) in [
            (50_721, 6 * 60 * 1_000_000_000),
            (50_722, 2 * 60 * 1_000_000_000),
            (50_723, 14 * 60 * 1_000_000_000),
            (50_724, 10 * 60 * 1_000_000_000),
            (59_507, 14 * 60 * 1_000_000_000),
            (59_508, 10 * 60 * 1_000_000_000),
            (59_509, 6 * 60 * 1_000_000_000),
            (59_510, 2 * 60 * 1_000_000_000),
        ] {
            assert_eq!(
                calendar.first_start_offset_nanos(mjd),
                t0_nanos,
                "failed for mdj={}",
                mjd,
            );
        }

        // Test a few verified values
        for (i, (t, expected)) in [
            (
                Epoch::from_mjd_utc(50_722.0),
                Epoch::from_mjd_utc(50_722.0) + Duration::from_seconds(120.0),
            ),
            (
                Epoch::from_mjd_utc(50_722.0) + Duration::from_seconds(1.0),
                Epoch::from_mjd_utc(50_722.0) + Duration::from_seconds(120.0),
            ),
            (
                Epoch::from_mjd_utc(50_722.0) + Duration::from_seconds(2.0),
                Epoch::from_mjd_utc(50_722.0) + Duration::from_seconds(120.0),
            ),
            (
                Epoch::from_mjd_utc(50_722.0) + Duration::from_seconds(10.0),
                Epoch::from_mjd_utc(50_722.0) + Duration::from_seconds(120.0),
            ),
            // TODO
            // (
            //     Epoch::from_mjd_utc(50_722.0) - Duration::from_seconds(10.0),
            //     Epoch::from_mjd_utc(50_722.0) + Duration::from_seconds(120.0),
            // ),
            (
                Epoch::from_mjd_utc(50_722.0) + Duration::from_seconds(119.0),
                Epoch::from_mjd_utc(50_722.0) + Duration::from_seconds(120.0),
            ),
            (
                Epoch::from_mjd_utc(50_722.0) + Duration::from_seconds(120.0),
                Epoch::from_mjd_utc(50_722.0) + Duration::from_seconds(120.0 + 960.0),
            ),
            (
                Epoch::from_mjd_utc(50_722.0) + Duration::from_seconds(121.0),
                Epoch::from_mjd_utc(50_722.0) + Duration::from_seconds(120.0 + 960.0),
            ),
            (
                Epoch::from_mjd_utc(50_722.0) + Duration::from_seconds(120.0 + 959.0),
                Epoch::from_mjd_utc(50_722.0) + Duration::from_seconds(120.0 + 960.0),
            ),
            (
                Epoch::from_mjd_utc(50_722.0) + Duration::from_seconds(120.0 + 960.0),
                Epoch::from_mjd_utc(50_722.0) + Duration::from_seconds(120.0 + 2.0 * 960.0),
            ),
            (
                Epoch::from_mjd_utc(50_722.0) + Duration::from_seconds(120.0 + 961.0),
                Epoch::from_mjd_utc(50_722.0) + Duration::from_seconds(120.0 + 2.0 * 960.0),
            ),
            (
                Epoch::from_mjd_utc(50_722.0) + Duration::from_seconds(120.0 + 2.0 * 960.0 - 1.0),
                Epoch::from_mjd_utc(50_722.0) + Duration::from_seconds(120.0 + 2.0 * 960.0),
            ),
            (
                Epoch::from_mjd_utc(50_722.0) + Duration::from_seconds(120.0 + 2.0 * 960.0),
                Epoch::from_mjd_utc(50_722.0) + Duration::from_seconds(120.0 + 3.0 * 960.0),
            ),
            (
                Epoch::from_mjd_utc(59_506.0),
                Epoch::from_mjd_utc(59_506.0) + Duration::from_seconds(2.0 * 60.0),
            ),
            (
                Epoch::from_mjd_utc(59_507.0),
                Epoch::from_mjd_utc(59_507.0) + Duration::from_seconds(14.0 * 60.0),
            ),
            (
                Epoch::from_mjd_utc(59_508.0),
                Epoch::from_mjd_utc(59_508.0) + Duration::from_seconds(10.0 * 60.0),
            ),
            (
                Epoch::from_mjd_utc(59_509.0),
                Epoch::from_mjd_utc(59_509.0) + Duration::from_seconds(6.0 * 60.0),
            ),
        ]
        .iter()
        .enumerate()
        {
            assert_eq!(
                calendar.next_period_start_after(*t),
                *expected,
                "failed for i={}/t={:?}",
                i,
                t
            );
        }
    }

    #[test]
    fn test_bipm_not_sideral_gps() {
        const MJD0_OFFSET_NANOS: i128 = 120_000_000_000;
        const PERIOD_DURATION_NANOS_128: i128 = 960_000_000_000;

        let calendar = CommonViewCalendar::bipm_unaliged_gps_sideral();

        assert_eq!(calendar.periods_per_day, 90);
        assert_eq!(calendar.daily_offset.to_seconds(), 0.0);
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
