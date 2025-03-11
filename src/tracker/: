mod fit;
// mod fitted;

pub use fit::{FitError, Observation, SVTracker};
// pub use fitted::FittedData;

use crate::prelude::SV;
use std::collections::HashMap;

/// [SkyTracker] is used to track all Satellite vehicles
/// in sight during a common view period and eventually collect CGGTTS.
#[derive(Default, Debug, Clone)]
pub struct SkyTracker {
    /// Internal buffer
    sv_trackers: HashMap<SV, SVTracker>,
}
