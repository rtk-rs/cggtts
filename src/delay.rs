use crate::Code;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Different types of delay known,
/// refer to documentation to truly understand what they
/// represent. <!> Delays are always specified in nanoseconds <!>
#[derive(Debug, PartialEq, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Delay {
    /// Delay defined as `internal`
    Internal(f64),
    /// `System` delay
    System(f64),
}

impl Default for Delay {
    fn default() -> Delay {
        Delay::System(0.0_f64)
    }
}

impl Delay {
    /// Returns (`unwraps`) delay value in nanoseconds
    pub fn value(&self) -> f64 {
        match self {
            Delay::Internal(d) => *d,
            Delay::System(d) => *d,
        }
    }
    /// Returns (`unwraps`) itself in seconds
    pub fn value_seconds(&self) -> f64 {
        self.value() * 1.0E-9
    }

    /// Adds value to self
    pub fn add_value(&self, rhs: f64) -> Self {
        match self {
            Delay::System(d) => Delay::System(*d + rhs),
            Delay::Internal(d) => Delay::Internal(*d + rhs),
        }
    }
}

/*
impl std::fmt::Display for CalibratedDelay {
fn fmt (&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
    if self.values.len() == 1 {
        fmt.write_str(&format!("{:.1} ns ({} {})", self.values[0] * 1E9, self.constellation, self.channels[0]))?
    } else {
        // CSV
        for i in 0..self.values.len()-1 {
            fmt.write_str(&format!("{:.1} ns ({} {}), ",
                self.values[i] *1E9, self.constellation, self.channels[i]))?
        }
        fmt.write_str(&format!("{:.1} ns ({} {})",
            self.values[self.values.len()-1] *1E9, self.constellation,
                self.channels[self.values.len()-1]))?
    }
    fmt.write_str(&format!("     CAL_ID = {}", self.report))?;
    Ok(())
}
}
*/

/*
/// Identifies carrier dependant informations
/// from a string shaped like '53.9 ns (GLO C1)'
fn carrier_dependant_delay_parsing (string: &str)
        -> Result<(f64,track::Constellation,String),Error>
{
    let (delay, const_str, channel) : (f64, String, String) = match scan_fmt!(string, "{f} ns ({} {})", f64, String, String) {
        (Some(delay),Some(constellation),Some(channel)) => (delay,constellation,channel),
        _ => return Err(Error::FrequencyDependentDelayParsingError(String::from(string)))
    };
    let mut constellation: track::Constellation = track::Constellation::default();
    if const_str.eq("GPS") {
        constellation = track::Constellation::GPS
    } else if const_str.eq("GLO") {
        constellation = track::Constellation::Glonass
    } else if const_str.eq("BDS") {
        constellation = track::Constellation::Beidou
    } else if const_str.eq("GAL") {
        constellation = track::Constellation::Galileo
    } else if const_str.eq("QZS") {
        constellation = track::Constellation::QZSS
    }
    Ok((delay,constellation,channel))
}
*/

/// System Delay describe the total measurement systems delay
/// to be used in `Cggtts`
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SystemDelay {
    /// RF/cable delay
    pub rf_cable_delay: f64,
    /// reference delay
    pub ref_delay: f64,
    /// carrier dependend delays
    pub delays: Vec<(Code, Delay)>,
    /// Calibration ID
    pub cal_id: Option<String>,
}

impl Default for SystemDelay {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemDelay {
    /// Builds a new system delay description,
    /// with empty fields. Use `add_delay()` to customize.
    pub fn new() -> Self {
        Self {
            rf_cable_delay: 0.0_f64,
            ref_delay: 0.0_f64,
            delays: Vec::new(),
            cal_id: None,
        }
    }
    /// Returns Self with additionnal calibration info
    pub fn with_calibration_id(&self, info: &str) -> Self {
        Self {
            rf_cable_delay: self.rf_cable_delay,
            ref_delay: self.ref_delay,
            delays: self.delays.clone(),
            cal_id: Some(info.to_string()),
        }
    }
    /// Returns total system delay for given carrier code
    pub fn total_delay(&self, code: Code) -> Option<f64> {
        for (k, v) in self.delays.iter() {
            if *k == code {
                return Some(v.value() + self.rf_cable_delay + self.ref_delay);
            }
        }
        None
    }
    /// Groups total system delay per carrier codes
    pub fn total_delays(&self) -> Vec<(Code, f64)> {
        let mut res: Vec<(Code, f64)> = Vec::new();
        for (k, v) in self.delays.iter() {
            res.push((*k, v.value() + self.rf_cable_delay + self.ref_delay))
        }
        res
    }
}

#[cfg(test)]
mod delay {
    use super::*;
    #[test]
    fn test_delay() {
        let delay = Delay::Internal(10.0);
        assert_eq!(delay.value(), 10.0);
        assert_eq!(delay.value_seconds(), 10.0E-9);
        assert!(delay == Delay::Internal(10.0));
        assert!(delay != Delay::System(10.0));
        let d = delay.add_value(20.0);
        assert_eq!(d, Delay::Internal(30.0));
        assert_eq!(delay.value() + 20.0, d.value());
        assert_eq!(Delay::default(), Delay::System(0.0));
        let delay = Delay::System(30.5);
        assert_eq!(delay.value(), 30.5);
        let d = delay.add_value(20.0);
        assert_eq!(d.value(), 50.5);
    }

    #[test]
    fn test_system_delay() {
        let mut delay = SystemDelay::new();
        assert_eq!(delay.rf_cable_delay, 0.0);
        delay.rf_cable_delay = 10.0;
        delay.ref_delay = 20.0;
        delay.delays.push((Code::C1, Delay::Internal(50.0)));
        assert_eq!(delay.rf_cable_delay, 10.0);
        assert_eq!(delay.ref_delay, 20.0);
        let total = delay.total_delay(Code::C1);
        assert!(total.is_some());
        assert_eq!(total.unwrap(), 80.0);
        let totals = delay.total_delays();
        assert!(!totals.is_empty());
        assert_eq!(totals[0].0, Code::C1);
        assert_eq!(totals[0].1, 80.0);
        assert!(delay.total_delay(Code::P1).is_none());
    }
}
