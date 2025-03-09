/// Describes whether this common view is based on a unique
/// or a combination of SV
use crate::track::Error;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Default, Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum CommonViewClass {
    /// Single Channel
    #[default]
    SingleChannel,
    /// Multi Channel
    MultiChannel,
}

impl std::fmt::Display for CommonViewClass {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::SingleChannel => write!(f, "Single Channel"),
            Self::MultiChannel => write!(f, "Multi Channel"),
        }
    }
}

impl std::fmt::UpperHex for CommonViewClass {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CommonViewClass::MultiChannel => write!(fmt, "FF"),
            CommonViewClass::SingleChannel => write!(fmt, "99"),
        }
    }
}

impl std::str::FromStr for CommonViewClass {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq("FF") {
            Ok(Self::MultiChannel)
        } else if s.eq("99") {
            Ok(Self::SingleChannel)
        } else {
            Err(Error::UnknownClass)
        }
    }
}

#[cfg(test)]
mod test {
    use super::CommonViewClass;
    use std::str::FromStr;
    #[test]
    fn cv_class() {
        assert_eq!(format!("{:X}", CommonViewClass::MultiChannel), "FF");
        assert_eq!(format!("{:X}", CommonViewClass::SingleChannel), "99");
        assert_eq!(
            CommonViewClass::from_str("FF"),
            Ok(CommonViewClass::MultiChannel)
        );
        assert_eq!(
            CommonViewClass::from_str("FF"),
            Ok(CommonViewClass::MultiChannel)
        );
    }
}
