use crate::prelude::{CommonViewClass, Duration, Epoch, IonosphericData, Track, TrackData, SV};

/// [FittedData] resulting from running the fit algorithm over many [Observation]s.
#[derive(Debug, Copy, Default, Clone)]
pub struct FittedData {
    /// [SV] that was used
    pub sv: SV,
    /// Fit time window duration
    pub duration: Duration,
    /// Fit start time
    pub first_t: Epoch,
    /// [Epoch] at midtrack
    pub midtrack: Epoch,
    /// Satellite elevation at midtrack (in degrees)
    pub elevation_deg: f64,
    /// Satellite azimuth at midtrack (in degrees)
    pub azimuth_deg: f64,
    /// Satellite onboard clock offset to local clock
    pub refsv_s: f64,
    /// REFSV derivative (s/s)
    pub srsv_s_s: f64,
    /// Satellite onboard clock offset to timescale
    pub refsys_s: f64,
    /// REFSYS derivative (s/s)
    pub srsys_s_s: f64,
    /// DSG: REFSYS Root Mean Square
    pub dsg: f64,
    /// MDTR: modeled troposphere delay (s)
    pub mdtr_s: f64,
    /// SMDT: MDTR derivative (s/s)
    pub smdt_s_s: f64,
    /// MDIO: modeled ionosphere delay (s)
    pub mdio_s: f64,
    /// SMDI: MDIO derivative (s/s)
    pub smdi_s_s: f64,
    /// Possible MSIO: measured ionosphere delay (s)
    pub msio_s: Option<f64>,
    /// Possible MSIO derivative (s/s)
    pub smsi_s_s: Option<f64>,
    /// Possible ISG: MSIO Root Mean Square
    pub isg: Option<f64>,
}

impl FittedData {
    /// Form a new CGGTTS [Track] from this [FittedData],
    /// ready to be formatted.
    /// ## Input
    /// - class: [CommonViewClass]
    /// - data: serves many purposes.
    /// Most often (GPS, Gal, QZSS) it is the Issue of Ephemeris.
    /// For Glonass, it should be between 1-96 as the date of ephemeris as
    /// daily quarters of hours, starting at 1 for 00:00:00 midnight.
    /// For BeiDou, the hour of clock, between 0-23 should be used.
    /// - rinex_code: RINEX readable code.
    pub fn to_track(&self, class: CommonViewClass, data: u16, rinex_code: &str) -> Track {
        Track {
            class,
            epoch: self.first_t,
            duration: self.duration,
            sv: self.sv,
            azimuth_deg: self.azimuth_deg,
            elevation_deg: self.elevation_deg,
            fdma_channel: None,
            data: TrackData {
                ioe: data,
                refsv: self.refsv_s,
                srsv: self.srsv_s_s,
                refsys: self.refsys_s,
                srsys: self.srsys_s_s,
                dsg: self.dsg,
                mdtr: self.mdtr_s,
                smdt: self.smdt_s_s,
                mdio: self.mdio_s,
                smdi: self.smdi_s_s,
            },
            iono: if let Some(msio_s) = self.msio_s {
                Some(IonosphericData {
                    msio: msio_s,
                    smsi: self.smsi_s_s.unwrap(),
                    isg: self.isg.unwrap(),
                })
            } else {
                None
            },
            // TODO
            hc: 0,
            frc: rinex_code.to_string(),
        }
    }
}
