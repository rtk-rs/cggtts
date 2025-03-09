//! Set of methods to compute CGGTTS data and
//! produce tracks

/// Speed of light in [m/s]
const SPEED_OF_LIGHT: f64 = 300_000_000.0_f64;

/// SAGNAC correction associated with Earth rotation
const SAGNAC_CORRECTION: f64 = 0.0_f64;

/// Refractivity Index @ seal level
const NS: f64 = 324.8_f64;

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Vec3D {
    x: f64,
    y: f64,
    z: f64,
}

impl Default for Vec3D {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }
}

impl Vec3D {
    pub fn norm(&self) -> f64 {
        (self.x.powf(2.0) + self.y.powf(2.0) + self.z.powf(2.0)).sqrt()
    }
}

impl std::ops::Sub<Vec3D> for Vec3D {
    type Output = Vec3D;
    fn sub(self, rhs: Vec3D) -> Vec3D {
        Vec3D {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

pub enum Policy {
    /// Simple straight forward processing,
    /// see [p6: Data processing paragraph]
    Simple,
    /// Use n tap smoothing.   
    /// This feature is not needed when using a
    /// modern GNSS receiver
    Smoothing(u32),
}

pub struct Params {
    /// Raw measurements
    pr: f64,
    /// Current elevation [°]
    e: f64,
    /// Current altitude [km]
    h: f64,
    /// Current Sv vector
    x_sat: Vec3D,
    /// Broadcast satellite clock offset
    t_sat: f64,
    /// reference timescale
    t_ref: f64,
    /// Current Rcvr vector
    x_rec: Vec3D,
    /// Carrier dependent delay
    delay: f64,
    /// RF delay
    rf_delay: f64,
    /// REF delay
    ref_delay: f64,
    /// Group delay
    grp_delay: f64,
}

/// Computes dn constant
fn dn() -> f64 {
    -7.32 * (0.005577 * NS).exp()
}

fn nslog() -> f64 {
    (NS + dn() / 105.0).ln()
}

/// Computes R_h quantity [eq(8)] Tropospheric delay at zenith,
/// from space vehicule altitude in [km]
fn r_h(altitude: f64) -> f64 {
    let dn = dn();
    let nslog = nslog();
    if altitude < 1.0 {
        (2162.0 + NS * (1.0 - altitude) + 0.5 * dn * (1.0 - altitude.powf(2.0))) * 10E-3
            / SPEED_OF_LIGHT
    } else {
        let frac = (NS + dn) / nslog;
        let e_1 = (-nslog).exp();
        let e_2 = (0.125 * (1.0 - altitude) * nslog).exp();
        (732.0 - (8.0 * frac * (e_1 - e_2))) * 10E-3 / SPEED_OF_LIGHT
    }
}

/// Computes f_e
/// - e: elevation [°]
fn f_e(e: f64) -> f64 {
    1.0 / (e.sin() + 0.00143 / (e.tan() + 0.0455))
}

/// Relativistic delay
fn dt_rel() -> f64 {
    0.0
}

/// Ionospheric delay
fn dt_iono() -> f64 {
    0.0
}

/// Inputs:
/// - pr: raw measurement
/// - x_sat: current Sv vector
/// - x_rec: rcvr estimate
/// - h: altitude in km
/// - e: elevation in °
///
/// Returns
/// - dt_sat : [eq(2)]
/// - dt_ref : [eq(7)]
/// - dt_tropo : [eq(6)]
/// - dt_iono : [eq(5)]
pub fn process(data: Params) -> (f64, f64, f64) {
    // compensation
    let p = data.pr - SPEED_OF_LIGHT * (data.delay + data.rf_delay - data.ref_delay);
    let fe = f_e(data.e);
    let rh = r_h(data.h);
    let dt_tropo = fe * rh;
    let d_tclk_tsat = 1.0 / SPEED_OF_LIGHT
        * (p - (data.x_sat - data.x_rec).norm() - SAGNAC_CORRECTION)
        + dt_rel()
        - dt_iono()
        - dt_tropo
        - data.grp_delay;
    let d_tclk_tref = d_tclk_tsat + data.t_sat - data.t_ref;
    (d_tclk_tsat, d_tclk_tref, dt_tropo)
}

/*
    /// Computes f(elevation) [eq(7)] neded by NATO hydrostatic model
    fn f_elev (elevation: f64) -> {
        1.0 / (0.000143 / (e.tan() +0.0455) + e.sin())
    }

    /// Computes delta troposphere using NATO hydrostatic model [eq(6)]
    fn d_tropo (elevation: f64, altitude: f64) -> {
        f_elev(elevation) * R_h(altitude)
    }

    /// Call this once per cycle
    /// to process a new symbol.
    /// Compensations & computations are then performed internally
    ///
    /// # Input:
    /// - pr: raw pseudo range
    /// - x_sat: 3D vehicule position estimate in IRTF system (must be IRTF!)
    /// - x_rec: 3D receiver position estimate in IRTF system (must be IRTF!)
    /// - dt_rel_corr : relativistic clock correction for space vehicule
    /// redshift along its orbit
    /// - iono_dt: carrier dependent ionospheric delay
    /// - dtropo: troposphere induced delay
    /// - grp_delay: broadcast group delay
    pub fn process (&mut self, pr: f64) {
        let p = symbol - SPEED_OF_LIGHT * (self.delay.value() + self.cab_delay - self.ref_delay);
        self.buffer.push(p)
    }

    pub fn run (&mut self, elevation: f64,
            x_sat: (f64,f64,f64), x_rec: (f64,f64,f64), dt_rel_corr: f64,
                iono_dt: f64, dtropo: f64, grp_delay: f64)
    {
    }

    pub fn next()
        self.buffer.push(p);
        dt =
}

/*
pub struct Scheduler {
    now: chrono::NaiveDateTime;
    pub trk_duration: std::time::Duration,
}

impl Iterator for Scheduler {
    type Item = bool;
}

pub struct Scheduler {
    /// TrackGen policy
    pub processing : Policy,
    /// Current work date
    day: chrono::NaiveDate,
    /// should match BIPM recommendations,
    /// but custom values can be used (shortest tracking in particular)
    trk_duration: std::time::Duration,
    /// Scheduled events for today
    events: Vec<chrono::NaiveTime>,
    /// System delays
    /// Only single frequency generation supported @ the moment
    delay: delay::SystemDelay,
    /// Internal data buffer
    p : Vec<f64>,
}

impl Scheduler {
    /// Builds a new measurement scheduler,
    /// Inputs:
    ///   day: optionnal work day, otherwise uses `now()`
    ///
    ///   trk_duration: optionnal custom tracking duration,
    ///   defaults to `BIPM_RECOMMENDED_TRACKING`
    pub fn new (day: chrono::NaiveDate, trk_duration: std::time::Duration) -> Self {
        let day = day.unwrap_or(chrono::Utc::now().naive_utc().date());
        let duration = trk_duration.unwrap_or(BIPM_RECOMMENDED_TRACKING);
        //let events = Scheduler::events(day, duration);
        Self {
            day,
            trk_duration,
            events: Vec::new(),
        }
    }

/*
    /// Returns scheduled measurements for given day,
    /// if date is not provided, we use now()
    pub fn scheduled_events (&self, date: Option<chrono::NaiveDate>) -> Vec<chrono::NaiveDateTime> {
        let mut res : Vec<chrono::NaiveDateTime> = Vec::new();

    /// Call this once day has changed to reset internal FSM
    pub fn new_day (&mut self) {
        //self.day = chrono::Utc::now().naive_utc().date();
        //self.events = Scheduler::events(self.day, self.duration);
    }

    /// Updates tracking duration to new value
    pub fn update_trk_duration (&mut self, new: std::time::Duration) {
        self.duration = new
    }

    /// Returns scheduled measurements for given day,
    /// if date is not provided, we use now()
    pub fn events (&self, date: Option<chrono::NaiveDate>) -> Vec<chrono::NaiveDateTime> {
        /*let mut res : Vec<chrono::NaiveDateTime> = Vec::new();
>>>>>>> Stashed changes
        let mjd_ref = ModifiedJulianDay::new(REFERENCE_MJD).inner();
        let date = date.unwrap_or(chrono::Utc::now().naive_utc().date());
        let mjd = ModifiedJulianDay::from(date).inner();
        for i in 1..self.tracks_in_24h()-1 {
            let offset = Scheduler::time_ref(self.n) as i32 - 4*(mjd_ref - mjd)/60;
            if offset > 0 {
                let h = offset / 3600;
                let m = (offset - h*3600)/60;
                let s = offset -h*3600 - m*60;
                res.push(
                    chrono::NaiveDate::from_ymd(date.year(), date.month(), date.day())
                        .and_hms(h as u32 ,m as u32,s as u32));
            }
        }
        res*/
        Vec::new()
    }
*/
    /// Returns duration (time interval) between given date
    /// and next scheduled measurement
    pub fn time_to_next (&self, datetime: chrono::NaiveDateTime) -> std::time::Duration {
        //let offset = Scheduler::time_ref(self.n);
        std::time::Duration::from_secs(10)
    }

    /// Returns offset in seconds during the course of `MJD_REF`
    /// reference Modified Julian Day (defined in standards),
    /// for given nth observation within that day.
    ///
    /// Input:
    ///  - observation: observation counter
    fn time_ref (observation: u32) -> u32 {
        60 * 2 + (observation -1)*16*60
    }

    /// Returns number of measurements to perform within 24hours
    fn tracks_in_24h (&self) -> u64 {
        24 * 3600 / self.duration.as_secs()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::{NaiveDate, NaiveDateTime};
    #[test]
    fn test_scheduler_basic() {
        let t0 = chrono::NaiveDate::from_ymd(2022, 07, 05)
            .and_hms(00, 00, 00);
        let scheduler = Scheduler::new(Some(t0), None);
    }
}*/
*/
