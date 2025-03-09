use crate::{errors::ParsingError, header::Code};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(docsrs)]
use crate::prelude::CGGTTS;

/// Indication about precise system delay calibration process,
/// as found in [CGGTTS].
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CalibrationID {
    /// ID # of this calibration process
    pub process_id: u16,
    /// Year of calibration
    pub year: u16,
}

impl std::str::FromStr for CalibrationID {
    type Err = ParsingError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parsed_items = 0;
        let (mut process_id, mut year) = (0, 0);

        for (nth, item) in s.trim().split('-').enumerate() {
            if nth == 0 {
                if let Ok(value) = item.parse::<u16>() {
                    process_id = value;
                    parsed_items += 1;
                }
            } else if nth == 1 {
                if let Ok(value) = item.parse::<u16>() {
                    year = value;
                    parsed_items += 1;
                }
            }
        }

        if parsed_items == 2 {
            Ok(Self { process_id, year })
        } else {
            Err(ParsingError::InvalidCalibrationId)
        }
    }
}

/// [Delay] describes all supported types of propagation delay.
/// NB: the specified value is always in nanoseconds.
#[derive(Debug, PartialEq, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Delay {
    /// Delay defined as internal in nanoseconds
    Internal(f64),
    /// Systemic delay, in nanoseconds
    System(f64),
}

impl Default for Delay {
    fn default() -> Delay {
        Delay::System(0.0_f64)
    }
}

impl Delay {
    /// Define new internal [Delay]
    pub fn new_internal_nanos(nanos: f64) -> Self {
        Self::Internal(nanos)
    }

    /// Define new systemic [Delay]
    pub fn new_systemic(nanos: f64) -> Self {
        Self::System(nanos)
    }

    /// Returns total delay in nanoseconds, whatever its kind.
    pub fn total_nanoseconds(&self) -> f64 {
        match self {
            Delay::Internal(d) => *d,
            Delay::System(d) => *d,
        }
    }

    /// Returns total delay in seconds, whatever its kind.
    pub fn total_seconds(&self) -> f64 {
        self.total_nanoseconds() * 1.0E-9
    }

    /// Adds specific amount of nanoseconds to internal delay,
    /// whatever its definition.
    pub fn add_nanos(&self, rhs: f64) -> Self {
        match self {
            Delay::System(d) => Delay::System(*d + rhs),
            Delay::Internal(d) => Delay::Internal(*d + rhs),
        }
    }
}

/// [SystemDelay] describes total measurement systems delay.
/// This is used in [CGGTTS] to describe the measurement system
/// accurately.
///
/// Example of simplistic definition, compatible with
/// very precise single frequency Common View:
/// ```
/// use cggtts::prelude::SystemDelay;
///
/// let system_specs = SystemDelay::default()
///     .with_antenna_cable_delay(10.0)
///     .with_ref_delay(20.0);
///
/// assert_eq!(system_specs.total_cable_delay_nanos(), 30.0);
/// ```
///
/// Example of advanced definition, compatible with
/// ultra precise dual frequency Common View:
/// ```
/// use cggtts::prelude::SystemDelay;
///
/// ```
#[derive(Clone, Default, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SystemDelay {
    /// Delay induced by GNSS antenna cable length.
    pub antenna_cable_delay: f64,
    /// Delay induced by cable between measurement system
    /// and local clock.
    pub local_ref_delay: f64,
    /// Carrier frequency dependend delays
    pub freq_dependent_delays: Vec<(Code, Delay)>,
    /// Possible calibration ID
    pub calibration_id: Option<CalibrationID>,
}

impl SystemDelay {
    /// Define new [SystemDelay] with desired readable calibration ID.
    /// This is usually the official ID of the calibration process.
    pub fn with_calibration_id(&self, calibration: CalibrationID) -> Self {
        Self {
            antenna_cable_delay: self.antenna_cable_delay,
            local_ref_delay: self.local_ref_delay,
            freq_dependent_delays: self.freq_dependent_delays.clone(),
            calibration_id: Some(calibration),
        }
    }

    /// Define new [SystemDelay] with desired
    /// RF cable delay in nanoseconds ie.,
    /// delay induced by the antenna cable length itself.
    pub fn with_antenna_cable_delay(&self, nanos: f64) -> Self {
        let mut s = self.clone();
        s.antenna_cable_delay = nanos;
        s
    }

    /// Define new [SystemDelay] with REF delay in nanoseconds,
    /// ie., the delay induced by cable between the measurement
    /// system and the local clock.
    pub fn with_ref_delay(&self, nanos: f64) -> Self {
        let mut s = self.clone();
        s.local_ref_delay = nanos;
        s
    }

    /// Returns total cable delay in nanoseconds, that will affect all measurements.
    pub fn total_cable_delay_nanos(&self) -> f64 {
        self.antenna_cable_delay + self.local_ref_delay
    }

    /// Returns total system delay, in nanoseconds,
    /// for desired frequency represented by [Code], if we
    /// do have specifications for it.
    ///
    /// ```
    /// ```
    pub fn total_frequency_dependent_delay_nanos(&self, code: &Code) -> Option<f64> {
        for (k, v) in self.freq_dependent_delays.iter() {
            if k == code {
                return Some(v.total_nanoseconds() + self.total_cable_delay_nanos());
            }
        }
        None
    }

    /// Iterates over all frequency dependent delays, per carrier frequency,
    /// in nanoseconds of propagation delay for said frequency.
    pub fn frequency_dependent_nanos_delay_iter(
        &self,
    ) -> Box<dyn Iterator<Item = (&Code, f64)> + '_> {
        Box::new(
            self.freq_dependent_delays
                .iter()
                .map(move |(k, v)| (k, v.total_nanoseconds() + self.total_cable_delay_nanos())),
        )
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use std::str::FromStr;

    #[test]
    fn calibration_id() {
        let calibration = CalibrationID::from_str("1015-2024").unwrap();
        assert_eq!(calibration.process_id, 1015);
        assert_eq!(calibration.year, 2024);

        assert!(CalibrationID::from_str("NA").is_err());
        assert!(CalibrationID::from_str("1nnn-2024").is_err());
    }

    #[test]
    fn test_delay() {
        let delay = Delay::Internal(10.0);
        assert_eq!(delay.total_nanoseconds(), 10.0);

        assert_eq!(delay.total_seconds(), 10.0E-9);
        assert!(delay == Delay::Internal(10.0));
        assert!(delay != Delay::System(10.0));

        let d = delay.add_nanos(20.0);
        assert_eq!(d, Delay::Internal(30.0));
        assert_eq!(delay.total_nanoseconds() + 20.0, d.total_nanoseconds());
        assert_eq!(Delay::default(), Delay::System(0.0));

        let delay = Delay::System(30.5);
        assert_eq!(delay.total_nanoseconds(), 30.5);

        let d = delay.add_nanos(20.0);
        assert_eq!(d.total_nanoseconds(), 50.5);
    }

    #[test]
    fn test_system_delay() {
        let delay = SystemDelay::default();
        assert_eq!(delay.antenna_cable_delay, 0.0);
        assert_eq!(delay.local_ref_delay, 0.0);

        let delay = SystemDelay::default().with_antenna_cable_delay(10.0);

        assert_eq!(delay.antenna_cable_delay, 10.0);
        assert_eq!(delay.local_ref_delay, 0.0);

        let delay = SystemDelay::default()
            .with_antenna_cable_delay(10.0)
            .with_ref_delay(20.0);

        assert_eq!(delay.antenna_cable_delay, 10.0);
        assert_eq!(delay.local_ref_delay, 20.0);
        assert_eq!(delay.total_cable_delay_nanos(), 30.0);

        assert_eq!(delay.antenna_cable_delay, 10.0);
        assert_eq!(delay.local_ref_delay, 20.0);

        assert!(delay
            .total_frequency_dependent_delay_nanos(&Code::C1)
            .is_none());

        for (k, v) in delay.frequency_dependent_nanos_delay_iter() {
            assert_eq!(*k, Code::C1);
            assert_eq!(v, 80.0);
        }

        assert!(delay
            .total_frequency_dependent_delay_nanos(&Code::P1)
            .is_none());
    }
}
