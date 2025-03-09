use thiserror::Error;

#[derive(PartialEq, Debug, Error)]
pub enum Error {
    #[error("can only calculate over valid utf8 data")]
    NonUtf8Data,
    #[error("checksum error, got \"{0}\" but \"{1}\" locally computed")]
    ChecksumError(u8, u8),
}

/// computes crc for given str content
pub(crate) fn calc_crc(content: &str) -> Result<u8, Error> {
    match content.is_ascii() {
        true => {
            let mut ck: u8 = 0;
            let mut ptr = content.encode_utf16();
            for _ in 0..ptr.clone().count() {
                ck = ck.wrapping_add(ptr.next().unwrap() as u8)
            }
            Ok(ck)
        },
        false => Err(Error::NonUtf8Data),
    }
}

#[cfg(test)]
mod test {
    use crate::crc::calc_crc;
    #[test]
    fn test_crc() {
        let content = ["R24 FF 57000 000600  780 347 394 +1186342 +0 163 +0 40 2 141 +22 23 -1 23 -1 29 +2 0 L3P",
            "G99 99 59509 002200 0780 099 0099 +9999999999 +99999 +9999989831   -724    35 999 9999 +999 9999 +999 00 00 L1C"];
        let expected = [0x0F, 0x71];
        for i in 0..content.len() {
            let ck = calc_crc(content[i]).unwrap();
            let expect = expected[i];
            assert_eq!(
                ck, expect,
                "Failed for \"{}\", expect \"{}\" but \"{}\" locally computed",
                content[i], expect, ck
            )
        }
    }
}
