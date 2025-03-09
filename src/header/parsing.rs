use crate::{
    errors::ParsingError,
    header::{CalibrationID, Code, Coordinates, Delay, SystemDelay},
    prelude::{Epoch, Hardware, Header, ReferenceTime, Version},
};

use scan_fmt::scan_fmt;

use std::{
    io::{BufRead, BufReader, Read},
    str::FromStr,
};

fn parse_header_version(s: &str) -> Result<Version, ParsingError> {
    const MARKER: &str = "CGGTTS     GENERIC DATA FORMAT VERSION = ";
    const SIZE: usize = MARKER.len();

    if !s.starts_with(MARKER) {
        return Err(ParsingError::VersionFormat);
    };

    let content = s[SIZE..].trim();
    let version = Version::from_str(&content)?;
    Ok(version)
}

fn parse_header_date(s: &str) -> Result<Epoch, ParsingError> {
    const SIZE: usize = "REV DATE = ".len();

    let t = Epoch::from_format_str(s[SIZE..].trim(), "%Y-%m-%d")
        .or(Err(ParsingError::RevisionDateFormat))?;

    Ok(t)
}

fn parse_hardware(s: &str) -> Result<Hardware, ParsingError> {
    let mut hw = Hardware::default();

    for (i, item) in s.split_ascii_whitespace().enumerate() {
        if i == 0 {
            hw.manufacturer = item.trim().to_string();
        } else if i == 1 {
            hw.model = item.trim().to_string();
        } else if i == 2 {
            hw.serial_number = item.trim().to_string();
        } else if i == 3 {
            hw.year = item
                .trim()
                .parse::<u16>()
                .or(Err(ParsingError::InvalidFormat))?;
        } else if i == 4 {
            hw.release = item.trim().to_string();
        }
    }

    Ok(hw)
}

impl Header {
    /// Parse [Header] from any [Read]able input.
    pub fn parse<R: Read>(reader: &mut BufReader<R>) -> Result<Self, ParsingError> {
        const CKSUM_PATTERN: &str = "CKSUM = ";
        const CKSUM_LEN: usize = CKSUM_PATTERN.len();

        let mut lines_iter = reader.lines();

        // init variables
        let mut crc = 0u8;
        let mut system_delay = SystemDelay::default();

        let (mut blank, mut field_labels, mut unit_labels) = (false, false, false);

        let mut revision_date = Epoch::default();
        let mut nb_channels: u16 = 0;

        let mut receiver = Hardware::default();
        let mut ims_hardware: Option<Hardware> = None;

        let mut station = String::from("LAB");

        let mut comments: Option<String> = None;
        let mut reference_frame = String::with_capacity(16);
        let mut apc_coordinates = Coordinates::default();

        let mut reference_time = ReferenceTime::default();

        // VERSION must come first
        let first_line = lines_iter.next().ok_or(ParsingError::VersionFormat)?;
        let first_line = first_line.map_err(|_| ParsingError::VersionFormat)?;
        let version = parse_header_version(&first_line)?;

        // calculate first CRC contributions
        for byte in first_line.as_bytes().iter() {
            if *byte != b'\r' && *byte != b'\n' {
                crc = crc.wrapping_add(*byte);
            }
        }

        for line in lines_iter {
            if line.is_err() {
                continue;
            }

            let line = line.unwrap();
            let line_len = line.len();

            // CRC contribution
            let crc_max = if line.starts_with(CKSUM_PATTERN) {
                CKSUM_LEN
            } else {
                line_len
            };

            for byte in line.as_bytes()[..crc_max].iter() {
                if *byte != b'\r' && *byte != b'\n' {
                    crc = crc.wrapping_add(*byte);
                }
            }

            if line.starts_with("REV DATE = ") {
                revision_date = parse_header_date(&line)?;
            } else if line.starts_with("RCVR = ") {
                receiver = parse_hardware(&line[7..])?;
            } else if line.starts_with("IMS = ") {
                ims_hardware = Some(parse_hardware(&line[6..])?);
            } else if line.starts_with("CH = ") {
                nb_channels = line[5..]
                    .trim()
                    .parse::<u16>()
                    .or(Err(ParsingError::ChannelNumber))?;
            } else if line.starts_with("LAB = ") {
                station = line[5..].trim().to_string();
            } else if line.starts_with("X = ") {
                apc_coordinates.x = line[3..line_len - 1]
                    .trim()
                    .parse::<f64>()
                    .or(Err(ParsingError::Coordinates))?;
            } else if line.starts_with("Y = ") {
                apc_coordinates.y = line[3..line_len - 1]
                    .trim()
                    .parse::<f64>()
                    .or(Err(ParsingError::Coordinates))?;
            } else if line.starts_with("Z = ") {
                apc_coordinates.z = line[3..line_len - 1]
                    .trim()
                    .parse::<f64>()
                    .or(Err(ParsingError::Coordinates))?;
            } else if line.starts_with("FRAME = ") {
                reference_frame = line[8..].trim().to_string();
            } else if line.starts_with("COMMENTS = ") {
                let c = line.strip_prefix("COMMENTS =").unwrap().trim();
                if !c.eq("NO COMMENTS") {
                    comments = Some(c.to_string());
                }
            } else if line.starts_with("REF = ") {
                reference_time = line[5..].trim().parse::<ReferenceTime>()?;
            } else if line.contains("DLY = ") {
                let items: Vec<&str> = line.split_ascii_whitespace().collect();

                let dual_carrier = line.contains(',');

                if items.len() < 4 {
                    continue; // format mismatch
                }

                match items[0] {
                    "CAB" => {
                        system_delay.antenna_cable_delay = items[3]
                            .trim()
                            .parse::<f64>()
                            .or(Err(ParsingError::AntennaCableDelay))?;
                    },
                    "REF" => {
                        system_delay.local_ref_delay = items[3]
                            .trim()
                            .parse::<f64>()
                            .or(Err(ParsingError::LocalRefDelay))?;
                    },
                    "SYS" => {
                        if line.contains("CAL_ID") {
                            let offset = line.rfind('=').ok_or(ParsingError::CalibrationFormat)?;

                            if let Ok(cal_id) = CalibrationID::from_str(&line[offset + 1..]) {
                                system_delay = system_delay.with_calibration_id(cal_id);
                            }
                        }

                        if dual_carrier {
                            if let Ok(value) = f64::from_str(items[3]) {
                                let code = items[6].replace("),", "");
                                if let Ok(code) = Code::from_str(&code) {
                                    system_delay
                                        .freq_dependent_delays
                                        .push((code, Delay::System(value)));
                                }
                            }
                            if let Ok(value) = f64::from_str(items[7]) {
                                let code = items[9].replace(')', "");
                                if let Ok(code) = Code::from_str(&code) {
                                    system_delay
                                        .freq_dependent_delays
                                        .push((code, Delay::System(value)));
                                }
                            }
                        } else {
                            let value = f64::from_str(items[3]).unwrap();
                            let code = items[6].replace(')', "");
                            if let Ok(code) = Code::from_str(&code) {
                                system_delay
                                    .freq_dependent_delays
                                    .push((code, Delay::System(value)));
                            }
                        }
                    },
                    "INT" => {
                        if line.contains("CAL_ID") {
                            let offset = line.rfind('=').ok_or(ParsingError::CalibrationFormat)?;

                            if let Ok(cal_id) = CalibrationID::from_str(&line[offset + 1..]) {
                                system_delay = system_delay.with_calibration_id(cal_id);
                            }
                        }

                        if dual_carrier {
                            if let Ok(value) = f64::from_str(items[3]) {
                                let code = items[6].replace("),", "");
                                if let Ok(code) = Code::from_str(&code) {
                                    system_delay
                                        .freq_dependent_delays
                                        .push((code, Delay::Internal(value)));
                                }
                            }
                            if let Ok(value) = f64::from_str(items[7]) {
                                let code = items[10].replace(')', "");
                                if let Ok(code) = Code::from_str(&code) {
                                    system_delay
                                        .freq_dependent_delays
                                        .push((code, Delay::Internal(value)));
                                }
                            }
                        } else if let Ok(value) = f64::from_str(items[3]) {
                            let code = items[6].replace(')', "");
                            if let Ok(code) = Code::from_str(&code) {
                                system_delay
                                    .freq_dependent_delays
                                    .push((code, Delay::Internal(value)));
                            }
                        }
                    },
                    "TOT" => {
                        if line.contains("CAL_ID") {
                            let offset = line.rfind('=').ok_or(ParsingError::CalibrationFormat)?;

                            if let Ok(cal_id) = CalibrationID::from_str(&line[offset + 1..]) {
                                system_delay = system_delay.with_calibration_id(cal_id);
                            }
                        }

                        if dual_carrier {
                            if let Ok(value) = f64::from_str(items[3]) {
                                let code = items[6].replace("),", "");
                                if let Ok(code) = Code::from_str(&code) {
                                    system_delay
                                        .freq_dependent_delays
                                        .push((code, Delay::System(value)));
                                }
                            }
                            if let Ok(value) = f64::from_str(items[7]) {
                                let code = items[9].replace(')', "");
                                if let Ok(code) = Code::from_str(&code) {
                                    system_delay
                                        .freq_dependent_delays
                                        .push((code, Delay::System(value)));
                                }
                            }
                        } else if let Ok(value) = f64::from_str(items[3]) {
                            let code = items[6].replace(')', "");
                            if let Ok(code) = Code::from_str(&code) {
                                system_delay
                                    .freq_dependent_delays
                                    .push((code, Delay::System(value)));
                            }
                        }
                    },
                    _ => {}, // non recognized delay type
                };
            } else if line.starts_with("CKSUM = ") {
                // CRC verification
                let value = match scan_fmt!(&line, "CKSUM = {x}", String) {
                    Some(s) => match u8::from_str_radix(&s, 16) {
                        Ok(hex) => hex,
                        _ => return Err(ParsingError::ChecksumParsing),
                    },
                    _ => return Err(ParsingError::ChecksumFormat),
                };

                if value != crc {
                    return Err(ParsingError::ChecksumValue);
                }

                // CKSUM initiates the end of header section
                blank = true;
            } else if blank {
                // Field labels expected next
                blank = false;
                field_labels = true;
            } else if field_labels {
                // Unit labels expected next
                field_labels = false;
                unit_labels = true;
            } else if unit_labels {
                // last line that concludes this section
                break;
            }
        }

        Ok(Self {
            version,
            revision_date,
            nb_channels,
            receiver,
            ims_hardware,
            station,
            reference_frame,
            apc_coordinates,
            comments,
            delay: system_delay,
            reference_time,
        })
    }
}

#[cfg(test)]
mod test {
    use super::{parse_hardware, parse_header_date, parse_header_version};
    use crate::prelude::Version;
    use hifitime::Epoch;

    #[test]
    fn version_parsing() {
        for (content, version) in [(
            "CGGTTS     GENERIC DATA FORMAT VERSION = 2E",
            Version::Version2E,
        )] {
            let parsed = parse_header_version(content).unwrap();
            assert_eq!(parsed, version);
        }
    }

    #[test]
    fn date_parsing() {
        for (content, date) in [(
            "REV DATE = 2023-06-27",
            Epoch::from_gregorian_utc_at_midnight(2023, 06, 27),
        )] {
            let parsed = parse_header_date(content).unwrap();
            assert_eq!(parsed, date);
        }
    }

    #[test]
    fn hardware_parsing() {
        for (content, manufacturer, model, serial) in
            [("GTR51 2204005 1.12.0", "GTR51", "2204005", "1.12.0")]
        {
            let parsed = parse_hardware(content).unwrap();
            assert_eq!(parsed.manufacturer, manufacturer);
            assert_eq!(parsed.model, model);
            assert_eq!(parsed.serial_number, serial);
        }
    }
}
