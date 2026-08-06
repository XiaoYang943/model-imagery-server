#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub struct Cartographic {
    pub longitude: f64,
    pub latitude: f64,
    pub height: f64,
}
impl Cartographic {
    pub fn new(longitude: f64, latitude: f64, height: f64) -> Self {
        Cartographic::from_radians(longitude, latitude, height)
    }
    pub fn from_radians(longitude: f64, latitude: f64, height: f64) -> Self {
        Cartographic {
            longitude: longitude,
            latitude: latitude,
            height,
        }
    }
    pub fn from_degrees(longitude: f64, latitude: f64, height: f64) -> Self {
        Cartographic {
            longitude: longitude.to_radians(),
            latitude: latitude.to_radians(),
            height,
        }
    }
    pub fn to_radians(&self) -> Self {
        Cartographic {
            longitude: self.longitude.to_radians(),
            latitude: self.latitude.to_radians(),
            height: self.height,
        }
    }
    pub fn to_degrees(&self) -> Self {
        Cartographic {
            longitude: self.longitude.to_degrees(),
            latitude: self.latitude.to_degrees(),
            height: self.height,
        }
    }
    pub fn equals(&self, right: &Cartographic) -> bool {
        return self.longitude == right.longitude
            && self.latitude == right.latitude
            && self.height == right.height;
    }
    pub fn equals_epsilon(self, right: Cartographic, epsilon: f64) -> bool {
        return (self.longitude - right.longitude).abs() <= epsilon
            && (self.latitude - right.latitude).abs() <= epsilon
            && (self.height - right.height).abs() <= epsilon;
    }
    pub const ZERO: Cartographic = Cartographic {
        longitude: 0.0,
        latitude: 0.0,
        height: 0.0,
    };
}
impl ToString for Cartographic {
    fn to_string(&self) -> String {
        return format!(
            "Cartographic {{ longitude: {}, latitude: {}, height: {} }}",
            self.longitude, self.latitude, self.height
        );
    }
}
