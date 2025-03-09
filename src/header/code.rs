#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use strum_macros::EnumString;

#[derive(Clone, Copy, PartialEq, Debug, EnumString)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Default)]
pub enum Code {
    #[default]
    C1,
    C2,
    P1,
    P2,
    E1,
    E5,
    B1,
    B2,
}

impl std::fmt::Display for Code {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Code::C1 => fmt.write_str("C1"),
            Code::C2 => fmt.write_str("C2"),
            Code::P1 => fmt.write_str("P1"),
            Code::P2 => fmt.write_str("P2"),
            Code::E1 => fmt.write_str("E1"),
            Code::E5 => fmt.write_str("E5"),
            Code::B1 => fmt.write_str("B1"),
            Code::B2 => fmt.write_str("B2"),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_code() {
        assert_eq!(Code::default(), Code::C1);
        assert_eq!(Code::from_str("C2").unwrap(), Code::C2);
        assert_eq!(Code::from_str("P1").unwrap(), Code::P1);
        assert_eq!(Code::from_str("P2").unwrap(), Code::P2);
        assert_eq!(Code::from_str("E5").unwrap(), Code::E5);
    }
}
