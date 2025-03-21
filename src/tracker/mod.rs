mod fit;
mod fitted;

pub use fit::{FitError, Observation, SVTracker};
pub use fitted::FittedData;

use crate::prelude::{Duration, SV};
use std::collections::HashMap;

/// [SkyTracker] is used to track all Satellite vehicles
/// in sight during a common view period and eventually collect CGGTTS.
#[derive(Default, Debug, Clone)]
pub struct SkyTracker {
    /// Internal buffer
    trackers: HashMap<SV, SVTracker>,
    /// Gap tolerance
    gap_tolerance: Option<Duration>,
}

impl SkyTracker {
    /// Allocate new [SkyTracker]
    pub fn new() -> Self {
        Self {
            trackers: HashMap::with_capacity(8),
            gap_tolerance: None,
        }
    }

    /// Define a [SkyTracker] with desired observation gap tolerance.
    pub fn with_gap_tolerance(&self, tolerance: Duration) -> Self {
        let mut s = self.clone();
        s.gap_tolerance = Some(tolerance);
        s
    }

    /// Provide new [Observation] for that particular satellite.
    pub fn new_observation(&mut self, satellite: SV, data: Observation) {
        if let Some(tracker) = self.trackers.get_mut(&satellite) {
            tracker.new_observation(data);
        } else {
            let mut new = SVTracker::new(satellite);
            if let Some(tolerance) = self.gap_tolerance {
                new = new.with_gap_tolerance(tolerance);
            }
            new.new_observation(data);
            self.trackers.insert(satellite, new);
        }
    }

    /// Attempt new satellite track fitting.
    /// ## Input
    /// - satellite: [SV] that must have been tracked.
    /// That tracking duration might have to respect external requirements.
    /// ## Output
    /// - fitted: [FittedData] that you can turn into a valid
    /// CGGTTS track if you provide a little more information.
    pub fn track_fit(&mut self, satellite: SV) -> Result<FittedData, FitError> {
        if let Some(sv) = self.trackers.get_mut(&satellite) {
            sv.fit()
        } else {
            Err(FitError::UnknownSatellite)
        }
    }
}
