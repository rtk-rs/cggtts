use crate::{buffer::Utf8Buffer, errors::FormattingError, prelude::Track};

use std::io::{BufWriter, Write};

use std::cmp::{max as cmp_max, min as cmp_min};

fn fmt_saturated<T: std::cmp::Ord + std::fmt::Display>(nb: T, sat: T, padding: usize) -> String {
    format!("{:>padding$}", std::cmp::min(nb, sat))
}

fn fmt_saturated_f64(nb: f64, scaling: f64, sat: i64, padding: usize) -> String {
    let scaled = (nb * scaling).round() as i64;
    if scaled.is_negative() {
        format!(
            "{:>padding$}",
            cmp_max(scaled, -sat / 10),
            padding = padding
        ) // remove 1 digit for sign
    } else {
        format!("{:>padding$}", cmp_min(scaled, sat), padding = padding)
    }
}

impl Track {
    pub(crate) fn format<W: Write>(
        &self,
        writer: &mut BufWriter<W>,
        buffer: &mut Utf8Buffer,
    ) -> Result<(), FormattingError> {
        // start by clearing buffer from past residues
        buffer.clear();

        buffer.push_str(&format!("{} {:X} ", self.sv, self.class));

        buffer.push_str(&format!(
            "{} ",
            fmt_saturated_f64(self.epoch.to_mjd_utc_days().floor(), 1.0, 99999, 4)
        ));

        let (_, _, _, h, m, s, _) = self.epoch.to_gregorian_utc();
        buffer.push_str(&format!("{:02}{:02}{:02} ", h, m, s));

        buffer.push_str(&format!(
            "{} ",
            fmt_saturated(self.duration.to_seconds() as u64, 9999, 4)
        ));

        buffer.push_str(&format!(
            "{} ",
            fmt_saturated_f64(self.elevation_deg, 10.0, 999, 3)
        ));

        buffer.push_str(&format!(
            "{} ",
            fmt_saturated_f64(self.azimuth_deg, 10.0, 9999, 4)
        ));

        buffer.push_str(&format!(
            "{} ",
            fmt_saturated_f64(self.data.refsv, 1E10, 99_999_999_999, 11)
        ));

        buffer.push_str(&format!(
            "{} ",
            fmt_saturated_f64(self.data.srsv, 1E13, 999_999, 6)
        ));

        buffer.push_str(&format!(
            "{} ",
            fmt_saturated_f64(self.data.refsys, 1E10, 99_999_999_999, 11)
        ));

        buffer.push_str(&format!(
            "{} ",
            fmt_saturated_f64(self.data.srsys, 1E13, 999_999, 6)
        ));

        buffer.push_str(&format!(
            "{} ",
            fmt_saturated_f64(self.data.dsg, 1E10, 9_999, 4)
        ));

        buffer.push_str(&format!("{} ", fmt_saturated(self.data.ioe, 999, 3)));

        buffer.push_str(&format!(
            "{} ",
            fmt_saturated_f64(self.data.mdtr, 1E10, 9_999, 4)
        ));

        buffer.push_str(&format!(
            "{} ",
            fmt_saturated_f64(self.data.smdt, 1E13, 9_999, 4)
        ));

        buffer.push_str(&format!(
            "{} ",
            fmt_saturated_f64(self.data.mdio, 1E10, 9_999, 4)
        ));

        buffer.push_str(&format!(
            "{} ",
            fmt_saturated_f64(self.data.smdi, 1E13, 9_999, 4)
        ));

        if let Some(iono) = self.iono {
            buffer.push_str(&format!(
                "{} {} {} ",
                fmt_saturated_f64(iono.msio, 1E10, 9_999, 4),
                fmt_saturated_f64(iono.smsi, 1E13, 999_999, 4),
                fmt_saturated_f64(iono.isg, 1E10, 9_999, 3),
            ));
        }

        if let Some(fdma) = &self.fdma_channel {
            buffer.push_str(&format!("{:2} ", fdma));
        } else {
            buffer.push_str(" 0 ");
        }

        buffer.push_str(&format!(
            "{:2} {:>frc_padding$} ",
            self.hc,
            self.frc,
            frc_padding = 3
        ));

        // ready to proceed to calculation
        let crc = buffer.calculate_crc();

        // append CRC
        buffer.push_str(&format!("{:02X}", crc));

        // interprate
        let utf8 = buffer.to_utf8_ascii()?; // we will never format bad Utf8

        // forward to user buffer
        write!(writer, "{}", &utf8)?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::buffer::Utf8Buffer;
    use crate::track::Track;
    use std::io::BufWriter;
    use std::str::FromStr;

    #[test]
    fn track_crc_formatting() {
        let mut buf = Utf8Buffer::new(1024);
        let mut user_buf = BufWriter::new(Utf8Buffer::new(1024));

        let track = Track::from_str(
            "E03 FF 60258 001000  780 139  548     +723788    +14        -302    -14    2 076  325  -36   32   -3   20  +20   3  0  0  E1 A5"
        )
            .unwrap();

        track.format(&mut user_buf, &mut buf).unwrap();

        let inner = user_buf.into_inner().unwrap_or_else(|_| panic!("oops"));
        let ascii_utf8 = inner.to_utf8_ascii().expect("generated invalid utf-8!");

        assert_eq!(
            ascii_utf8,
            "E03 FF 60258 001000  780 139  548      723788     14        -302    -14    2  76  325  -36   32   -3   20   20   3  0  0  E1 74",
        );

        // 3 letter modern Frequency modulation Code
        let mut buf = Utf8Buffer::new(1024);
        let mut user_buf = BufWriter::new(Utf8Buffer::new(1024));

        let track = Track::from_str(
            "E08 FF 60258 002600  780 142  988     1745615     40        -233    -19    4  79  321  -96   73  -14  116  -53  13  0  0 E5a 84"
            ).unwrap();

        track.format(&mut user_buf, &mut buf).unwrap();

        let inner = user_buf.into_inner().unwrap_or_else(|_| panic!("oops"));
        let ascii_utf8 = inner.to_utf8_ascii().expect("generated invalid utf-8!");

        assert_eq!(
            ascii_utf8,
            "E08 FF 60258 002600  780 142  988     1745615     40        -233    -19    4  79  321  -96   73  -14  116  -53  13  0  0 E5a 30"
        );

        // 3 letter modern Frequency modulation Code (bis)
        let mut buf = Utf8Buffer::new(1024);
        let mut user_buf = BufWriter::new(Utf8Buffer::new(1024));

        let track = Track::from_str(
            "E03 FF 60258 001000  780 139  548      724092     28           2      1    2  76  325  -36   54   -6   34   35   5  0  0 E5b 77"
        ).unwrap();

        track.format(&mut user_buf, &mut buf).unwrap();

        let inner = user_buf.into_inner().unwrap_or_else(|_| panic!("oops"));
        let ascii_utf8 = inner.to_utf8_ascii().expect("generated invalid utf-8!");

        assert_eq!(
            ascii_utf8,
            "E03 FF 60258 001000  780 139  548      724092     28           2      1    2  76  325  -36   54   -6   34   35   5  0  0 E5b 77"
        );
    }
}
