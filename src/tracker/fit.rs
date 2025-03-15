//! Satellite track fitting
use hifitime::Unit;
use log::debug;
use polyfit_rs::polyfit_rs::polyfit;
use thiserror::Error;

use crate::prelude::{Duration, Epoch, FittedData, SV};

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
    /// Modeled Ionospheric Delay in seconds of propagation delay
    pub mdio: f64,
    /// Possible measured Ionospheric Delay in seconds of propagation delay
    pub msio: Option<f64>,
    /// Elevation in degrees
    pub elevation: f64,
    /// Azimuth in degrees
    pub azimuth: f64,
}

impl SVTracker {
    /// Allocate a new [SVTracker] for that particular satellite.
    ///
    /// ## Input
    /// - satellite: [SV]
    pub fn new(satellite: SV) -> Self {
        Self {
            size: 0,
            t0: None,
            prev_t: None,
            sv: satellite,
            gap_tolerance: None,
            buffer: Vec::with_capacity(16),
        }
    }

    /// Define a new [SVTracker] with desired observation gap tolerance.
    pub fn with_gap_tolerance(&self, tolerance: Duration) -> Self {
        let mut s = self.clone();
        s.gap_tolerance = Some(tolerance);
        s
    }

    /// Feed new [Observation] at t [Epoch] of observation (sampling).
    /// Although CGGTTS works in UTC internally, we accept any timescale here.
    /// Samples must be provided in chronological order.
    /// If you provide MSIO, you are expected to provide it at very single epoch,
    /// like any other fields, in order to obtain valid results.
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

    /// Apply fit algorithm over internal buffer.
    /// You manage the buffer content and sampling and are responsible
    /// for the [FittedData] you may obtain. The requirement being at least 3
    /// symbols must have been buffered.
    pub fn fit(&mut self) -> Result<FittedData, FitError> {
        // Request 3 symbols at least
        if self.size < 3 {
            return Err(FitError::NotEnoughSymbols);
        }

        let midpoint = if self.size % 2 == 0 {
            self.size / 2 - 1
        } else {
            (self.size + 1) / 2 - 1
        };

        // Retrieve information @ mid point
        let t0 = self.buffer[0].epoch;
        let t_mid = self.buffer[midpoint].epoch;
        let t_mid_s = t_mid.duration.to_unit(Unit::Second);
        let t_last = self.buffer[self.size - 1].epoch;

        let azim_mid = self.buffer[midpoint].azimuth;
        let elev_mid = self.buffer[midpoint].elevation;

        let mut fitted = FittedData::default();

        fitted.sv = self.sv;
        fitted.first_t = t0;
        fitted.duration = t_last - t0;
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
        let fit = polyfit(
            &x_t,
            self.buffer
                .iter()
                .map(|data| data.refsv)
                .collect::<Vec<_>>()
                .as_slice(),
            1,
        )
        .or(Err(FitError::LinearRegressionFailure))?;

        let (srsv, srsv_b) = (fit[0], fit[1]);
        let refsv = srsv * t_mid_s + srsv_b;

        // REFSYS
        let fit = polyfit(
            &x_t,
            self.buffer
                .iter()
                .map(|data| data.refsys)
                .collect::<Vec<_>>()
                .as_slice(),
            1,
        )
        .or(Err(FitError::LinearRegressionFailure))?;

        let (srsys, srsys_b) = (fit[0], fit[1]);
        let refsys_fit = srsys * t_mid_s + srsys_b;

        // DSG
        let mut dsg = 0.0_f64;
        for obs in self.buffer.iter() {
            dsg += (obs.refsys - refsys_fit).powi(2);
        }
        dsg /= self.size as f64;
        dsg = dsg.sqrt();

        // MDTR
        let fit = polyfit(
            &x_t,
            self.buffer
                .iter()
                .map(|data| data.mdtr)
                .collect::<Vec<_>>()
                .as_slice(),
            1,
        )
        .or(Err(FitError::LinearRegressionFailure))?;

        let (smdt, smdt_b) = (fit[0], fit[1]);
        let mdtr = smdt * t_mid_s + smdt_b;

        // MDIO
        let fit = polyfit(
            &x_t,
            self.buffer
                .iter()
                .map(|data| data.mdio)
                .collect::<Vec<_>>()
                .as_slice(),
            1,
        )
        .or(Err(FitError::LinearRegressionFailure))?;

        let (smdi, smdi_b) = (fit[0], fit[1]);
        let mdio = smdi * t_mid_s + smdi_b;

        // MSIO
        let msio = self
            .buffer
            .iter()
            .filter_map(|data| {
                if let Some(msio) = data.msio {
                    Some(msio)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let msio_len = msio.len();

        if msio_len > 0 {
            let fit = polyfit(&x_t, &msio, 1).or(Err(FitError::LinearRegressionFailure))?;

            let (smsi, smsi_b) = (fit[0], fit[1]);
            let msio_fit = smsi * t_mid_s + smsi_b;

            // ISG
            let mut isg = 0.0_f64;
            for i in 0..msio_len {
                isg += (msio_fit - msio[i]).powi(2);
            }
            isg /= self.size as f64;
            isg = isg.sqrt();

            fitted.isg = Some(isg);
            fitted.msio_s = Some(msio_fit);
            fitted.smsi_s_s = Some(smsi);
        }

        fitted.srsv_s_s = srsv;
        fitted.refsv_s = refsv;
        fitted.srsys_s_s = srsys;
        fitted.refsys_s = refsys_fit;
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

    fn has_msio(&self) -> bool {
        self.buffer
            .iter()
            .filter(|data| data.msio.is_some())
            .count()
            > 0
    }
}

#[cfg(test)]
mod test {
    use crate::prelude::{Duration, Epoch, Observation, SVTracker, SV};
    use std::str::FromStr;

    #[test]
    fn tracker_no_gap_x3() {
        let g01 = SV::from_str("G01").unwrap();
        let mut tracker = SVTracker::new(g01);

        for obs in [
            Observation {
                epoch: Epoch::from_str("2020-01-01T00:00:00 UTC").unwrap(),
                refsv: 1.0,
                refsys: 2.0,
                mdtr: 3.0,
                mdio: 4.0,
                msio: None,
                elevation: 6.0,
                azimuth: 7.0,
            },
            Observation {
                epoch: Epoch::from_str("2020-01-01T00:00:30 UTC").unwrap(),
                refsv: 1.1,
                refsys: 2.1,
                mdtr: 3.1,
                mdio: 4.1,
                msio: None,
                elevation: 6.1,
                azimuth: 7.1,
            },
        ] {
            tracker.new_observation(obs);
        }

        assert!(tracker.fit().is_err());

        tracker.new_observation(Observation {
            epoch: Epoch::from_str("2020-01-01T00:01:00 UTC").unwrap(),
            refsv: 1.2,
            refsys: 2.2,
            mdtr: 3.2,
            mdio: 4.2,
            msio: None,
            elevation: 6.2,
            azimuth: 7.2,
        });

        let fitted = tracker.fit().unwrap();

        assert_eq!(fitted.sv, g01);
        assert_eq!(fitted.duration, Duration::from_seconds(60.0));

        assert_eq!(
            fitted.first_t,
            Epoch::from_str("2020-01-01T00:00:00 UTC").unwrap()
        );

        assert_eq!(
            fitted.midtrack,
            Epoch::from_str("2020-01-01T00:00:30 UTC").unwrap()
        );

        assert_eq!(fitted.elevation_deg, 6.1);
        assert_eq!(fitted.azimuth_deg, 7.1);
    }

    #[test]
    fn tracker_no_gap_x4() {
        let g01 = SV::from_str("G01").unwrap();
        let mut tracker = SVTracker::new(g01);

        for obs in [
            Observation {
                epoch: Epoch::from_str("2020-01-01T00:00:00 UTC").unwrap(),
                refsv: 1.0,
                refsys: 2.0,
                mdtr: 3.0,
                mdio: 4.0,
                msio: None,
                elevation: 6.0,
                azimuth: 7.0,
            },
            Observation {
                epoch: Epoch::from_str("2020-01-01T00:00:30 UTC").unwrap(),
                refsv: 1.1,
                refsys: 2.1,
                mdtr: 3.1,
                mdio: 4.1,
                msio: None,
                elevation: 6.1,
                azimuth: 7.1,
            },
        ] {
            tracker.new_observation(obs);
        }

        assert!(tracker.fit().is_err());

        tracker.new_observation(Observation {
            epoch: Epoch::from_str("2020-01-01T00:01:00 UTC").unwrap(),
            refsv: 1.2,
            refsys: 2.2,
            mdtr: 3.2,
            mdio: 4.2,
            msio: None,
            elevation: 6.2,
            azimuth: 7.2,
        });

        let fitted = tracker.fit().unwrap();

        assert_eq!(fitted.sv, g01);
        assert_eq!(fitted.duration, Duration::from_seconds(60.0));

        assert_eq!(
            fitted.first_t,
            Epoch::from_str("2020-01-01T00:00:00 UTC").unwrap()
        );

        assert_eq!(
            fitted.midtrack,
            Epoch::from_str("2020-01-01T00:00:30 UTC").unwrap()
        );

        assert_eq!(fitted.elevation_deg, 6.1);
        assert_eq!(fitted.azimuth_deg, 7.1);
    }

    #[test]
    fn tracker_30s_15s_ok() {
        let g01 = SV::from_str("G01").unwrap();
        let dt_30s = Duration::from_str("30 s").unwrap();

        let mut tracker = SVTracker::new(g01).with_gap_tolerance(dt_30s);

        for obs in [
            Observation {
                epoch: Epoch::from_str("2020-01-01T00:00:00 UTC").unwrap(),
                refsv: 1.0,
                refsys: 2.0,
                mdtr: 3.0,
                mdio: 4.0,
                msio: None,
                elevation: 6.0,
                azimuth: 7.0,
            },
            Observation {
                epoch: Epoch::from_str("2020-01-01T00:00:15 UTC").unwrap(),
                refsv: 1.1,
                refsys: 2.1,
                mdtr: 3.1,
                mdio: 4.1,
                msio: None,
                elevation: 6.1,
                azimuth: 7.1,
            },
            Observation {
                epoch: Epoch::from_str("2020-01-01T00:00:30 UTC").unwrap(),
                refsv: 1.1,
                refsys: 2.1,
                mdtr: 3.1,
                mdio: 4.1,
                msio: None,
                elevation: 6.1,
                azimuth: 7.1,
            },
            Observation {
                epoch: Epoch::from_str("2020-01-01T00:00:45 UTC").unwrap(),
                refsv: 1.1,
                refsys: 2.1,
                mdtr: 3.1,
                mdio: 4.1,
                msio: None,
                elevation: 6.1,
                azimuth: 7.1,
            },
        ] {
            tracker.new_observation(obs);
        }

        assert!(tracker.fit().is_ok());
    }

    #[test]
    fn tracker_30s_30s_ok() {
        let g01 = SV::from_str("G01").unwrap();
        let dt_30s = Duration::from_str("30 s").unwrap();

        let mut tracker = SVTracker::new(g01).with_gap_tolerance(dt_30s);

        for obs in [
            Observation {
                epoch: Epoch::from_str("2020-01-01T00:00:00 UTC").unwrap(),
                refsv: 1.0,
                refsys: 2.0,
                mdtr: 3.0,
                mdio: 4.0,
                msio: None,
                elevation: 6.0,
                azimuth: 7.0,
            },
            Observation {
                epoch: Epoch::from_str("2020-01-01T00:00:30 UTC").unwrap(),
                refsv: 1.1,
                refsys: 2.1,
                mdtr: 3.1,
                mdio: 4.1,
                msio: None,
                elevation: 6.1,
                azimuth: 7.1,
            },
            Observation {
                epoch: Epoch::from_str("2020-01-01T00:01:00 UTC").unwrap(),
                refsv: 1.1,
                refsys: 2.1,
                mdtr: 3.1,
                mdio: 4.1,
                msio: None,
                elevation: 6.1,
                azimuth: 7.1,
            },
            Observation {
                epoch: Epoch::from_str("2020-01-01T00:01:30 UTC").unwrap(),
                refsv: 1.1,
                refsys: 2.1,
                mdtr: 3.1,
                mdio: 4.1,
                msio: None,
                elevation: 6.1,
                azimuth: 7.1,
            },
        ] {
            tracker.new_observation(obs);
        }

        assert!(tracker.fit().is_ok());

        for obs in [
            Observation {
                epoch: Epoch::from_str("2020-01-01T00:00:00 UTC").unwrap(),
                refsv: 1.0,
                refsys: 2.0,
                mdtr: 3.0,
                mdio: 4.0,
                msio: None,
                elevation: 6.0,
                azimuth: 7.0,
            },
            Observation {
                epoch: Epoch::from_str("2020-01-01T00:00:30 UTC").unwrap(),
                refsv: 1.1,
                refsys: 2.1,
                mdtr: 3.1,
                mdio: 4.1,
                msio: None,
                elevation: 6.1,
                azimuth: 7.1,
            },
            Observation {
                epoch: Epoch::from_str("2020-01-01T00:01:00 UTC").unwrap(),
                refsv: 1.1,
                refsys: 2.1,
                mdtr: 3.1,
                mdio: 4.1,
                msio: None,
                elevation: 6.1,
                azimuth: 7.1,
            },
            Observation {
                epoch: Epoch::from_str("2020-01-01T00:01:15 UTC").unwrap(),
                refsv: 1.1,
                refsys: 2.1,
                mdtr: 3.1,
                mdio: 4.1,
                msio: None,
                elevation: 6.1,
                azimuth: 7.1,
            },
        ] {
            tracker.new_observation(obs);
        }

        assert!(tracker.fit().is_ok());
    }

    #[test]
    fn tracker_30s_nok() {
        let g01 = SV::from_str("G01").unwrap();
        let dt_30s = Duration::from_str("30 s").unwrap();

        let mut tracker = SVTracker::new(g01).with_gap_tolerance(dt_30s);

        for obs in [
            Observation {
                epoch: Epoch::from_str("2020-01-01T00:00:00 UTC").unwrap(),
                refsv: 1.0,
                refsys: 2.0,
                mdtr: 3.0,
                mdio: 4.0,
                msio: None,
                elevation: 6.0,
                azimuth: 7.0,
            },
            Observation {
                epoch: Epoch::from_str("2020-01-01T00:00:30 UTC").unwrap(),
                refsv: 1.1,
                refsys: 2.1,
                mdtr: 3.1,
                mdio: 4.1,
                msio: None,
                elevation: 6.1,
                azimuth: 7.1,
            },
            Observation {
                epoch: Epoch::from_str("2020-01-01T00:01:00 UTC").unwrap(),
                refsv: 1.1,
                refsys: 2.1,
                mdtr: 3.1,
                mdio: 4.1,
                msio: None,
                elevation: 6.1,
                azimuth: 7.1,
            },
            Observation {
                epoch: Epoch::from_str("2020-01-01T00:01:31 UTC").unwrap(),
                refsv: 1.1,
                refsys: 2.1,
                mdtr: 3.1,
                mdio: 4.1,
                msio: None,
                elevation: 6.1,
                azimuth: 7.1,
            },
        ] {
            tracker.new_observation(obs);
        }

        assert!(tracker.fit().is_err());
    }
}
