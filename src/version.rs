use crate::Error;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd, Eq, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Version {
    #[default]
    Version2E,
}

impl std::str::FromStr for Version {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq("2E") {
            Ok(Self::Version2E)
        } else {
            Err(Error::NonSupportedRevision(s.to_string()))
        }
    }
}
