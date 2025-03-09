//! Common View Planification table

use crate::prelude::{CommonViewPeriod, Duration, Epoch, TimeScale};
use hifitime::errors::HifitimeError;

/// [CommonViewCalendar] is a serie of evenly spaced [CommonViewPeriod]s.
pub struct CommonViewCalendar {
    /// Deploy time is the [Epoch] at which
    /// the first [CommonViewPeriod] was launched
    start_time: Epoch,
    /// Indicates whether this is the first tracking in a MJD.
    is_t0: bool,
    /// [CommonViewPeriod] specification
    period: CommonViewPeriod,
}

impl CommonViewCalendar {
    /// Returns "now" (system time) expressed in [TimeScale::UTC].
    fn now_utc() -> Result<Epoch, HifitimeError> {
        Ok(Epoch::now()?.to_time_scale(TimeScale::UTC))
    }

    /// Design a new [CommonViewCalendar] (planification table)
    /// which is a serie of steady [CommonViewPeriod]s starting "right now",
    /// as opposed to [Self::new_postponed].
    /// Whether "right now" is already inside active measurement or not, is up
    /// to your [CommonViewPeriod] specifications.
    pub fn now(period: CommonViewPeriod) -> Result<Self, HifitimeError> {
        let now = Self::now_utc()?;
        let (start_time, is_t0) = period.next_period_start(now);
        Ok(Self {
            start_time,
            is_t0,
            period,
        })
    }

    /// Design a new [CommonViewCalendar] (planification table)
    /// which is a serie of steady [CommonViewPeriod]s
    /// intended to start working later, as opposed to [Self::now].
    pub fn new_postponed(
        postponing: Duration,
        period: CommonViewPeriod,
    ) -> Result<Self, HifitimeError> {
        let now = Self::now_utc()?;
        let (start_time, is_t0) = period.next_period_start(now + postponing);
        Ok(Self {
            start_time,
            is_t0,
            period,
        })
    }

    /// Returns true if this [CommonViewCalendar] is actively working.
    /// That means we're inside a [CommonViewPeriod]. Whether this is
    /// active measurement or not, depends on your [CommonViewPeriod] specifications.
    pub fn active(&self) -> Result<bool, HifitimeError> {
        let now = Self::now_utc()?;
        Ok(now > self.start_time)
    }

    /// Returns true if we're currently inside an observation period (active measurement).
    /// To respect this [CommonViewCalendar] table, your measurement system should be active!
    pub fn active_measurement(&self) -> Result<bool, HifitimeError> {
        let now = Self::now_utc()?;
        if now > self.start_time {
            // we're inside a cv-period
            Ok(false)
        } else {
            // not inside a cv-period
            Ok(false)
        }
    }

    /// Returns remaining [Duration] before beginning of next
    /// [CommonViewPeriod]. `now` may be any [Epoch]
    /// but is usually `now()` when actively tracking.
    /// Although CGGTTS uses UTC strictly, we accept any timescale here.
    pub fn time_to_next_period(now: Epoch) -> Duration {
        let (next_period_start, _) = Self::next_period_start(now);
        next_period_start - now
    }

    /// Offset of first track for any given MJD, expressed in nanoseconds
    /// within that day.
    fn first_track_offset_nanos(mjd: u32) -> i128 {
        if self.setup_duration != Duration::from_seconds(BIPM_SETUP_DURATION_SECONDS as f64)
            || self.tracking_duration
                != Duration::from_seconds(BIPM_TRACKING_DURATION_SECONDS as f64)
        {
            return 0i128;
        }

        let tracking_nanos = self.total_period().total_nanoseconds();

        let mjd_difference = REFERENCE_MJD as i128 - mjd as i128;

        let offset_nanos = (mjd_difference
            // this is the shift per day
            * 4 * 1_000_000_000 * 60
            // this was the offset on MJD reference
            + 2 * 1_000_000_000 * 60)
            % tracking_nanos;

        if offset_nanos < 0 {
            offset_nanos + tracking_nanos
        } else {
            offset_nanos
        }
    }

    /// Returns the date and time of the next [CommonViewPeriod] expressed as an [Epoch]
    /// and a boolean indicating whether the next [CommonViewPeriod] is `t0`.
    /// `now` may be any [Epoch]
    /// but is usually `now()` when actively tracking.
    /// Although CGGTTS uses UTC strictly, we accept any timescale here.
    pub fn next_period_start(&self, now: Epoch) -> (Epoch, bool) {
        let total_period = self.total_period();
        let total_period_nanos = total_period.total_nanoseconds();

        let now_utc = match now.time_scale {
            TimeScale::UTC => now,
            _ => Epoch::from_utc_duration(now.to_utc_duration()),
        };

        let mjd_utc = (now_utc.to_mjd_utc_days()).floor() as u32;
        let today_midnight_utc = Epoch::from_mjd_utc(mjd_utc as f64);

        let today_t0_offset_nanos = self.first_track_offset_nanos(mjd_utc);
        let today_offset_nanos = (now_utc - today_midnight_utc).total_nanoseconds();

        let today_t0_utc = today_midnight_utc + (today_t0_offset_nanos as f64) * Unit::Nanosecond;

        if today_offset_nanos < today_t0_offset_nanos {
            // still within first track
            (today_t0_utc, true)
        } else {
            let ith_period = (((now_utc - today_t0_utc).total_nanoseconds() as f64)
                / total_period_nanos as f64)
                .ceil() as i128;

            let number_periods_per_day = (24 * 3600 * 1_000_000_000) / total_period_nanos;

            if ith_period >= number_periods_per_day {
                let tomorrow_t0_offset_nanos = self.first_track_offset_nanos(mjd_utc + 1);

                (
                    Epoch::from_mjd_utc((mjd_utc + 1) as f64)
                        + tomorrow_t0_offset_nanos as f64 * Unit::Nanosecond,
                    false,
                )
            } else {
                (
                    today_midnight_utc
                        + today_t0_offset_nanos as f64 * Unit::Nanosecond
                        + (ith_period * total_period_nanos) as f64 * Unit::Nanosecond,
                    false,
                )
            }
        }
    }
}
