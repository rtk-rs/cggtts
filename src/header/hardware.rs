/// [Hardware] is used to describe a piece of equipment.
/// Usually the GNSS receiver.
#[derive(Clone, PartialEq, Debug, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Hardware {
    /// Model type.
    pub model: String,
    /// Manufacturer
    pub manufacturer: String,
    /// Readable serial number.
    pub serial_number: String,
    /// Year of production or release
    pub year: u16,
    /// Software or firmware version
    pub release: String,
}

impl Hardware {
    /// Define a new [Hardware] with desired model name
    pub fn with_model(&self, model: &str) -> Self {
        let mut s = self.clone();
        s.model = model.to_string();
        s
    }

    /// Define a new [Hardware] with desired manufacturer
    pub fn with_manufacturer(&self, manufacturer: &str) -> Self {
        let mut s = self.clone();
        s.manufacturer = manufacturer.to_string();
        s
    }

    /// Define a new [Hardware] with desired serial number
    pub fn with_serial_number(&self, serial_number: &str) -> Self {
        let mut s = self.clone();
        s.serial_number = serial_number.to_string();
        s
    }

    /// Define a new [Hardware] with desired year of production
    /// or release.
    pub fn with_release_year(&self, y: u16) -> Self {
        let mut s = self.clone();
        s.year = y;
        s
    }

    /// Define a new [Hardware] with desired firmware or
    /// software release version.
    pub fn with_release_version(&self, version: &str) -> Self {
        let mut s = self.clone();
        s.release = version.to_string();
        s
    }
}

impl std::fmt::LowerHex for Hardware {
    /// Formats [Hardware] as used in a CGGTTS header.
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.write_str(&format!(
            "{} {} {} {} {}",
            self.manufacturer, self.model, self.serial_number, self.year, self.release
        ))
    }
}

#[cfg(test)]
mod test {
    use crate::header::hardware::Hardware;

    #[test]
    fn hardware_parsing() {
        let hw = Hardware::default()
            .with_manufacturer("TEST")
            .with_model("MODEL")
            .with_release_year(2024)
            .with_serial_number("1234")
            .with_release_version("v00");

        assert_eq!(format!("{:x}", hw), "TEST MODEL 1234 2024 v00");
    }
}
