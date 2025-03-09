/// GNSS Receiver description
#[derive(Clone, PartialEq, Debug, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Rcvr {
    /// Manufacturer of this hardware
    pub manufacturer: String,
    /// Type of receiver
    pub model: String,
    /// Receiver's serial number
    pub serial_number: String,
    /// Receiver manufacturing year
    pub year: u16,
    /// Receiver software revision number
    pub release: String,
}

impl Rcvr {
    pub fn manufacturer(&self, man: &str) -> Self {
        let mut s = self.clone();
        s.manufacturer = man.to_string();
        s
    }
    pub fn receiver(&self, rcvr: &str) -> Self {
        let mut s = self.clone();
        s.model = rcvr.to_string();
        s
    }
    pub fn serial_number(&self, sn: &str) -> Self {
        let mut s = self.clone();
        s.serial_number = sn.to_string();
        s
    }
    pub fn year(&self, y: u16) -> Self {
        let mut s = self.clone();
        s.year = y;
        s
    }
    pub fn release(&self, release: &str) -> Self {
        let mut s = self.clone();
        s.release = release.to_string();
        s
    }
}

impl std::fmt::UpperHex for Rcvr {
    /*
     * Formats Self as in file Header
     */
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.write_str(&format!(
            "{} {} {} {} {}",
            self.manufacturer, self.model, self.serial_number, self.year, self.release
        ))
    }
}
