/// [FittedData] obtained by fitting many [Observation]s
/// over the common view period.
pub struct FittedData {
    /// Elevation (in degrees) at track midpoint
    pub elevation: f64,
    /// Azimuth (in degrees) at track midpoint
    pub azimuth: f64,
    /// Satellite onboard clock offset to local clock (s)
    pub refsv: f64,
    /// REFSV derivative (s.s⁻¹) 
    pub srsv: f64,
    /// Satellite onboard clock offset to timescale (s)
    pub refsys: f64,
    /// REFSYS derivative (s.s⁻¹)
    pub srsys: f64,
    /// Modeled Tropospheric Delay in seconds of propagation delay (s)
    pub mdtr: f64,
    /// Modeled Ionospheric Delay in seconds of propagation delay (s)
    pub mdio: Option<f64>,
    /// Measured Ionospheric Delay in seconds of propagation delay (s)
    pub msio: Option<f64>,
}