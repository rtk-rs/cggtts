//! Satellite track fitting
use hifitime::Unit;
use log::debug;
use polyfit_rs::polyfit_rs::polyfit;
use thiserror::Error;

use crate::prelude::{Duration, Epoch, IonosphericData, TrackData, SV};

use std::collections::BTreeMap;

fn linear_reg_2d(i: (f64, f64), j: (f64, f64)) -> (f64, f64) {
    let (_, y_i) = i;
    let (x_j, y_j) = j;
    let a = y_j - y_i;
    let b = y_j - a * x_j;
    (a, b)
}

/// CGGTTS track formation errors
#[derive(Debug, Clone, Error)]
pub enum FitError {
    /// At least two symbols are required to fit
    #[error("track fitting requires at least 3 observations")]
    NotEnoughSymbols,
    /// Linear regression failure. Either extreme values
    /// encountered or data gaps are present.
    #[error("linear regression failure")]
    LinearRegressionFailure,
}

/// [SVTracker] is used to track an individual [SV].
#[derive(Default, Debug, Clone)]
pub struct SVTracker {
    /// SV being tracked
    sv: SV,
    /// Symbols counter
    size: usize,
    /// Sampling gap tolerance
    gap_tolerance: Option<Duration>,
    /// First Epoch of this fit
    t0: Option<Epoch>,
    /// Previous Epoch (for internal logic)
    prev_t: Option<Epoch>,
    /// Internal buffer
    buffer: Vec<Observation>,
}

/// [Observation] you need to provide to attempt a CGGTTS fit.
#[derive(Debug, Default, Clone)]
pub struct Observation {
    /// Epoch of [Observation]
    pub epoch: Epoch,
    /// Satellite onboard clock offset to local clock
    pub refsv: f64,
    /// Satellite onboard clock offset to timescale
    pub refsys: f64,
    /// Modeled Tropospheric Delay in seconds of propagation delay
    pub mdtr: f64,
    /// Elevation in degrees
    pub elevation: f64,
    /// Azimuth in degrees
    pub azimuth: f64,
}

/// [FittedData] resulting from running the fit algorithm over many [Observation]s.
#[derive(Debug, Copy, Default, Clone)]
pub struct FittedData {
    /// [Epoch] at midtrack
    pub midtrack: Epoch,
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
    /// Modeled Tropospheric Delay in seconds of propagation delay
    pub mdtr_s: f64,
    /// Elevation at midtrack (in degrees)
    pub elevation_deg: f64,
    /// Azimuth at midtrack (in degrees)
    pub azimuth_deg: f64,
}

impl SVTracker {
    /// Creates (allocates) a new [SVTracker] for that particular satellite.
    ///
    /// ## Input
    /// - satellite: as [SV]
    /// - tolerance: sampling gap tolerance as [Duration]
    pub fn new(sv: SV, gap_tolerance: Option<Duration>) -> Self {
        Self {
            sv,
            size: 0,
            t0: None,
            prev_t: None,
            gap_tolerance,
            buffer: Vec::with_capacity(16),
        }
    }

    /// Feed new [Observation] at t [Epoch] of observation (sampling).
    /// Although CGGTTS works in UTC internally, we accept any timescale here.
    /// Samples must be provided in chronological order.
    ///
    /// ## Input
    /// - data: [Observation]
    pub fn new_observation(&mut self, data: Observation) {
        if let Some(past_t) = self.prev_t {
            if let Some(tolerance) = self.gap_tolerance {
                let dt = data.epoch - past_t;
                if dt > tolerance {
                    debug!("{}({}) - {} data gap", data.epoch, self.sv, dt);
                    self.size = 0;
                    self.buffer.clear();
                }
            }
        }

        if self.t0.is_none() {
            self.t0 = Some(data.epoch);
        }

        self.prev_t = Some(data.epoch);
        self.buffer.push(data);
        self.size += 1;
    }

    /// Manual reset of the internal buffer.
    pub fn reset(&mut self) {
        self.prev_t = None;
        self.size = 0;
        self.buffer.clear();
    }

    /// True if at least one measurement is currently latched and may contribute to a fit.
    pub fn not_empty(&self) -> bool {
        self.size > 0
    }

    /// Track fitting attempt.
    pub fn fit(
        &mut self,
        trk_duration: Duration,
        sampling_period: Duration,
        trk_midpoint: Epoch,
    ) -> Result<FittedData, FitError> {
        // Request 3 symbols at least
        if self.size < 2 {
            return Err(FitError::NotEnoughSymbols);
        }

        let midpoint = self.size / 2 - 1;

        // Retrieve information @ mid point
        let t_mid = self.buffer[midpoint].epoch;
        let t_mid_s = t_mid.duration.to_unit(Unit::Second);

        let azim_mid = self.buffer[midpoint].azimuth;
        let elev_mid = self.buffer[midpoint].elevation;

        let mut fitted = FittedData::default();

        fitted.midtrack = t_mid;
        fitted.azimuth_deg = azim_mid;
        fitted.elevation_deg = elev_mid;

        // retrieve x_t
        let x_t = self
            .buffer
            .iter()
            .map(|data| data.epoch.duration.to_unit(Unit::Second))
            .collect::<Vec<_>>();

        // REFSV
        let (srsv, srsv_b) = polyfit(
            &x_t,
            self.buffer
                .iter()
                .map(|data| data.refsv)
                .collect::<Vec<_>>()
                .as_slice(),
            1,
        )
        .or(Err(FitError::LinearRegressionFailure))?;

        let refsv = srsv * t_mid_s + srsv_b;

        // REFSYS
        let (srsys, srsys_b) = polyfit(
            &x_t,
            self.buffer
                .iter()
                .map(|data| data.refsys)
                .collect::<Vec<_>>()
                .as_slice(),
            1,
        )
        .or(Err(FitError::LinearRegressionFailure))?;

        let refsys = srsys * t_mid_s + srsys_b;

        // DSG
        let mut dsg = 0.0_f64;
        for data in self.buffer.iter() {
            dsg += (srsys * data.epoch + srsys_b - refsys).powi(2);
        }
        dsg = dsg.sqrt();

        // MDTR
        let (smdt, smdt_b) = polyfit(
            &x_t,
            self.buffer
                .iter()
                .map(|data| data.mdtr)
                .collect::<Vec<_>>()
                .as_slice(),
            1,
        )
        .or(Err(FitError::LinearRegressionFailure))?;

        let mdtr = smdt * t_mid_s + smdt_b;

        // MDIO
        let (smdi, smdi_b) = polyfit(
            &x_t,
            self.buffer
                .iter()
                .map(|data| data.mdio)
                .collect::<Vec<_>>()
                .as_slice(),
            1,
        )
        .or(Err(FitError::LinearRegressionFailure))?;

        let mdio = smdi * t_mid_s + smdi_b;

        fitted.srsv = srsv;
        fitted.refsv = refsv;
        fitted.srsys = srsys;
        fitted.refsys = refsys;
        fitted.dsg = dsg;
        fitted.mdtr_s = mdtr;
        fitted.smdt_s_s = smdt;
        fitted.mdio_s = mdio;
        fitted.smdi_s_s = smdi;

        // reset for next time
        self.t0 = None;
        self.buffer.clear();
        self.size = 0;

        Ok(fitted)
    }
}
