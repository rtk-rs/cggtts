use std::str::{from_utf8, Utf8Error};

pub struct Utf8Buffer {
    inner: Vec<u8>,
}

#[cfg(test)]
use std::io::Write;

/// [Write] implementation is only used when testing.
#[cfg(test)]
impl Write for Utf8Buffer {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        for b in buf {
            self.inner.push(*b);
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.clear();
        Ok(())
    }
}

impl Utf8Buffer {
    /// Allocated new [Utf8Buffer].
    pub fn new(size: usize) -> Self {
        Self {
            inner: Vec::with_capacity(size),
        }
    }

    /// Clear (discard) internal content
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// Pushes "content" (valid Utf-8) into internal buffer.
    pub fn push_str(&mut self, content: &str) {
        let bytes = content.as_bytes();
        self.inner.extend_from_slice(&bytes);
    }

    pub fn calculate_crc(&self) -> u8 {
        let mut crc = 0u8;
        for byte in self.inner.iter() {
            if *byte != b'\n' && *byte != b'\r' {
                crc = crc.wrapping_add(*byte);
            }
        }
        crc
    }

    pub fn to_utf8_ascii<'a>(&'a self) -> Result<&'a str, Utf8Error> {
        from_utf8(&self.inner[..])
    }
}

#[cfg(test)]
mod test {
    use super::Utf8Buffer;

    #[test]
    fn test_crc_tracks_buffering() {
        let mut buf = Utf8Buffer::new(1024);

        buf.push_str("R24 FF 57000 000600  780 347 394 +1186342 +0 163 +0 40 2 141 +22 23 -1 23 -1 29 +2 0 L3P");
        assert_eq!(buf.calculate_crc(), 0x0f);

        buf.clear();

        buf.push_str("G99 99 59509 002200 0780 099 0099 +9999999999 +99999 +9999989831   -724    35 999 9999 +999 9999 +999 00 00 L1C");
        assert_eq!(buf.calculate_crc(), 0x71);
    }

    #[test]
    fn test_crc_header_buffering() {
        let mut buf = Utf8Buffer::new(1024);

        let content = "CGGTTS     GENERIC DATA FORMAT VERSION = 2E
REV DATE = 2023-06-27
RCVR = GTR51 2204005 1.12.0
CH = 20
IMS = GTR51 2204005 1.12.0
LAB = LAB
X = +3970727.80 m
Y = +1018888.02 m
Z = +4870276.84 m
FRAME = FRAME
COMMENTS = NO COMMENTS
INT DLY =   34.6 ns (GAL E1),   0.0 ns (GAL E5),   0.0 ns (GAL E6),   0.0 ns (GAL E5b),  25.6 ns (GAL E5a)     CAL_ID = 1015-2021
CAB DLY =  155.2 ns
REF DLY =    0.0 ns
REF = REF_IN
CKSUM = ";

        buf.push_str(&content);
        assert_eq!(buf.calculate_crc(), 0xD7);

        let mut buf = Utf8Buffer::new(1024);

        let content = "CGGTTS     GENERIC DATA FORMAT VERSION = 2E
REV DATE = 2023-06-27
RCVR = GTR51 2204005 1.12.0
CH = 20
IMS = GTR51 2204005 1.12.0
LAB = LAB
X = +3970727.80 m
Y = +1018888.02 m
Z = +4870276.84 m
FRAME = FRAME
COMMENTS = NO COMMENTS
INT DLY =   32.9 ns (GPS C1),  32.9 ns (GPS P1),   0.0 ns (GPS C2),  25.8 ns (GPS P2),   0.0 ns (GPS L5),   0.0 ns (GPS L1C)     CAL_ID = 1015-2021
CAB DLY =  155.2 ns
REF DLY =    0.0 ns
REF = REF_IN
CKSUM = ";

        buf.push_str(&content);
        assert_eq!(buf.calculate_crc(), 0x07);
    }
}
