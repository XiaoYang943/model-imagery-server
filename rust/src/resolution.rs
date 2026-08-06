use std::f64::consts::PI;

use crate::{
    cartographic::Cartographic, ellipsoidal_geodesic::EllipsoidGeodesic, rectangle::Rectangle,
};

pub struct Resolution {
    tile_size: u32,
    quality: f64,
    geodesic: EllipsoidGeodesic,
    metersPerPixelAtZeroLevel: f64,
}
impl Resolution {
    pub fn new(tile_size: Option<u32>, quality: Option<f64>) -> Self {
        let tileSize = tile_size.unwrap_or(256);
        let quality = quality.unwrap_or(0.8);
        let xNumAtZeroLevel = 2.0;
        let earthRadius = 6378137.0;
        let metersPerPixelAtZeroLevel =
            (2.0 * PI * earthRadius) / ((tileSize as f64) * xNumAtZeroLevel);
        return Self {
            tile_size: tileSize,
            quality: quality,
            geodesic: EllipsoidGeodesic::default(),
            metersPerPixelAtZeroLevel: metersPerPixelAtZeroLevel,
        };
    }
    pub fn tileSizeWithQuality(self: &Self) -> f64 {
        return (self.tile_size as f64) * self.quality;
    }
    pub fn computeLevel(
        self: &mut Self,
        tile_rectangle: &Rectangle,
        epsg: u32,
        max_level: u32,
    ) -> u32 {
        let midLatitude = (tile_rectangle.south + tile_rectangle.north) / 2.0;
        let startPoint = Cartographic::new(tile_rectangle.west, midLatitude, 0.0);
        let endPoint = Cartographic::new(tile_rectangle.east, midLatitude, 0.0);
        self.geodesic.setEndPoints(startPoint, endPoint);
        let tileSizeWithQuality = self.tileSizeWithQuality();
        let mut level = (self.metersPerPixelAtZeroLevel * midLatitude.cos())
            / (self.geodesic.get_surface_distance() / tileSizeWithQuality);
        level = level.ln() / 2f64.ln();
        let mut rounded = level.round() as u32;
        if rounded > max_level {
            rounded = max_level;
        }
        return rounded | 0;
    }
    pub fn computeLevel2(
        self: &Self,
        tile_rectangle: &Rectangle,
        epsg: u32,
        max_level: u32,
    ) -> u32 {
        let east = tile_rectangle.east;
        let west = tile_rectangle.west;
        let init_radius = std::f64::consts::PI;
        let mut level: u32 = 0;
        let gap_lon = east - west;
        loop {
            let current_level_gap = init_radius / ((2u32.pow(level)) as f64);
            let next_level_gap = init_radius / ((2u32.pow(level + 1)) as f64);
            if gap_lon <= current_level_gap && gap_lon >= next_level_gap {
                level = level + 1;
                break;
            } else {
                level = level + 1;
            }
        }
        if level > max_level {
            level = max_level;
        }
        return level;
    }
}
