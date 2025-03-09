mod code;
mod delay;
mod formatting;
mod hardware;
mod parsing;
mod reference_time;
mod version;

#[cfg(docsrs)]
use crate::prelude::CGGTTS;

pub use crate::header::{
    code::Code,
    delay::{CalibrationID, Delay, SystemDelay},
    hardware::Hardware,
    reference_time::ReferenceTime,
    version::Version,
};

use crate::prelude::{Epoch, TimeScale};

#[derive(PartialEq, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Coordinates {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Header {
    /// CGGTTS [Version] used at production time of this [CGGTTS].
    pub version: Version,
    /// Date this [Header] was revised. Updated each
    /// time a new field appears.
    pub revision_date: Epoch,
    /// Station name, usually the data producer (agency, laboratory..).
    pub station: String,
    /// Information about GNSS receiver
    pub receiver: Hardware,
    /// # of channels this GNSS receiver possesses
    pub nb_channels: u16,
    /// Possible Ionospheric Measurement System (IMS) information.
    /// Should always be attached to multi channel / modern [CGGTTS].
    pub ims_hardware: Option<Hardware>,
    /// [ReferenceTime] used in the solving process of each [Track]
    pub reference_time: ReferenceTime,
    /// Name of the ECEF Coordinates system in which the APC
    /// [Coordinates] are expressed in.
    pub reference_frame: String,
    /// Antenna Phase Center (APC) coordinates in meters
    pub apc_coordinates: Coordinates,
    /// Short readable comments (if any)
    pub comments: Option<String>,
    /// Measurement [SystemDelay]
    pub delay: SystemDelay,
}

impl Default for Header {
    fn default() -> Self {
        let version = Version::default();
        Self {
            version,
            station: String::from("LAB"),
            nb_channels: Default::default(),
            apc_coordinates: Default::default(),
            receiver: Default::default(),
            ims_hardware: Default::default(),
            comments: Default::default(),
            delay: Default::default(),
            reference_time: Default::default(),
            reference_frame: Default::default(),
            revision_date: Epoch::from_gregorian_utc_at_midnight(2014, 2, 20),
        }
    }
}

impl Header {
    /// Returns [Header] with desired station name
    pub fn with_station(&self, station: &str) -> Self {
        let mut c = self.clone();
        c.station = station.to_string();
        c
    }

    /// Adds readable comments to this [Header].
    /// Try to keep it short, because it will eventually be
    /// wrapped in a single line.
    pub fn with_comment(&self, comment: &str) -> Self {
        let mut s = self.clone();
        s.comments = Some(comment.to_string());
        s
    }

    /// Returns a new [Header] with desired number of channels.
    pub fn with_channels(&self, ch: u16) -> Self {
        let mut c = self.clone();
        c.nb_channels = ch;
        c
    }

    /// Returns a new [Header] with [Hardware] information about
    /// the GNSS receiver.
    pub fn with_receiver_hardware(&self, receiver: Hardware) -> Self {
        let mut c = self.clone();
        c.receiver = receiver;
        c
    }

    /// Returns a new [Header] with [Hardware] information about
    /// the device that help estimate the Ionosphere parameters.
    pub fn with_ims_hardware(&self, ims: Hardware) -> Self {
        let mut c = self.clone();
        c.ims_hardware = Some(ims);
        c
    }

    /// Returns new [Header] with desired APC coordinates in ECEF.
    pub fn with_apc_coordinates(&self, apc: Coordinates) -> Self {
        let mut c = self.clone();
        c.apc_coordinates = apc;
        c
    }

    /// Returns new [Header] with [TimeScale::UTC] reference system time.
    pub fn with_utc_reference_time(&self) -> Self {
        let mut c = self.clone();
        c.reference_time = TimeScale::UTC.into();
        c
    }

    /// Returns new [Header] with desired [ReferenceTime] system
    pub fn with_reference_time(&self, reference: ReferenceTime) -> Self {
        let mut c = self.clone();
        c.reference_time = reference;
        c
    }

    /// Returns new [Header] with desired Reference Frame
    pub fn with_reference_frame(&self, reference: &str) -> Self {
        let mut c = self.clone();
        c.reference_frame = reference.to_string();
        c
    }
}
