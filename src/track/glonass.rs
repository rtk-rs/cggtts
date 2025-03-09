#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::track::Error;

/// Describes Glonass Frequency channel,
/// in case this `Track` was estimated using Glonass
#[derive(Debug, Default, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum GlonassChannel {
    /// Default value when not using Glonass constellation
    #[default]
    Unknown,
    /// Glonass Frequency channel number
    ChanNum(u8),
}

impl std::fmt::Display for GlonassChannel {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            GlonassChannel::Unknown => write!(fmt, "00"),
            GlonassChannel::ChanNum(c) => write!(fmt, "{:02X}", c),
        }
    }
}

impl std::str::FromStr for GlonassChannel {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let ch = s
            .trim()
            .parse::<u8>()
            .map_err(|_| Error::FieldParsing(String::from("FR")))?;
        if ch == 0 {
            Ok(Self::Unknown)
        } else {
            Ok(Self::ChanNum(ch))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::track::GlonassChannel;
    #[test]
    fn glonass_chx() {
        for (value, expected) in [
            (GlonassChannel::Unknown, "00"),
            (GlonassChannel::ChanNum(1), "01"),
            (GlonassChannel::ChanNum(9), "09"),
            (GlonassChannel::ChanNum(10), "0A"),
            (GlonassChannel::ChanNum(11), "0B"),
        ] {
            assert_eq!(value.to_string(), expected);
        }
    }
}
