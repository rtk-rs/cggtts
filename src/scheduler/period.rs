//! Common View Period definition
use crate::prelude::Duration;

/// Standard setup duration (in seconds), as per BIPM specifications.
const BIPM_SETUP_DURATION_SECONDS: u32 = 180;

/// Standard tracking duration (in seconds), as per BIPM specifications
const BIPM_TRACKING_DURATION_SECONDS: u32 = 780;

/// Reference MJD used in Common View tracking
pub(crate) const REFERENCE_MJD: u32 = 50_722;

/// [CommonViewPeriod] describes the period of satellite
/// tracking and common view realizations.
#[derive(Debug, Clone, PartialEq)]
pub struct CommonViewPeriod {
    /// Setup duration, may serve as a warmup [Duration] at the beginning
    /// of each period. Historically, this was a 3' duration that is still
    /// in use in strict CGTTTS 2E collection (which is arbitrary).
    pub setup_duration: Duration,
    /// Active tracking [Duration].
    /// In strict CGGTTS 2E collection, is is set to 13'.
    pub tracking_duration: Duration,
}

impl Default for CommonViewPeriod {
    /// Creates a default [CommonViewPeriod] of 13' tracking,
    /// and no dead time.
    fn default() -> Self {
        Self {
            setup_duration: Duration::ZERO,
            tracking_duration: Duration::from_seconds(60.0 * 13.0),
        }
    }
}

impl CommonViewPeriod {
    /// Creates a [CommonViewPeriod] as per historical
    /// BIPM Common View specifications.
    pub fn bipm_common_view_period() -> Self {
        Self::default().with_setup_duration_s(BIPM_SETUP_DURATION_SECONDS as f64)
    }

    /// Returns total period [Duration].
    /// ```
    /// use cggtts::prelude::CommonViewPeriod;
    ///
    /// let bipm = CommonViewPeriod::bipm_common_view_period();
    /// assert_eq!(bipm.total_period().to_seconds(), 960.0);
    /// ```
    pub fn total_duration(&self) -> Duration {
        self.setup_duration + self.tracking_duration
    }

    /// Returns a new [CommonViewPeriod] with desired setup [Duration]
    /// for which data should not be collected (at the beginning of each period)
    pub fn with_setup_duration(&self, setup_duration: Duration) -> Self {
        let mut s = self.clone();
        s.setup_duration = setup_duration;
        s
    }

    /// Returns a new [CommonViewPeriod] with desired setup duration in seconds,
    /// for which data should not be collected (at the beginning of each period)
    pub fn with_setup_duration_s(&self, setup_s: f64) -> Self {
        let mut s = self.clone();
        s.setup_duration = Duration::from_seconds(setup_s);
        s
    }

    /// Returns a new [CommonViewPeriod] with desired tracking [Duration]
    /// for which data should be collected (at the end of each period, after possible
    /// setup [Duration]).
    pub fn with_tracking_duration(&self, tracking_duration: Duration) -> Self {
        let mut s = self.clone();
        s.tracking_duration = tracking_duration;
        s
    }

    /// Returns a new [CommonViewPeriod] with desired tracking duration (in seconds)
    /// for which data should be collected (at the end of each period, after possible
    /// setup [Duration]).
    pub fn with_tracking_duration_s(&self, tracking_s: f64) -> Self {
        let mut s = self.clone();
        s.tracking_duration = Duration::from_seconds(tracking_s);
        s
    }
}

#[cfg(test)]
mod test {

    use super::{BIPM_SETUP_DURATION_SECONDS, BIPM_TRACKING_DURATION_SECONDS};
    use crate::prelude::CommonViewPeriod;

    const BIPM_PERIOD_SPECIFICATION_SECONDS: u32 =
        BIPM_SETUP_DURATION_SECONDS + BIPM_TRACKING_DURATION_SECONDS;

    #[test]
    fn bipm_specifications() {
        let cv = CommonViewPeriod::bipm_common_view_period();
        assert_eq!(cv.total_duration().to_seconds(), 960.0);
        assert_eq!(cv.setup_duration.to_seconds(), 180.0);
        assert_eq!(cv.tracking_duration.to_seconds(), 780.0);
    }

    // #[test]
    // fn mjd_first_day_track() {
    //     let cv = CommonViewPeriod::bipm_common_view_period();

    //     for (mjd, expected) in vec![
    //         (50720, 10 * 60 * 1_000_000_000),
    //         (50721, 6 * 60 * 1_000_000_000),
    //         (50722, 2 * 60 * 1_000_000_000),
    //         (50723, 14 * 60 * 1_000_000_000),
    //         (50724, 10 * 60 * 1_000_000_000),
    //         (59507, 14 * 60 * 1_000_000_000),
    //         (59508, 10 * 60 * 1_000_000_000),
    //         (59509, 6 * 60 * 1_000_000_000),
    //         (59510, 2 * 60 * 1_000_000_000),
    //         (59511, 14 * 60 * 1_000_000_000),
    //         (59026, 2 * 60 * 1_000_000_000),
    //     ] {
    //         let t0_offset = cv.first_track_offset_nanos(mjd);
    //         assert_eq!(t0_offset, expected);
    //     }
    // }

    // #[test]
    // fn cv_next_period_start() {
    //     let cv = CommonViewPeriod::bipm_common_view_period();

    //     let (mjd_59025_last_t, _) = CommonViewPeriod::bipm_common_view_period()
    //         .next_period_start(Epoch::from_mjd_utc(59026.0));

    //     for (now, expected) in vec![
    //         // reference MJD
    //         (
    //             Epoch::from_mjd_utc(50722.0),
    //             Epoch::from_mjd_utc(50722.0) + Duration::from_seconds(120.0),
    //         ),
    //         // 1 sec into reference MJD
    //         (
    //             Epoch::from_mjd_utc(50722.0) + Duration::from_seconds(1.0),
    //             Epoch::from_mjd_utc(50722.0) + Duration::from_seconds(120.0),
    //         ),
    //         // 10 sec into reference MJD
    //         (
    //             Epoch::from_mjd_utc(50722.0) + Duration::from_seconds(10.0),
    //             Epoch::from_mjd_utc(50722.0) + Duration::from_seconds(120.0),
    //         ),
    //         // 1 sec before reference MJD
    //         (
    //             Epoch::from_mjd_utc(50722.0) - Duration::from_seconds(1.0),
    //             Epoch::from_mjd_utc(50722.0) + Duration::from_seconds(120.0),
    //         ),
    //         // 10 sec before reference MJD
    //         (
    //             Epoch::from_mjd_utc(50722.0) - Duration::from_seconds(10.0),
    //             Epoch::from_mjd_utc(50722.0) + Duration::from_seconds(120.0),
    //         ),
    //         // 1 period into reference MJD
    //         (
    //             Epoch::from_mjd_utc(50722.0)
    //                 + BIPM_PERIOD_SPECIFICATION_SECONDS as f64 * Unit::Second
    //                 + 120.0 * Unit::Second,
    //             Epoch::from_mjd_utc(50722.0)
    //                 + BIPM_PERIOD_SPECIFICATION_SECONDS as f64 * Unit::Second
    //                 + 120.0 * Unit::Second,
    //         ),
    //         // two periods into reference MJD
    //         (
    //             Epoch::from_mjd_utc(50722.0)
    //                 + 2.0 * BIPM_PERIOD_SPECIFICATION_SECONDS as f64 * Unit::Second
    //                 + 120.0 * Unit::Second,
    //             Epoch::from_mjd_utc(50722.0)
    //                 + 2.0 * BIPM_PERIOD_SPECIFICATION_SECONDS as f64 * Unit::Second
    //                 + 120.0 * Unit::Second,
    //         ),
    //         // two periods + 10s into reference MJD
    //         (
    //             Epoch::from_mjd_utc(50722.0)
    //                 + 2.0 * BIPM_PERIOD_SPECIFICATION_SECONDS as f64 * Unit::Second
    //                 + 10.0 * Unit::Second
    //                 + 120.0 * Unit::Second,
    //             Epoch::from_mjd_utc(50722.0)
    //                 + 3.0 * BIPM_PERIOD_SPECIFICATION_SECONDS as f64 * Unit::Second
    //                 + 120.0 * Unit::Second,
    //         ),
    //         // two periods + 90% into reference MJD
    //         (
    //             Epoch::from_mjd_utc(50722.0)
    //                 + 2.0 * BIPM_PERIOD_SPECIFICATION_SECONDS as f64 * Unit::Second
    //                 + 950.0 * Unit::Second
    //                 + 120.0 * Unit::Second,
    //             Epoch::from_mjd_utc(50722.0)
    //                 + 3.0 * BIPM_PERIOD_SPECIFICATION_SECONDS as f64 * Unit::Second
    //                 + 120.0 * Unit::Second,
    //         ),
    //         // MJD = 59_506
    //         (
    //             Epoch::from_mjd_utc(59506.0),
    //             Epoch::from_mjd_utc(59506.0) + Duration::from_seconds(2.0 * 60.0),
    //         ),
    //         // MJD = 59_507
    //         (
    //             Epoch::from_mjd_utc(59507.0),
    //             Epoch::from_mjd_utc(59507.0) + Duration::from_seconds(14.0 * 60.0),
    //         ),
    //         // MJD = 59_508
    //         (
    //             Epoch::from_mjd_utc(59508.0),
    //             Epoch::from_mjd_utc(59508.0) + Duration::from_seconds(10.0 * 60.0),
    //         ),
    //         // MJD = 59_509
    //         (
    //             Epoch::from_mjd_utc(59509.0),
    //             Epoch::from_mjd_utc(59509.0) + Duration::from_seconds(6.0 * 60.0),
    //         ),
    //         // MJD = 59_025 N-1
    //         (
    //             Epoch::from_mjd_utc(59025.0) + 23.0 * Unit::Hour + (34.0 * 60.0) * Unit::Second,
    //             Epoch::from_mjd_utc(59025.0) + 23.0 * Unit::Hour + (34.0 * 60.0) * Unit::Second,
    //         ),
    //         // MJD = 59_025 N-2 +10s
    //         (
    //             Epoch::from_mjd_utc(59025.0)
    //                 + 23.0 * Unit::Hour
    //                 + (34.0 * 60.0) * Unit::Second
    //                 + 10.0 * Unit::Second,
    //             Epoch::from_mjd_utc(59025.0) + 23.0 * Unit::Hour + (50.0 * 60.0) * Unit::Second,
    //         ),
    //         // MJD = 59_025 N-2 +950s
    //         (
    //             Epoch::from_mjd_utc(59025.0)
    //                 + 23.0 * Unit::Hour
    //                 + (34.0 * 60.0) * Unit::Second
    //                 + 950.0 * Unit::Second,
    //             Epoch::from_mjd_utc(59025.0) + 23.0 * Unit::Hour + (50.0 * 60.0) * Unit::Second,
    //         ),
    //         // // MJD = 59_025 N-1 => MJD 59_026 N-1
    //         (
    //             Epoch::from_mjd_utc(59025.0) + 23.0 * Unit::Hour + (50.0 * 60.0) * Unit::Second,
    //             Epoch::from_mjd_utc(59025.0) + 23.0 * Unit::Hour + (50.0 * 60.0) * Unit::Second,
    //         ),
    //         // // MJD = 59_025 N-1 +10s => MJD 59_026 T0
    //         (
    //             Epoch::from_mjd_utc(59025.0)
    //                 + 23.0 * Unit::Hour
    //                 + (50.0 * 60.0 + 10.0) * Unit::Second,
    //             mjd_59025_last_t,
    //         ),
    //         // // MJD = 59_025 N-1 +10s => MJD 59_026 T0
    //         (
    //             Epoch::from_mjd_utc(59025.0)
    //                 + 23.0 * Unit::Hour
    //                 + (50.0 * 60.0 + 10.0) * Unit::Second,
    //             Epoch::from_mjd_utc(59026.0) + (2.0 * 60.0) * Unit::Second,
    //         ),
    //     ] {
    //         let (next_start, _) = cv.next_period_start(now);
    //         let error_nanos = (next_start - expected).abs().total_nanoseconds();
    //         assert!(
    //             error_nanos < 1,
    //             "test failed for now={:?} got {:?} (error is {}ns)",
    //             now,
    //             next_start,
    //             error_nanos
    //         );
    //     }
    // }
}
