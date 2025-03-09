use crate::errors::ParsingError;
use hifitime::TimeScale;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Reference Time System
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ReferenceTime {
    /// TAI: Temps Atomic International
    TAI,
    /// UTC: Universal Coordinate Time
    UTC,
    /// UTC(k) laboratory local copy
    UTCk(String),
    /// Custom Reference time system
    Custom(String),
}

impl Default for ReferenceTime {
    fn default() -> Self {
        Self::UTC
    }
}

impl std::str::FromStr for ReferenceTime {
    type Err = ParsingError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq("TAI") {
            Ok(Self::TAI)
        } else if s.eq("UTC") {
            Ok(Self::UTC)
        } else if s.starts_with("UTC(") && s.ends_with(')') {
            let len = s.len();
            let utc_k = &s[4..len - 1];
            Ok(Self::UTCk(utc_k.to_string()))
        } else {
            Ok(Self::Custom(s.to_string()))
        }
    }
}

impl From<TimeScale> for ReferenceTime {
    fn from(ts: TimeScale) -> Self {
        match ts {
            TimeScale::UTC => Self::UTC,
            TimeScale::TAI => Self::TAI,
            _ => Self::TAI, /* incorrect usage */
        }
    }
}

impl std::fmt::Display for ReferenceTime {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::TAI => fmt.write_str("TAI"),
            Self::UTC => fmt.write_str("UTC"),
            Self::UTCk(lab) => write!(fmt, "UTC({})", lab),
            Self::Custom(s) => fmt.write_str(s),
        }
    }
}

#[cfg(test)]
mod test {
    use super::ReferenceTime;
    use std::str::FromStr;
    #[test]
    fn from_str() {
        assert_eq!(ReferenceTime::default(), ReferenceTime::UTC);
        assert_eq!(ReferenceTime::from_str("TAI").unwrap(), ReferenceTime::TAI);
        assert_eq!(ReferenceTime::from_str("UTC").unwrap(), ReferenceTime::UTC);
        assert_eq!(
            ReferenceTime::from_str("UTC(LAB-X)").unwrap(),
            ReferenceTime::UTCk(String::from("LAB-X"))
        );
    }
}
