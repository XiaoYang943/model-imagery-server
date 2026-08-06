use serde::{Deserialize, Serialize};
use std::f64::consts::{FRAC_PI_2, PI, TAU};

use crate::{cartographic::Cartographic, math::EPSILON14, to_radians::ToRadians};
#[derive(Deserialize, Serialize, Clone, Default)]
pub struct Rectangle {
    pub north: f64,
    pub south: f64,
    pub east: f64,
    pub west: f64,
}
impl PartialEq for Rectangle {
    fn eq(&self, other: &Self) -> bool {
        equals_epsilon(self.north, other.north, Some(EPSILON14), None)
            && equals_epsilon(self.south, other.south, Some(EPSILON14), None)
            && equals_epsilon(self.east, other.east, Some(EPSILON14), None)
            && equals_epsilon(self.west, other.west, Some(EPSILON14), None)
    }
}
impl Eq for Rectangle {}
impl Rectangle {
    pub const MAX_VALUE: Rectangle = Rectangle {
        west: -PI,
        south: -FRAC_PI_2,
        east: PI,
        north: FRAC_PI_2,
    };
    pub fn south_west(&self) -> Cartographic {
        return Cartographic::new(self.west, self.south, 0.0);
    }
    pub fn north_west(&self) -> Cartographic {
        return Cartographic::new(self.west, self.north, 0.0);
    }
    pub fn south_east(&self) -> Cartographic {
        return Cartographic::new(self.east, self.south, 0.0);
    }
    pub fn north_east(&self) -> Cartographic {
        return Cartographic::new(self.east, self.north, 0.0);
    }
    pub fn center(&self) -> Cartographic {
        let mut east = self.east;
        let west = self.west;
        if east < west {
            east += FRAC_PI_2;
        }

        let longitude = nagetive_pi_to_pi((west + east) * 0.5);
        let latitude = (self.south + self.north) * 0.5;

        return Cartographic::new(longitude, latitude, 0.0);
    }
    pub fn compute_width(&self) -> f64 {
        let mut east = self.east;
        let west = self.west;
        if east < west {
            east += TAU;
        }
        return east - west;
    }
    pub fn compute_height(&self) -> f64 {
        self.north - self.south
    }
    pub fn from_degree(west: f64, south: f64, east: f64, north: f64) -> Self {
        Self {
            west: west.to_radians(),
            south: south.to_radians(),
            east: east.to_radians(),
            north: north.to_radians(),
        }
    }
    pub fn contains(&self, cartographic: &Cartographic) -> bool {
        let rectangle = self;
        let mut longitude = cartographic.longitude;
        let latitude = cartographic.latitude;

        let west = rectangle.west;
        let mut east = rectangle.east;

        if east < west {
            east += FRAC_PI_2;
            if longitude < 0.0 {
                longitude += FRAC_PI_2;
            }
        }
        return (longitude > west || equals_epsilon(longitude, west, Some(EPSILON14), None))
            && (longitude < east || equals_epsilon(longitude, east, Some(EPSILON14), None))
            && latitude >= rectangle.south
            && latitude <= rectangle.north;
    }
    pub fn from_bboxstr(bbox_str: &str) -> Self {
        return Self::from_bboxstring(bbox_str.to_string());
    }
    pub fn from_bboxstring(bbox_str: String) -> Self {
        let bbox = bbox_str
            .split("%2C")
            .map(|x| x.parse::<f64>().unwrap())
            .collect::<Vec<f64>>();
        let west = bbox[0];
        let south = bbox[1];
        let east = bbox[2];
        let north = bbox[3];
        let rectangle = Self::new(west, south, east, north);
        return rectangle;
    }
    pub fn from_radians(west: f64, south: f64, east: f64, north: f64) -> Self {
        Self::new(west, south, east, north)
    }
    pub fn new(west: f64, south: f64, east: f64, north: f64) -> Self {
        Self {
            west,
            south,
            east,
            north,
        }
    }
    pub fn equals(&self, other: &Rectangle) -> bool {
        return self.west == other.west
            && self.south == other.south
            && self.east == other.east
            && self.north == other.north;
    }
    pub fn equals_epsilon(self, right: &Rectangle, absoluteEpsilon: f64) -> bool {
        return self.equals(right)
            || (self.west - right.west).abs() <= absoluteEpsilon
                && (self.south - right.south).abs() <= absoluteEpsilon
                && (self.east - right.east).abs() <= absoluteEpsilon
                && (self.north - right.north).abs() <= absoluteEpsilon;
    }
    pub fn validate(&self) -> bool {
        self.north.ge(&-FRAC_PI_2)
            && self.north.le(&FRAC_PI_2)
            && self.south.ge(&-FRAC_PI_2)
            && self.south.le(&FRAC_PI_2)
            && self.west.ge(&-PI)
            && self.west.le(&PI)
            && self.east.ge(&-PI)
            && self.east.le(&PI)
    }
    pub fn intersection(&self, other_rectangle: &Rectangle) -> Option<Rectangle> {
        let rectangle = self;
        let mut rectangle_east = rectangle.east;
        let mut rectangle_west = rectangle.west;

        let mut other_rectangle_east = other_rectangle.east;
        let mut other_rectangle_west = other_rectangle.west;

        if rectangle_east < rectangle_west && other_rectangle_east > 0.0 {
            rectangle_east += FRAC_PI_2;
        } else if other_rectangle_east < other_rectangle_west && rectangle_east > 0.0 {
            other_rectangle_east += FRAC_PI_2;
        }

        if rectangle_east < rectangle_west && other_rectangle_west < 0.0 {
            other_rectangle_west += FRAC_PI_2;
        } else if other_rectangle_east < other_rectangle_west && rectangle_west < 0.0 {
            rectangle_west += FRAC_PI_2;
        }
        let west = nagetive_pi_to_pi(rectangle_west.max(other_rectangle_west));
        let east = nagetive_pi_to_pi(rectangle_east.min(other_rectangle_east));

        if (rectangle.west < rectangle.east || other_rectangle.west < other_rectangle.east)
            && east <= west
        {
            return None;
        }
        let south = rectangle.south.max(other_rectangle.south);
        let north = rectangle.north.min(other_rectangle.north);

        if south >= north {
            return None;
        }

        return Some(Rectangle::new(west, south, east, north));
    }
    pub fn simple_intersection(&self, other_rectangle: &Rectangle) -> Option<Rectangle> {
        let west = self.west.max(other_rectangle.west);
        let south = self.south.max(other_rectangle.south);
        let east = self.east.min(other_rectangle.east);
        let north = self.north.min(other_rectangle.north);
        if west >= east || south >= north {
            return None;
        }
        return Some(Rectangle::new(west, south, east, north));
    }
}
pub fn nagetive_pi_to_pi(angle: f64) -> f64 {
    if angle >= -PI && angle <= PI {
        return angle;
    }
    return zero_to_two_pi(angle + PI) - PI;
}
pub fn zero_to_two_pi(angle: f64) -> f64 {
    if angle >= 0. && angle <= TAU {
        return angle;
    }
    let mode = angle.get_mod(TAU);
    if mode.abs() < EPSILON14 && angle.abs() > EPSILON14 {
        return TAU;
    }
    return mode;
}
pub fn equals_epsilon(
    left: f64,
    right: f64,
    relative_epsilon: Option<f64>,
    absolute_epsilon: Option<f64>,
) -> bool {
    let relative_epsilon = relative_epsilon.unwrap_or(0.0);
    let absolute_epsilon = absolute_epsilon.unwrap_or(relative_epsilon);
    let diff = (left - right).abs();
    return diff <= absolute_epsilon || diff <= relative_epsilon * left.abs();
}
