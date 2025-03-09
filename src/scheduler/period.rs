//! Common View Period definition
use crate::prelude::Duration;

/// Standard setup duration (in seconds), as per BIPM specifications.
pub(crate) const BIPM_SETUP_DURATION_SECONDS: u32 = 180;

/// Standard tracking duration (in seconds), as per BIPM specifications
const BIPM_TRACKING_DURATION_SECONDS: u32 = 780;

/// Reference MJD used in Common View tracking
pub(crate) const BIPM_REFERENCE_MJD: u32 = 50_722;

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
            tracking_duration: Duration::from_seconds(BIPM_TRACKING_DURATION_SECONDS as f64),
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
    /// assert_eq!(bipm.total_duration().to_seconds(), 960.0);
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
    use crate::prelude::CommonViewPeriod;

    #[test]
    fn bipm_specifications() {
        let cv = CommonViewPeriod::bipm_common_view_period();
        assert_eq!(cv.total_duration().to_seconds(), 960.0);
        assert_eq!(cv.setup_duration.to_seconds(), 180.0);
        assert_eq!(cv.tracking_duration.to_seconds(), 780.0);
    }
}
