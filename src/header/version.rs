use crate::errors::ParsingError;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd, Eq, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Version {
    #[default]
    Version2E,
}

impl std::str::FromStr for Version {
    type Err = ParsingError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq("2E") {
            Ok(Self::Version2E)
        } else {
            Err(ParsingError::NonSupportedRevision)
        }
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Version2E => write!(f, "2E"),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::prelude::Version;
    use std::str::FromStr;

    #[test]
    fn version_parsing() {
        let version_2e = Version::from_str("2E").unwrap();
        assert_eq!(version_2e.to_string(), "2E");
    }
}
