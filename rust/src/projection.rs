use glam::DVec3;

use crate::{cartographic::Cartographic, ellipsoid::Ellipsoid};

pub trait Projection {
    type Output;
    fn project(&self, coord: &Cartographic) -> DVec3;
    fn un_project(&self, vec: &DVec3) -> Cartographic;
    fn from_ellipsoid(ellipsoid: &Ellipsoid) -> Self::Output;
}
