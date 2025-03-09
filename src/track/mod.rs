use crate::crc::calc_crc;
use thiserror::Error; //Error as CrcError};

mod glonass;
pub use glonass::GlonassChannel;

mod class;
pub use class::CommonViewClass;

#[cfg(feature = "scheduler")]
#[cfg_attr(docsrs, doc(cfg(feature = "scheduler")))]
mod scheduler;

#[cfg(feature = "scheduler")]
#[cfg_attr(docsrs, doc(cfg(feature = "scheduler")))]
pub use scheduler::{FitData, SVTracker, Scheduler};

use gnss::prelude::{Constellation, SV};
use hifitime::{Duration, Epoch, Unit};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

const TRACK_WITH_IONOSPHERIC: usize = 24;
const TRACK_WITHOUT_IONOSPHERIC: usize = 21;

/// A Track is a CGGTTS measurement
#[derive(Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Track {
    /// Common View Class (Single/Multi channel)
    pub class: CommonViewClass,
    /// Epoch of this track
    pub epoch: Epoch,
    /// Tracking duration
    pub duration: Duration,
    /// SV tracked during this realization
    pub sv: SV,
    /// Elevation at track midpoint, expressed in degrees
    pub elevation: f64,
    /// Azimuth at track midpoint, expressed in degrees
    pub azimuth: f64,
    /// Track data
    pub data: TrackData,
    /// Optionnal Ionospheric compensation terms
    pub iono: Option<IonosphericData>,
    /// Glonass Channel Frequency [1:24], O for other GNSS
    pub fr: GlonassChannel,
    /// Hardware / receiver channel [0:99], 0 if Unknown
    pub hc: u8,
    /// Carrier frequency standard 3 letter code,
    /// refer to RINEX specifications for meaning
    pub frc: String,
}

#[derive(Error, Debug, PartialEq)]
pub enum Error {
    #[error("invalid track format")]
    InvalidFormat,
    #[error("invalid sttime field format")]
    InvalidTrkTimeFormat,
    #[error("unknown common view class")]
    UnknownClass,
    #[error("failed to parse sv")]
    SVParsing(#[from] gnss::sv::ParsingError),
    #[error("failed to parse \"{0}\" field")]
    FieldParsing(String),
    #[error("missing \"{0}\" field")]
    MissingField(String),
    #[error("checksum error")]
    CrcError(#[from] crate::crc::Error),
}

/// Track data
#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TrackData {
    /// REFSV
    pub refsv: f64,
    /// SRSV
    pub srsv: f64,
    /// REFSYS
    pub refsys: f64,
    /// SRSYS
    pub srsys: f64,
    /// Data signma (`DSG`) : RMS residuals to linear fit
    pub dsg: f64,
    /// Issue of Ephemeris (`IOE`),
    /// Three-digit decimal code indicating the ephemeris used for the computation.
    /// As no IOE is associated with the GLONASS navigation messages, the values 1-96 have to be
    /// used to indicate the date of the ephemeris used, given by the number of the quarter of an hour in
    /// the day, starting at 1=00h00m00s.
    /// For BeiDou, IOE will report the integer hour in the date of the ephemeris (Time of Clock).
    pub ioe: u16,
    /// Modeled tropospheric delay
    pub mdtr: f64,
    /// Slope of the modeled tropospheric delay
    pub smdt: f64,
    /// Modeled ionospheric delay
    pub mdio: f64,
    /// Slope of the modeled ionospheric delay
    pub smdi: f64,
}

/// Ionospheric Data are attached to a CGGTTS track
/// when generated in dual frequency contexts.
#[derive(Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct IonosphericData {
    /// Measured ionospheric delay
    /// corresponding to the solution E in section 2.3.3.
    pub msio: f64,
    /// Slope of the measured ionospheric delay
    /// corresponding to the solution E in section 2.3.3.
    pub smsi: f64,
    /// Root-mean-square of the residuals
    /// corresponding to the solution E in section2.3.3
    pub isg: f64,
}

impl Track {
    /// Builds a new CGGTTS track. To follow CGGTTS guidelines,
    /// it is important to use an Epoch expressed in UTC here.
    /// For Glonass SV: prefer [Self::new_glonass_sv]
    pub fn new(
        sv: SV,
        utc_epoch: Epoch,
        duration: Duration,
        class: CommonViewClass,
        elevation: f64,
        azimuth: f64,
        data: TrackData,
        iono: Option<IonosphericData>,
        rcvr_channel: u8,
        frc: &str,
    ) -> Self {
        Self {
            sv,
            epoch: utc_epoch,
            class,
            duration,
            elevation,
            azimuth,
            data,
            iono,
            fr: GlonassChannel::Unknown,
            hc: rcvr_channel,
            frc: frc.to_string(),
        }
    }
    /// Builds new CGGTTS track from single Glonass SV realization.
    /// Epoch should be expressed in UTC.
    pub fn new_glonass(
        sv: SV,
        utc_epoch: Epoch,
        duration: Duration,
        class: CommonViewClass,
        elevation: f64,
        azimuth: f64,
        data: TrackData,
        iono: Option<IonosphericData>,
        rcvr_channel: u8,
        glo_channel: GlonassChannel,
        frc: &str,
    ) -> Self {
        Self {
            sv,
            epoch: utc_epoch,
            duration,
            class,
            elevation,
            azimuth,
            data,
            iono,
            fr: glo_channel,
            hc: rcvr_channel,
            frc: frc.to_string(),
        }
    }
    /// Returns true if Self was measured against given `GNSS` Constellation
    pub fn uses_constellation(&self, c: Constellation) -> bool {
        self.sv.constellation == c
    }
    /// Returns True if Self follows BIPM specifications / requirements,
    /// in terms of tracking pursuit
    pub fn follows_bipm_specs(&self) -> bool {
        self.duration == Duration::from_seconds(780.0)
    }
    /// Returns a `Track` with desired unique space vehicule
    pub fn with_sv(&self, sv: SV) -> Self {
        let mut t = self.clone();
        t.sv = sv;
        t
    }
    /// Returns a track with desired elevation angle in Degrees
    pub fn with_elevation(&self, elevation: f64) -> Self {
        let mut t = self.clone();
        t.elevation = elevation;
        t
    }
    /// Returns a `Track` with given azimuth angle in Degrees, at tracking midpoint
    pub fn with_azimuth(&self, azimuth: f64) -> Self {
        let mut t = self.clone();
        t.azimuth = azimuth;
        t
    }
    /// Returns a `Track` with desired Frequency carrier code
    pub fn with_carrier_code(&self, code: &str) -> Self {
        let mut t = self.clone();
        t.frc = code.to_string();
        t
    }
    /// Returns true if Self comes with Ionospheric parameter estimates
    pub fn has_ionospheric_data(&self) -> bool {
        self.iono.is_some()
    }
}

fn cggtts_fmt<T: std::cmp::Ord + std::fmt::Display>(nb: T, sat: T, padding: usize) -> String {
    format!("{:>padding$}", std::cmp::min(nb, sat))
}

fn cggtts_fmt_f64(nb: f64, scaling: f64, sat: i64, padding: usize) -> String {
    let scaled = (nb * scaling).round() as i64;
    if scaled.is_negative() {
        format!("{:>padding$}", std::cmp::max(scaled, -sat / 10)) // remove 1 digit for sign
    } else {
        format!("{:>padding$}", std::cmp::min(scaled, sat))
    }
}

impl std::fmt::Display for Track {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut string = String::new();
        string.push_str(&format!("{} {:X} ", self.sv, self.class));

        string.push_str(&format!(
            "{} ",
            cggtts_fmt_f64(self.epoch.to_mjd_utc_days().floor(), 1.0, 99999, 5)
        ));

        let (_, _, _, h, m, s, _) = self.epoch.to_gregorian_utc();
        string.push_str(&format!("{:02}{:02}{:02} ", h, m, s));

        string.push_str(&format!(
            "{} ",
            cggtts_fmt(self.duration.to_seconds() as u64, 9999, 4)
        ));

        string.push_str(&format!(
            "{} ",
            cggtts_fmt_f64(self.elevation, 10.0, 999, 3)
        ));

        string.push_str(&format!("{} ", cggtts_fmt_f64(self.azimuth, 10.0, 9999, 4)));

        string.push_str(&format!(
            "{} ",
            cggtts_fmt_f64(self.data.refsv, 1E10, 99_999_999_999, 11)
        ));

        string.push_str(&format!(
            "{} ",
            cggtts_fmt_f64(self.data.srsv, 1E13, 999_999, 6)
        ));

        string.push_str(&format!(
            "{} ",
            cggtts_fmt_f64(self.data.refsys, 1E10, 99_999_999_999, 11)
        ));

        string.push_str(&format!(
            "{} ",
            cggtts_fmt_f64(self.data.srsys, 1E13, 999_999, 6)
        ));

        string.push_str(&format!(
            "{} ",
            cggtts_fmt_f64(self.data.dsg, 1E10, 9_999, 4)
        ));

        string.push_str(&format!("{} ", cggtts_fmt(self.data.ioe, 999, 3)));

        string.push_str(&format!(
            "{} ",
            cggtts_fmt_f64(self.data.mdtr, 1E10, 9_999, 4)
        ));

        string.push_str(&format!(
            "{} ",
            cggtts_fmt_f64(self.data.smdt, 1E13, 9_999, 4)
        ));

        string.push_str(&format!(
            "{} ",
            cggtts_fmt_f64(self.data.mdio, 1E10, 9_999, 4)
        ));

        string.push_str(&format!(
            "{} ",
            cggtts_fmt_f64(self.data.smdi, 1E13, 9_999, 4)
        ));

        if let Some(iono) = self.iono {
            string.push_str(&format!(
                "{} {} {} ",
                cggtts_fmt_f64(iono.msio, 1E10, 9_999, 4),
                cggtts_fmt_f64(iono.smsi, 1E13, 999_999, 6),
                cggtts_fmt_f64(iono.isg, 1E10, 9_999, 4),
            ));
        }

        string.push_str(&format!("{:02} {:02} {}", self.fr, self.hc, self.frc));

        let crc = calc_crc(&string);
        if crc.is_err() {
            panic!("failed to calculate line CRC: {}", crc.err().unwrap());
        }

        let crc = crc.unwrap();
        string.push_str(&format!(" {:02X}", crc)); //crc.wrapping_add(32)));

        fmt.write_str(&string)
    }
}

fn parse_data(items: &mut std::str::SplitAsciiWhitespace<'_>) -> Result<TrackData, Error> {
    let refsv = items
        .next()
        .ok_or(Error::MissingField(String::from("REFSV")))?
        .parse::<f64>()
        .map_err(|_| Error::FieldParsing(String::from("REFSV")))?
        * 1E-10;

    let srsv = items
        .next()
        .ok_or(Error::MissingField(String::from("SRSV")))?
        .parse::<f64>()
        .map_err(|_| Error::FieldParsing(String::from("SRSV")))?
        * 1E-13;

    let refsys = items
        .next()
        .ok_or(Error::MissingField(String::from("REFSYS")))?
        .parse::<f64>()
        .map_err(|_| Error::FieldParsing(String::from("REFSYS")))?
        * 1E-10;

    let srsys = items
        .next()
        .ok_or(Error::MissingField(String::from("SRSYS")))?
        .parse::<f64>()
        .map_err(|_| Error::FieldParsing(String::from("SRSYS")))?
        * 1E-13;

    let dsg = items
        .next()
        .ok_or(Error::MissingField(String::from("DSG")))?
        .parse::<f64>()
        .map_err(|_| Error::FieldParsing(String::from("DSG")))?
        * 1E-10;

    let ioe = items
        .next()
        .ok_or(Error::MissingField(String::from("IOE")))?
        .parse::<u16>()
        .map_err(|_| Error::FieldParsing(String::from("IOE")))?;

    let mdtr = items
        .next()
        .ok_or(Error::MissingField(String::from("MDTR")))?
        .parse::<f64>()
        .map_err(|_| Error::FieldParsing(String::from("MDTR")))?
        * 1E-10;

    let smdt = items
        .next()
        .ok_or(Error::MissingField(String::from("SMDT")))?
        .parse::<f64>()
        .map_err(|_| Error::FieldParsing(String::from("SMDT")))?
        * 1E-13;

    let mdio = items
        .next()
        .ok_or(Error::MissingField(String::from("MDIO")))?
        .parse::<f64>()
        .map_err(|_| Error::FieldParsing(String::from("MDIO")))?
        * 1E-10;

    let smdi = items
        .next()
        .ok_or(Error::MissingField(String::from("SMDI")))?
        .parse::<f64>()
        .map_err(|_| Error::FieldParsing(String::from("SMDI")))?
        * 1E-13;

    Ok(TrackData {
        refsv,
        srsv,
        refsys,
        srsys,
        dsg,
        ioe,
        mdtr,
        smdt,
        mdio,
        smdi,
    })
}

fn parse_without_iono(
    items: &mut std::str::SplitAsciiWhitespace<'_>,
) -> Result<(TrackData, Option<IonosphericData>), Error> {
    let data = parse_data(items)?;
    Ok((data, None))
}

fn parse_with_iono(
    items: &mut std::str::SplitAsciiWhitespace<'_>,
) -> Result<(TrackData, Option<IonosphericData>), Error> {
    let data = parse_data(items)?;

    let msio = items
        .next()
        .ok_or(Error::MissingField(String::from("MSIO")))?
        .parse::<f64>()
        .map_err(|_| Error::FieldParsing(String::from("MSIO")))?
        * 0.1E-9;

    let smsi = items
        .next()
        .ok_or(Error::MissingField(String::from("SMSI")))?
        .parse::<f64>()
        .map_err(|_| Error::FieldParsing(String::from("SMSI")))?
        * 0.1E-12;

    let isg = items
        .next()
        .ok_or(Error::MissingField(String::from("ISG")))?
        .parse::<f64>()
        .map_err(|_| Error::FieldParsing(String::from("ISG")))?
        * 0.1E-9;

    Ok((data, Some(IonosphericData { msio, smsi, isg })))
}

impl std::str::FromStr for Track {
    type Err = Error;
    /*
     * Builds a Track from given str description
     */
    fn from_str(line: &str) -> Result<Self, Self::Err> {
        let cleanedup = String::from(line.trim());
        let _epoch = Epoch::default();
        let mut items = cleanedup.split_ascii_whitespace();

        let nb_items = items.clone().count();

        let sv = SV::from_str(
            items
                .next()
                .ok_or(Error::MissingField(String::from("SV")))?,
        )?;

        let class = CommonViewClass::from_str(
            items
                .next()
                .ok_or(Error::MissingField(String::from("CL")))?
                .trim(),
        )?;

        let mjd = items
            .next()
            .ok_or(Error::MissingField(String::from("MJD")))?
            .parse::<i32>()
            .map_err(|_| Error::FieldParsing(String::from("MJD")))?;

        let trk_sttime = items
            .next()
            .ok_or(Error::MissingField(String::from("STTIME")))?;

        if trk_sttime.len() < 6 {
            return Err(Error::InvalidTrkTimeFormat);
        }

        let h = trk_sttime[0..2]
            .parse::<u8>()
            .map_err(|_| Error::FieldParsing(String::from("STTIME:%H")))?;

        let m = trk_sttime[2..4]
            .parse::<u8>()
            .map_err(|_| Error::FieldParsing(String::from("STTIME:%M")))?;

        let s = trk_sttime[4..6]
            .parse::<u8>()
            .map_err(|_| Error::FieldParsing(String::from("STTIME:%S")))?;

        let mut epoch = Epoch::from_mjd_utc(mjd as f64);
        epoch += (h as f64) * Unit::Hour;
        epoch += (m as f64) * Unit::Minute;
        epoch += (s as f64) * Unit::Second;

        let duration = Duration::from_seconds(
            items
                .next()
                .ok_or(Error::MissingField(String::from("STTIME")))?
                .parse::<f64>()
                .map_err(|_| Error::FieldParsing(String::from("STTIME")))?,
        );

        let elevation = items
            .next()
            .ok_or(Error::MissingField(String::from("ELV")))?
            .parse::<f64>()
            .map_err(|_| Error::FieldParsing(String::from("ELV")))?
            * 0.1;

        let azimuth = items
            .next()
            .ok_or(Error::MissingField(String::from("AZTH")))?
            .parse::<f64>()
            .map_err(|_| Error::FieldParsing(String::from("AZTH")))?
            * 0.1;

        let (data, iono) = match nb_items {
            TRACK_WITH_IONOSPHERIC => parse_with_iono(&mut items)?,
            TRACK_WITHOUT_IONOSPHERIC => parse_without_iono(&mut items)?,
            _ => {
                return Err(Error::InvalidFormat);
            },
        };

        let fr = GlonassChannel::from_str(
            items
                .next()
                .ok_or(Error::MissingField(String::from("fr")))?,
        )?;

        let hc = items
            .next()
            .ok_or(Error::MissingField(String::from("hc")))?
            .parse::<u8>()
            .map_err(|_| Error::FieldParsing(String::from("hc")))?;

        let frc: String = items
            .next()
            .ok_or(Error::MissingField(String::from("frc")))?
            .parse()
            .map_err(|_| Error::FieldParsing(String::from("frc")))?;

        // checksum
        let ck = items
            .next()
            .ok_or(Error::MissingField(String::from("ck")))?;

        let _ck =
            u8::from_str_radix(ck, 16).map_err(|_| Error::FieldParsing(String::from("ck")))?;

        // let cksum = calc_crc(&line.split_at(end_pos - 1).0)?;

        // verification
        /*if cksum != ck {
            println!("GOT {} EXPECT {}", ck, cksum);
            return Err(Error::ChecksumError(cksum, ck))
        }*/

        Ok(Track {
            sv,
            class,
            epoch,
            duration,
            elevation,
            azimuth,
            data,
            iono,
            fr,
            hc,
            frc,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use crate::track::GlonassChannel;
    use gnss::prelude::{Constellation, SV};
    use hifitime::Duration;
    use std::str::FromStr;
    //use cggtts::prelude::IonosphericData;
    //use cggtts::prelude::CommonViewClass;
    //use cggtts::prelude::Track;
    #[test]
    fn test_glonass_channel() {
        let c = GlonassChannel::Unknown;
        assert_eq!(c.to_string(), "00");
        let c = GlonassChannel::ChanNum(1);
        assert_eq!(c.to_string(), "01");
        let c = GlonassChannel::ChanNum(10);
        assert_eq!(c.to_string(), "0A");
        assert_eq!(c, GlonassChannel::ChanNum(10));
        assert!(c != GlonassChannel::Unknown);
        assert_eq!(GlonassChannel::default(), GlonassChannel::Unknown);
    }
    #[test]
    fn track_parsing() {
        let content =
"G99 99 59568 001000 0780 099 0099 +9999999999 +99999       +1536   +181   26 999 9999 +999 9999 +999 00 00 L1C D3";
        let track = Track::from_str(content);
        assert!(track.is_ok());
        let track = track.unwrap();
        assert_eq!(
            track.sv,
            SV {
                constellation: Constellation::GPS,
                prn: 99
            }
        );
        assert_eq!(track.class, CommonViewClass::SingleChannel);
        assert!(track.follows_bipm_specs());
        assert_eq!(track.duration, Duration::from_seconds(780.0));
        assert!(!track.has_ionospheric_data());
        assert_eq!(track.elevation, 9.9);
        assert_eq!(track.azimuth, 9.9);
        assert_eq!(track.fr, GlonassChannel::Unknown);
        assert!((track.data.dsg - 2.5E-9).abs() < 1E-6);
        assert!((track.data.srsys - 2.83E-11).abs() < 1E-6);
        assert_eq!(track.hc, 0);
        assert_eq!(track.frc, "L1C");

        let content =
"G99 99 59563 001400 0780 099 0099 +9999999999 +99999       +1588  +1027   27 999 9999 +999 9999 +999 00 00 L1C EA";
        let track = Track::from_str(content);
        assert!(track.is_ok());
        let track = track.unwrap();
        assert_eq!(
            track.sv,
            SV {
                constellation: Constellation::GPS,
                prn: 99
            }
        );
        assert_eq!(track.class, CommonViewClass::SingleChannel);
        assert!(track.follows_bipm_specs());
        assert_eq!(track.duration, Duration::from_seconds(780.0));
        assert!(!track.has_ionospheric_data());
        assert_eq!(track.elevation, 9.9);
        assert_eq!(track.azimuth, 9.9);
        assert_eq!(track.fr, GlonassChannel::Unknown);
        assert_eq!(track.hc, 0);
        assert_eq!(track.frc, "L1C");

        let content =
"G99 99 59563 232200 0780 099 0099 +9999999999 +99999       +1529   -507   23 999 9999 +999 9999 +999 00 00 L1C D9";
        let track = Track::from_str(content);
        assert!(track.is_ok());
        let track = track.unwrap();
        assert_eq!(track.class, CommonViewClass::SingleChannel);
        assert!(track.follows_bipm_specs());
        assert_eq!(track.duration, Duration::from_seconds(780.0));
        assert!(!track.has_ionospheric_data());
        assert_eq!(track.elevation, 9.9);
        assert_eq!(track.azimuth, 9.9);
        assert_eq!(track.fr, GlonassChannel::Unknown);
        assert_eq!(track.hc, 0);
        assert_eq!(track.frc, "L1C");

        let content =
"G99 99 59567 001400 0780 099 0099 +9999999999 +99999       +1561   -151   27 999 9999 +999 9999 +999 00 00 L1C D4";
        let track = Track::from_str(content);
        assert!(track.is_ok());
        let track = track.unwrap();
        assert_eq!(
            track.sv,
            SV {
                constellation: Constellation::GPS,
                prn: 99
            }
        );
        assert_eq!(track.class, CommonViewClass::SingleChannel);
        //assert_eq!(track.trktime 043400)
        assert!(track.follows_bipm_specs());
        assert_eq!(track.duration, Duration::from_seconds(780.0));
        assert!(!track.has_ionospheric_data());
        assert_eq!(track.elevation, 9.9);
        assert_eq!(track.azimuth, 9.9);
        assert_eq!(track.fr, GlonassChannel::Unknown);
        assert_eq!(track.hc, 0);
        assert_eq!(track.frc, "L1C");
    }
    #[test]
    fn parser_ionospheric() {
        let content =
"R24 FF 57000 000600 0780 347 0394 +1186342 +0 163 +0 40 2 141 +22 23 -1 23 -1 29 +2 0 L3P EF";
        let track = Track::from_str(content);
        //assert_eq!(track.is_ok(), true);
        let track = track.unwrap();
        assert_eq!(track.class, CommonViewClass::MultiChannel);
        assert!(track.follows_bipm_specs());
        assert_eq!(track.duration, Duration::from_seconds(780.0));
        assert!(track.has_ionospheric_data());
        let iono = track.iono.unwrap();
        assert_eq!(iono.msio, 23.0E-10);
        assert_eq!(iono.smsi, -1.0E-13);
        assert_eq!(iono.isg, 29.0E-10);
        assert_eq!(track.elevation, 34.7);
        assert!((track.azimuth - 39.4).abs() < 1E-6);
        assert_eq!(track.fr, GlonassChannel::ChanNum(2));
        assert_eq!(track.hc, 0);
        assert_eq!(track.frc, "L3P");
    }
}
