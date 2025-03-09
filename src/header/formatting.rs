use crate::{
    buffer::Utf8Buffer,
    errors::FormattingError,
    prelude::{Header, Version},
};

use std::io::{BufWriter, Write};

impl Header {
    /// Formats this [CGGTTS] following standard specifications.
    pub fn format<W: Write>(
        &self,
        writer: &mut BufWriter<W>,
        buf: &mut Utf8Buffer,
    ) -> Result<(), FormattingError> {
        // clear potential past residues
        buf.clear();

        buf.push_str(&format!(
            "CGGTTS GENERIC DATA FORMAT VERSION = {}\n",
            Version::Version2E,
        ));

        let (y, m, d, _, _, _, _) = self.revision_date.to_gregorian_utc();
        buf.push_str(&format!("REV DATE = {:04}-{:02}-{:02}\n", y, m, d));
        buf.push_str(&format!("RCVR = {:x}\n", &self.receiver));
        buf.push_str(&format!("CH = {}\n", self.nb_channels));

        if let Some(ims) = &self.ims_hardware {
            buf.push_str(&format!("IMS = {:x}\n", ims));
        }

        buf.push_str(&format!("LAB = {}\n", self.station));

        buf.push_str(&format!("X = {:12.3} m\n", self.apc_coordinates.x));
        buf.push_str(&format!("Y = {:12.3} m\n", self.apc_coordinates.y));
        buf.push_str(&format!("Z = {:12.3} m\n", self.apc_coordinates.z));
        buf.push_str(&format!("FRAME = {}\n", self.reference_frame));

        if let Some(comments) = &self.comments {
            buf.push_str(&format!("COMMENTS = {}\n", comments.trim()));
        } else {
            buf.push_str(&format!("COMMENTS = NO COMMENTS\n"));
        }

        // TODO system delay formatting
        // let delays = self.delay.delays.clone();
        // let constellation = if !self.tracks.is_empty() {
        //     self.tracks[0].sv.constellation
        // } else {
        //     Constellation::default()
        // };

        // if delays.len() == 1 {
        //     // Single frequency
        //     let (code, value) = delays[0];
        //     match value {
        //         Delay::Internal(v) => {
        //             content.push_str(&format!(
        //                 "INT DLY = {:.1} ns ({:X} {})\n",
        //                 v, constellation, code
        //             ));
        //         },
        //         Delay::System(v) => {
        //             content.push_str(&format!(
        //                 "SYS DLY = {:.1} ns ({:X} {})\n",
        //                 v, constellation, code
        //             ));
        //         },
        //     }
        //     if let Some(cal_id) = &self.delay.cal_id {
        //         content.push_str(&format!("       CAL_ID = {}\n", cal_id));
        //     } else {
        //         content.push_str("       CAL_ID = NA\n");
        //     }
        // } else if delays.len() == 2 {
        //     // Dual frequency
        //     let (c1, v1) = delays[0];
        //     let (c2, v2) = delays[1];
        //     match v1 {
        //         Delay::Internal(_) => {
        //             content.push_str(&format!(
        //                 "INT DLY = {:.1} ns ({:X} {}), {:.1} ns ({:X} {})\n",
        //                 v1.value(),
        //                 constellation,
        //                 c1,
        //                 v2.value(),
        //                 constellation,
        //                 c2
        //             ));
        //         },
        //         Delay::System(_) => {
        //             content.push_str(&format!(
        //                 "SYS DLY = {:.1} ns ({:X} {}), {:.1} ns ({:X} {})\n",
        //                 v1.value(),
        //                 constellation,
        //                 c1,
        //                 v2.value(),
        //                 constellation,
        //                 c2
        //             ));
        //         },
        //     }
        //     if let Some(cal_id) = &self.delay.cal_id {
        //         content.push_str(&format!("     CAL_ID = {}\n", cal_id));
        //     } else {
        //         content.push_str("     CAL_ID = NA\n");
        //     }
        // }

        buf.push_str(&format!(
            "CAB DLY = {:05.1} ns\n",
            self.delay.antenna_cable_delay,
        ));

        buf.push_str(&format!(
            "REF DLY = {:05.1} ns\n",
            self.delay.local_ref_delay
        ));

        buf.push_str(&format!("REF = {}\n", self.reference_time));

        // push last bytes contributing to CRC
        buf.push_str("CKSUM = ");

        // Run CK calculation
        let ck = buf.calculate_crc();

        // Append CKSUM
        buf.push_str(&format!("{:02X}\n", ck));

        // interprate
        let ascii_utf8 = buf.to_utf8_ascii()?;

        // forward to user
        write!(writer, "{}", ascii_utf8)?;

        Ok(())
    }
}

#[cfg(test)]
mod test {

    use std::io::BufWriter;
    use std::path::Path;

    use crate::{buffer::Utf8Buffer, CGGTTS};

    #[test]
    fn header_crc_buffering() {
        let mut utf8 = Utf8Buffer::new(1024);
        let mut buf = BufWriter::new(Utf8Buffer::new(1024));

        // This file does not have comments
        // run once
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("data/CGGTTS")
            .join("EZGTR60.258");

        let cggtts = CGGTTS::from_file(path).unwrap();

        let header = &cggtts.header;

        // TODO: unlock, problem @ RCVR parsing
        //let rcvr = header.receiver.as_ref().expect("missing RCVR");
        //assert_eq!(rcvr.model, "GTR51");
        //assert_eq!(rcvr.serial_number, "2204005");

        // TODO: unlock, problem @ RCVR parsing
        // let ims = header.ims_hardware.as_ref().expect("missing IMS");
        // assert_eq!(rcvr.model, "GTR51");
        // assert_eq!(rcvr.serial_number, "2204005");

        header.format(&mut buf, &mut utf8).unwrap();

        let inner = buf.into_inner().unwrap_or_else(|_| panic!("oops"));
        let ascii_utf8 = inner.to_utf8_ascii().expect("generated invalid utf-8!");

        // TODO: missing
        // RCVR = GTR51 2204005 1.12.0
        // IMS = GTR51 2204005 1.12.0

        // TODO: missing
        // INT DLY =   34.6 ns (GAL E1),   0.0 ns (GAL E5),   0.0 ns (GAL E6),   0.0 ns (GAL E5b),  25.6 ns (GAL E5a)     CAL_ID = 1015-2021
        let expected = "CGGTTS GENERIC DATA FORMAT VERSION = 2E
REV DATE = 2014-02-20
CH = 20
LAB = LAB
X =  3970727.800 m
Y =  1018888.020 m
Z =  4870276.840 m
FRAME = FRAME
COMMENTS = NO COMMENTS
CAB DLY = 155.2 ns
REF DLY = 000.0 ns
REF = REF_IN
CKSUM = A8";

        for (content, expected) in ascii_utf8.lines().zip(expected.lines()) {
            assert_eq!(content, expected);
        }
    }
}
