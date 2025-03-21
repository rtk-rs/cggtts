#![doc(html_logo_url = "https://raw.githubusercontent.com/rtk-rs/.github/master/logos/logo2.jpg")]
#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]

/*
 * CGGTTS is part of the rtk-rs framework.
 * Authors: Guillaume W. Bres <guillaume.bressaix@gmail.com> et al.
 * (cf. https://github.com/rtk-rs/cggtts/graphs/contributors)
 * This framework is shipped under Mozilla Public V2 license.
 *
 * Documentation: https://github.com/rtk-rs/cggtts
 */

extern crate gnss_rs as gnss;

use gnss::prelude::{Constellation, SV};
use hifitime::{Duration, Epoch, TimeScale};

use std::{
    fs::File,
    io::{BufRead, BufReader, BufWriter, Read, Write},
    path::Path,
    str::FromStr,
};

#[cfg(feature = "flate2")]
use flate2::{read::GzDecoder, write::GzEncoder, Compression as GzCompression};

mod header;

#[cfg(feature = "scheduler")]
#[cfg_attr(docsrs, doc(cfg(feature = "scheduler")))]
mod scheduler;

#[cfg(feature = "tracker")]
#[cfg_attr(docsrs, doc(cfg(feature = "tracker")))]
mod tracker;

#[cfg(test)]
mod tests;

pub mod buffer;
pub mod errors;
pub mod track;

#[cfg(feature = "serde")]
#[macro_use]
extern crate serde;

pub mod prelude {

    pub use crate::{
        header::*,
        track::{CommonViewClass, IonosphericData, Track, TrackData},
        CGGTTS,
    };

    #[cfg(feature = "scheduler")]
    pub use crate::scheduler::{calendar::CommonViewCalendar, period::CommonViewPeriod};

    #[cfg(feature = "tracker")]
    pub use crate::tracker::{FitError, FittedData, Observation, SVTracker, SkyTracker};

    // pub re-export
    pub use gnss::prelude::{Constellation, SV};
    pub use hifitime::prelude::{Duration, Epoch, TimeScale};
}

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{
    buffer::Utf8Buffer,
    errors::{FormattingError, ParsingError},
    header::{Header, ReferenceTime},
    track::{CommonViewClass, Track},
};

/// [CGGTTS] is a structure split in two:
/// - the [Header] section gives general information
/// about the measurement system and context
/// - [Track]s, ofen times referred to as measurements,
/// describe the behavior of the measurement system's local clock
/// with resepect to satellite onboard clocks. Each [Track]
/// was solved by tracking satellites individually.
/// NB: Correct [CGGTTS] only contain [Track]s of the same [Constellation].
///  
/// Remote (measurement systems) clock comparison is then allowed by
/// exchanging remote [CGGTTS] (from both sites), and comparing synchronous
/// (on both sites) [Track]s referring to identical satellite vehicles.
/// This is called the common view time transfer technique.
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CGGTTS {
    /// [Header] gives general information
    pub header: Header,
    /// [Track]s describe the result of track fitting,
    /// in chronological order.
    pub tracks: Vec<Track>,
}

impl CGGTTS {
    /// Returns true if all [Track]s (measurements) seems compatible
    /// with the [CommonViewPeriod] recommended by BIPM.
    /// This cannot be a complete confirmation, because only the receiver
    /// that generated this data knows if [Track] collection and fitting
    /// was implemented correctly.
    pub fn follows_bipm_tracking(&self) -> bool {
        for track in self.tracks.iter() {
            if !track.follows_bipm_tracking() {
                return false;
            }
        }
        true
    }

    /// Returns true if all tracks (measurements) contained in this
    /// [CGGTTS] have ionospheric parameters estimate.
    pub fn has_ionospheric_data(&self) -> bool {
        for track in self.tracks.iter() {
            if !track.has_ionospheric_data() {
                return false;
            }
        }
        true
    }

    /// Returns [CommonViewClass] used in this file.
    /// ## Returns
    /// - [CommonViewClass::MultiChannel] if at least one track (measurement)
    /// is [CommonViewClass::MultiChannel] measurement
    /// - [CommonViewClass::SingleChannel] if all tracks (measurements)
    /// are [CommonViewClass::SingleChannel] measurements
    pub fn common_view_class(&self) -> CommonViewClass {
        for trk in self.tracks.iter() {
            if trk.class != CommonViewClass::SingleChannel {
                return CommonViewClass::MultiChannel;
            }
        }
        CommonViewClass::SingleChannel
    }

    /// Returns true if this [CGGTTS] is a single channel [CGGTTS],
    /// meaning, all tracks (measurements) are [CommonViewClass::SingleChannel] measurements
    pub fn single_channel(&self) -> bool {
        self.common_view_class() == CommonViewClass::SingleChannel
    }

    /// Returns true if this [CGGTTS] is a single channel [CGGTTS],
    /// meaning, all tracks (measurements) are [CommonViewClass::MultiChannel] measurements
    pub fn multi_channel(&self) -> bool {
        self.common_view_class() == CommonViewClass::MultiChannel
    }

    /// Returns true if this is a [Constellation::GPS] [CGGTTS].
    /// Meaning, all measurements [Track]ed this constellation.
    pub fn is_gps_cggtts(&self) -> bool {
        if let Some(first) = self.tracks.first() {
            first.sv.constellation == Constellation::GPS
        } else {
            false
        }
    }

    /// Returns true if this is a [Constellation::Galileo] [CGGTTS].
    /// Meaning, all measurements [Track]ed this constellation.
    pub fn is_galileo_cggtts(&self) -> bool {
        if let Some(first) = self.tracks.first() {
            first.sv.constellation == Constellation::Galileo
        } else {
            false
        }
    }

    /// Returns true if this is a [Constellation::BeiDou] [CGGTTS].
    /// Meaning, all measurements [Track]ed this constellation.
    pub fn is_beidou_cggtts(&self) -> bool {
        if let Some(first) = self.tracks.first() {
            first.sv.constellation == Constellation::BeiDou
        } else {
            false
        }
    }

    /// Returns true if this is a [Constellation::Glonass] [CGGTTS].
    /// Meaning, all measurements[Track]ed this constellation.
    pub fn is_glonass_cggtts(&self) -> bool {
        if let Some(first) = self.tracks.first() {
            first.sv.constellation == Constellation::Glonass
        } else {
            false
        }
    }

    /// Returns true if this is a [Constellation::QZSS] [CGGTTS].
    /// Meaning, all measurements[Track]ed this constellation.
    pub fn is_qzss_cggtts(&self) -> bool {
        if let Some(first) = self.tracks.first() {
            first.sv.constellation == Constellation::QZSS
        } else {
            false
        }
    }

    /// Returns true if this is a [Constellation::IRNSS] [CGGTTS].
    /// Meaning, all measurements [Track]ed this constellation.
    pub fn is_irnss_cggtts(&self) -> bool {
        if let Some(first) = self.tracks.first() {
            first.sv.constellation == Constellation::IRNSS
        } else {
            false
        }
    }

    /// Returns true if this is a [Constellation::SBAS] [CGGTTS].
    /// Meaning, all measurements[Track]ed geostationary vehicles.
    pub fn is_sbas_cggtts(&self) -> bool {
        if let Some(first) = self.tracks.first() {
            first.sv.constellation.is_sbas()
        } else {
            false
        }
    }

    /// Returns [Track]s (measurements) Iterator
    pub fn tracks_iter(&self) -> impl Iterator<Item = &Track> {
        self.tracks.iter()
    }

    /// Iterate over [Track]s (measurements) that result from tracking
    /// this particular [SV] only.
    pub fn sv_tracks(&self, sv: SV) -> impl Iterator<Item = &Track> {
        self.tracks
            .iter()
            .filter_map(move |trk| if trk.sv == sv { Some(trk) } else { None })
    }

    /// Returns first Epoch contained in this file.
    pub fn first_epoch(&self) -> Option<Epoch> {
        self.tracks.first().map(|trk| trk.epoch)
    }

    /// Returns last Epoch contained in this file.
    pub fn last_epoch(&self) -> Option<Epoch> {
        self.tracks.last().map(|trk| trk.epoch)
    }

    /// Returns total [Duration] of this [CGGTTS].
    pub fn total_duration(&self) -> Duration {
        if let Some(t1) = self.last_epoch() {
            if let Some(t0) = self.first_epoch() {
                return t1 - t0;
            }
        }
        Duration::ZERO
    }

    /// Generates a standardized file name that would describes
    /// this [CGGTTS] correctly according to naming conventions.
    /// This method is infaillible, but might generate incomplete
    /// results. In particular, this [CGGTTS] should not be empty
    /// and must contain [Track]s measurements for this to work correctly.
    /// ## Inputs
    /// - custom_lab: Possible LAB ID overwrite and customization.
    /// Two characters are expected here, the result will not
    /// respect the standard convention if you provide less.
    /// When not defined, we use the LAB ID that was previously parsed.
    /// - custom_id: Possible GNSS RX identification number
    /// or whatever custom ID number you desire.
    /// Two characters are expected here, the result will not
    /// respect the standard convention if you provide less.
    /// When not defined, we use the first two digits of the serial number
    /// that was previously parsed.
    pub fn standardized_file_name(
        &self,
        custom_lab: Option<&str>,
        custom_id: Option<&str>,
    ) -> String {
        let mut ret = String::new();

        // Grab first letter of constellation
        if let Some(first) = self.tracks.first() {
            ret.push_str(&format!("{:x}", first.sv.constellation));
        } else {
            ret.push('X');
        }

        // Second letter depends on channelling capabilities
        if self.has_ionospheric_data() {
            ret.push('Z') // Dual Freq / Multi channel
        } else if self.single_channel() {
            ret.push('S') // Single Freq / Channel
        } else {
            ret.push('M') // Single Freq / Multi Channel
        }

        // LAB / Agency
        if let Some(custom_lab) = custom_lab {
            let size = std::cmp::min(custom_lab.len(), 2);
            ret.push_str(&custom_lab[0..size]);
        } else {
            let size = std::cmp::min(self.header.station.len(), 2);
            ret.push_str(&self.header.station[0..size]);
        }

        // GNSS RX / SN
        if let Some(custom_id) = custom_id {
            let size = std::cmp::min(custom_id.len(), 2);
            ret.push_str(&custom_id[..size]);
        } else {
            let size = std::cmp::min(self.header.receiver.serial_number.len(), 2);
            ret.push_str(&self.header.receiver.serial_number[..size]);
        }

        if let Some(epoch) = self.first_epoch() {
            let mjd = epoch.to_mjd_utc_days();
            ret.push_str(&format!("{:02.3}", (mjd / 1000.0)));
        } else {
            ret.push_str("dd.ddd");
        }

        ret
    }

    /// Parse [CGGTTS] from a local file.
    /// Advanced CGGTTS files generated from modern GNSS
    /// receivers may describe the ionospheric delay compensation:
    /// ```
    /// use cggtts::prelude::CGGTTS;
    ///
    /// let cggtts = CGGTTS::from_file("data/CGGTTS/GZGTR560.258")
    ///     .unwrap();
    ///
    /// if let Some(track) = cggtts.tracks.first() {
    ///     assert_eq!(track.has_ionospheric_data(), true);
    ///     if let Some(iono) = track.iono {
    ///         let (msio, smsi, isg) = (iono.msio, iono.smsi, iono.isg);
    ///     }
    /// }
    ///```
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ParsingError> {
        let fd = File::open(path).unwrap_or_else(|e| panic!("File open error: {}", e));

        let mut reader = BufReader::new(fd);
        Self::parse(&mut reader)
    }

    /// Parse a new [CGGTTS] from any [Read]able interface.
    /// This will fail on:
    /// - Any critical standard violation
    /// - If file revision is not 2E (latest)
    /// - If following [Track]s do not contain the same [Constellation]
    pub fn parse<R: Read>(reader: &mut BufReader<R>) -> Result<Self, ParsingError> {
        // Parse header section
        let header = Header::parse(reader)?;

        // Parse tracks:
        // consumes all remaning lines and attempt parsing on each new line.
        // Line CRC is internally verified for each line.
        // We abort if Constellation content is not constant, as per standard conventions.
        let mut tracks = Vec::with_capacity(16);
        let lines = reader.lines();

        let mut constellation = Option::<Constellation>::None;

        for line in lines {
            if line.is_err() {
                continue;
            }

            let line = line.unwrap();

            if let Ok(track) = Track::from_str(&line) {
                // constellation content verification
                if let Some(constellation) = &constellation {
                    if track.sv.constellation != *constellation {
                        return Err(ParsingError::MixedConstellation);
                    }
                } else {
                    constellation = Some(track.sv.constellation);
                }

                tracks.push(track);
            }
        }

        Ok(Self { header, tracks })
    }

    /// Parse [CGGTTS] from gzip compressed local path.
    #[cfg(feature = "flate2")]
    #[cfg_attr(docsrs, doc(cfg(feature = "flate2")))]
    pub fn from_gzip_file<P: AsRef<Path>>(path: P) -> Result<Self, ParsingError> {
        let fd = File::open(path).unwrap_or_else(|e| panic!("File open error: {}", e));

        let reader = GzDecoder::new(fd);

        let mut reader = BufReader::new(reader);
        Self::parse(&mut reader)
    }

    /// Format [CGGTTS] following standard specifications.
    ///
    /// To produce valid advanced CGGTTS, one should specify:
    /// - IMS [Hardware]
    /// - ionospheric parameters
    /// - System delay definitions, per signal carrier
    ///
    /// ```
    /// use std::io::Write;
    ///
    /// use cggtts::prelude::{
    ///     CGGTTS,
    ///     Header,
    ///     Hardware, Coordinates,
    ///     Track, TrackData,
    ///     SV, Epoch, Duration,
    ///     CommonViewClass,
    /// };
    ///
    /// let rcvr = Hardware::default()
    ///     .with_manufacturer("SEPTENTRIO")  
    ///     .with_model("POLARRx5")
    ///     .with_serial_number("#12345")
    ///     .with_release_year(2023)
    ///     .with_release_version("v1");
    ///
    /// // form a header
    /// let header = Header::default()
    ///     .with_station("AJACFR")
    ///     .with_receiver_hardware(rcvr);
    ///
    /// // Although CGGTTS intends ITRF high precision
    /// // frames, you may use whatever you can or need
    /// let header = header
    ///     .with_reference_frame("ITRF");
    ///
    /// // It is best practice to specify the APC coordinates
    /// // (obviously in previous reference frame)
    /// let header = header
    ///     .with_apc_coordinates(Coordinates {
    ///         x: 1.0_f64,
    ///         y: 2.0_f64,
    ///         z: 3.0_f64,
    ///     });
    ///
    /// // Create a [CGGTTS]
    /// let mut cggtts = CGGTTS::default()
    ///     .with_header(header);
    ///
    /// // Tracking context
    /// let epoch = Epoch::default();
    /// let sv = SV::default();
    /// let (elevation, azimuth) = (0.0_f64, 0.0_f64);
    /// let duration = Duration::from_seconds(780.0);
    ///
    /// // receiver channel being used
    /// let rcvr_channel = 0_u8;
    ///
    /// // TrackData is always mandatory (for each track)
    /// let data = TrackData {
    ///     refsv: 0.0_f64,
    ///     srsv: 0.0_f64,
    ///     refsys: 0.0_f64,
    ///     srsys: 0.0_f64,
    ///     dsg: 0.0_f64,
    ///     ioe: 0_u16,
    ///     smdt: 0.0_f64,
    ///     mdtr: 0.0_f64,
    ///     mdio: 0.0_f64,
    ///     smdi: 0.0_f64,
    /// };
    ///
    /// // option 1: track resulting from a single SV observation
    /// let track = Track::new(
    ///     sv,
    ///     epoch,
    ///     duration,
    ///     CommonViewClass::SingleChannel,
    ///     elevation,
    ///     azimuth,
    ///     data,
    ///     None,
    ///     rcvr_channel, // receiver channel
    ///     "L1C",
    /// );
    ///
    /// cggtts.tracks.push(track);
    ///
    /// // option 2: track resulting from multi channel SV observation
    /// let track = Track::new(
    ///     sv,
    ///     epoch,
    ///     duration,
    ///     CommonViewClass::MultiChannel,
    ///     elevation,
    ///     azimuth,
    ///     data,
    ///     None,
    ///     rcvr_channel, // receiver channel
    ///     "L1C",
    /// );
    ///
    /// cggtts.tracks.push(track);
    ///
    /// // option 3: when working with Glonass, use the dedicated method
    /// let track = Track::new_glonass(
    ///     sv,
    ///     epoch,
    ///     duration,
    ///     CommonViewClass::SingleChannel,
    ///     elevation,
    ///     azimuth,
    ///     data,
    ///     None,
    ///     rcvr_channel, // receiver channel
    ///     1, // FDMA channel
    ///     "C1P",
    /// );
    ///
    /// // produce CGGTTS
    /// cggtts.to_file("/tmp/test.txt")
    ///   .unwrap();
    /// ```
    pub fn format<W: Write>(&self, writer: &mut BufWriter<W>) -> Result<(), FormattingError> {
        const TRACK_LABELS_WITH_IONOSPHERIC_DATA: &str =
        "SAT CL  MJD  STTIME TRKL ELV AZTH   REFSV      SRSV     REFSYS    SRSYS DSG IOE MDTR SMDT MDIO SMDI MSIO SMSI ISG FR HC FRC CK";

        const UNIT_LABELS_WITH_IONOSPHERIC : &str = "             hhmmss  s  .1dg .1dg    .1ns     .1ps/s     .1ns    .1ps/s .1ns     .1ns.1ps/s.1ns.1ps/s.1ns.1ps/s.1ns";

        const TRACK_LABELS_WITHOUT_IONOSPHERIC_DATA: &str =
            "SAT CL  MJD  STTIME TRKL ELV AZTH   REFSV      SRSV     REFSYS    SRSYS  DSG IOE MDTR SMDT MDIO SMDI FR HC FRC CK";

        const UNIT_LABELS_WITHOUT_IONOSPHERIC :&str = "             hhmmss  s  .1dg .1dg    .1ns     .1ps/s     .1ns    .1ps/s .1ns     .1ns.1ps/s.1ns.1ps/s";

        // create local (tiny) Utf-8 buffer
        let mut buf = Utf8Buffer::new(1024);

        // format header
        self.header.format(writer, &mut buf)?;

        // BLANK at end of header section
        write!(writer, "\n")?;

        // format track labels
        if self.has_ionospheric_data() {
            writeln!(writer, "{}", TRACK_LABELS_WITH_IONOSPHERIC_DATA)?;
            writeln!(writer, "{}", UNIT_LABELS_WITH_IONOSPHERIC,)?;
        } else {
            writeln!(writer, "{}", TRACK_LABELS_WITHOUT_IONOSPHERIC_DATA)?;
            writeln!(writer, "{}", UNIT_LABELS_WITHOUT_IONOSPHERIC)?;
        }

        // format all tracks
        for track in self.tracks.iter() {
            track.format(writer, &mut buf)?;
            write!(writer, "\n")?;
        }

        Ok(())
    }

    /// Writes this [CGGTTS] into readable local file
    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), FormattingError> {
        let fd = File::create(path)?;
        let mut writer = BufWriter::new(fd);
        self.format(&mut writer)
    }

    /// Writes this [CGGTTS] into gzip compressed local file
    #[cfg(feature = "flate2")]
    pub fn to_gzip_file<P: AsRef<Path>>(&self, path: P) -> Result<(), FormattingError> {
        let fd = File::create(path)?;
        let compression = GzCompression::new(5);
        let mut writer = BufWriter::new(GzEncoder::new(fd, compression));
        self.format(&mut writer)
    }

    /// Returns a new [CGGTTS] ready to track in [TimeScale::UTC].
    /// This is the most (most) general use case, for the simple reason
    /// that UTC is a worldwide constant, hence, allows worldwide common-view.
    /// You can use our other method for exotic contexts.
    /// NB: use this prior solving any [Track]s, otherwise
    /// it will corrupt previously solved content, because
    /// it does not perform the time shift for you.
    pub fn with_utc_reference_time(&self) -> Self {
        let mut s = self.clone();
        s.header = s.header.with_reference_time(TimeScale::UTC.into());
        s
    }

    /// Returns a new [CGGTTS] ready to track in custom UTC-replica.
    /// Use this method when setting up a [CGGTTS] production context.
    /// NB(1): we differentiate UTC-replica (unofficial or local UTC)
    /// from custom reference time system (exotic or private),
    /// for which you have [Self::with_custom_reference_time].
    /// NB(2): use this prior solving any [Track]s, otherwise
    /// it will corrupt previously solved content, because
    /// it does not perform the time shift for you.
    /// ## Inputs
    /// - name: name of your UTC replica (also referred to, as UTCk).
    pub fn with_utc_replica_reference_time(&self, name: &str) -> Self {
        let mut s = self.clone();
        s.header = s
            .header
            .with_reference_time(ReferenceTime::UTCk(name.to_string()));
        s
    }

    /// Returns a new [CGGTTS] ready to track in [TimeScale::TAI].
    /// Use this method when setting up a [CGGTTS] production context.
    /// NB: use this prior solving any [Track]s, otherwise
    /// it will corrupt previously solved content, because
    /// it does not perform the time shift for you.
    pub fn with_tai_reference_time(&self) -> Self {
        let mut s = self.clone();
        s.header = s.header.with_reference_time(TimeScale::TAI.into());
        s
    }

    /// Returns a new [CGGTTS] ready to track in custom timescale
    /// (either exotic, privately owned..).
    /// Use this method when setting up a [CGGTTS] production context.
    /// NB(1): we differentiate custom reference time systems from
    /// UTC-replica (unofficial or local UTC),
    /// for which you have [Self::with_utc_replica_reference_time].
    /// NB(2): use this prior solving any [Track]s, otherwise
    /// it will corrupt previously solved content, because
    /// it does not perform the time shift for you.
    /// ## Inputs
    /// - name: name of your custom timescale
    pub fn with_custom_reference_time(&self, name: &str) -> Self {
        let mut s = self.clone();
        s.header = s
            .header
            .with_reference_time(ReferenceTime::UTCk(name.to_string()));
        s
    }

    /// Copies and returns new [CGGTTS] with updated [Header] section.
    pub fn with_header(&self, header: Header) -> Self {
        let mut s = self.clone();
        s.header = header;
        s
    }

    /// Copies and returns new [CGGTTS] with updated [Track] list.
    pub fn with_tracks(&self, tracks: Vec<Track>) -> Self {
        let mut s = self.clone();
        s.tracks = tracks;
        s
    }
}
