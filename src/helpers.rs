use crate::traits::InPlaceEndiannessConversion;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const VERSION_MAJOR: &str = env!("CARGO_PKG_VERSION_MAJOR");
pub const VERSION_MINOR: &str = env!("CARGO_PKG_VERSION_MINOR");
pub const VERSION_PATCH: &str = env!("CARGO_PKG_VERSION_PATCH");

/// Calculate the numerical version.
///
/// | Major | Minor |
/// | ----- | ----- |
/// | 3     | 20    |
///
/// -> 321
pub fn calculate_version() -> u32 {
    let major: u32 = VERSION_MAJOR.parse().unwrap_or_else(|_| {
        panic!(
            "Failed to parse Major Version `{}` into u32.",
            VERSION_MAJOR
        )
    });
    let minor: u32 = VERSION_MINOR.parse().unwrap_or_else(|_| {
        panic!(
            "Failed to parse Minor Version `{}` into u32.",
            VERSION_MINOR
        )
    });

    let places = (minor as f64).log(10.0).floor() as u32 + 1;
    major * 10_u32.pow(places) + minor
}

impl InPlaceEndiannessConversion for Vec<u8> {
    fn to_be(&mut self) {
        for v in self.iter_mut() {
            *v = v.to_be();
        }
    }

    fn to_le(&mut self) {
        for v in self.iter_mut() {
            *v = v.to_le();
        }
    }
}

impl InPlaceEndiannessConversion for [u8; 4] {
    fn to_be(&mut self) {
        for v in self.iter_mut() {
            *v = v.to_be();
        }
    }

    fn to_le(&mut self) {
        for v in self.iter_mut() {
            *v = v.to_le();
        }
    }
}
