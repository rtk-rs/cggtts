use hifitime::TimeScale;
use scan_fmt::scan_fmt;

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

impl ReferenceTime {
    pub fn from_str(s: &str) -> Self {
        let lower = s.to_lowercase();
        if lower.eq("tai") {
            Self::TAI
        } else if lower.eq("utc") {
            Self::UTC
        } else if let Some(lab) = scan_fmt!(s, "UTC({})", String) {
            Self::UTCk(lab.trim().to_string())
        } else {
            Self::Custom(s.to_string())
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
    #[test]
    fn from_str() {
        assert_eq!(ReferenceTime::default(), ReferenceTime::UTC);
        assert_eq!(ReferenceTime::from_str("TAI"), ReferenceTime::TAI);
        assert_eq!(ReferenceTime::from_str("UTC"), ReferenceTime::UTC);
        assert_eq!(
            ReferenceTime::from_str("UTC(LAB )"),
            ReferenceTime::UTCk(String::from("LAB"))
        );
    }
}
