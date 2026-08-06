use std::{fmt::Debug, sync::Arc};

use crate::{
    merge_tile::find_value, rectangle::Rectangle, tiling_scheme::TilingScheme, vec2::Vec2,
};
use image::RgbaImage;
pub struct Tile {
    pub x: u32,
    pub y: u32,
    pub level: u32,
    pub translation: Vec2,
    pub scale: Vec2,
    pub clip: Vec<f64>,
    pub images: Vec<Option<Arc<RgbaImage>>>,
}
#[derive(Default)]
pub struct Tiles {
    pub data: Vec<Tile>,
    pub minx: u32,
    pub miny: u32,
    pub maxx: u32,
    pub maxy: u32,
    pub x_num: u32,
    pub y_num: u32,
    pub level: u32,
    pub layer_num: u32,
}
impl Debug for Tiles {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Tiles = tile_num:{} img_num:{} size:{}x{} level:{} leftTop:({},{}) rightBottom:({},{})",
            self.data.len(),self.data.len()*(self.layer_num as usize),self.x_num, self.y_num, self.level, self.minx, self.miny, self.maxx, self.maxy
        )
    }
}
impl Tiles {
    pub fn new(level: u32, layer_num: u32) -> Self {
        return Self {
            level,
            layer_num,
            ..Default::default()
        };
    }
    pub fn push(&mut self, tile: Tile) {
        self.data.push(tile);
    }
    pub fn len(&self) -> usize {
        return self.data.len();
    }
    pub fn push_finish(&mut self) {
        let (minx, miny, maxx, maxy) = find_value(&self.data);
        self.minx = minx;
        self.miny = miny;
        self.maxx = maxx;
        self.maxy = maxy;
        self.x_num = maxx - minx + 1;
        self.y_num = maxy - miny + 1;
    }
}
//TODO 支持webmercator投影,参考Cesium.ImageryLayer.prototype.createTileImagerySkeletons
pub fn get_coords_of_layer(
    tiling_scheme: &Box<dyn TilingScheme>,
    tile_rectangle: &Rectangle,
    imagery_level: u32,
    imagery_layer_rectangle: Option<Rectangle>,
    imagery_layer_num: usize,
) -> Option<Tiles> {
    let imagery_bounds = imagery_layer_rectangle.unwrap_or(Rectangle::MAX_VALUE);
    let rectangle = tile_rectangle.intersection(&imagery_bounds)?;
    let northwest_tile_coordinates =
        tiling_scheme.position_to_tile_x_y(&rectangle.north_west(), imagery_level)?;
    let southeast_tile_coordinates =
        tiling_scheme.position_to_tile_x_y(&&rectangle.south_east(), imagery_level)?;
    let mut tiles = Tiles::new(imagery_level, imagery_layer_num as u32);
    tiles.level = imagery_level;
    let terrain_width = tile_rectangle.compute_width();
    let terrain_height = tile_rectangle.compute_height();
    for x in northwest_tile_coordinates.x..=southeast_tile_coordinates.x {
        for y in northwest_tile_coordinates.y..=southeast_tile_coordinates.y {
            let imagery_rectangle = tiling_scheme.tile_x_y_to_rectange(x, y, imagery_level);
            let intersection = tile_rectangle.intersection(&imagery_rectangle)?;
            let scale_x = terrain_width / imagery_rectangle.compute_width();
            let scale_y = terrain_height / imagery_rectangle.compute_height();
            let translation = Vec2::new(
                (scale_x * (tile_rectangle.west - imagery_rectangle.west)) / terrain_width,
                (scale_y * (imagery_rectangle.north - tile_rectangle.north)) / terrain_height,
            );
            let scale = Vec2::new(scale_x, scale_y);
            let width = intersection.compute_width() / terrain_width;
            let height = intersection.compute_height() / terrain_height;
            let min_x = (intersection.west - tile_rectangle.west) / terrain_width;
            let min_y = (tile_rectangle.north - intersection.north) / terrain_height;
            tiles.push(Tile {
                x,
                y,
                level: imagery_level,
                translation: translation,
                scale: scale,
                clip: vec![min_x, min_y, min_x + width, min_y + height],
                images: vec![None; imagery_layer_num as usize],
            })
        }
    }
    tiles.push_finish();
    return Some(tiles);
}
#[cfg(test)]
mod tests {
    use crate::{
        geographic_tiling_scehme::GeographicTilingScheme, resolution::Resolution,
        webmercator_tiling_scehme::WebMercatorTilingScheme,
    };

    use super::*;
    #[test]
    fn test_resolution_compute_level2_3857() {
        let rectangle = Rectangle::from_bboxstr(
            "2.0839091585416316%2C0.628381592444241%2C2.0839560868065337%2C0.6284199417917619",
        );
        let tiling_scheme: Box<dyn TilingScheme> = Box::new(WebMercatorTilingScheme::default());
        let tiles = get_coords_of_layer(&tiling_scheme, &rectangle, 13, None, 1).unwrap();
        assert_eq!(tiles.minx, 6812);
        assert_eq!(tiles.miny, 3216);
        assert_eq!(tiles.maxx, 6813);
        assert_eq!(tiles.maxy, 3216);
        let tiles = get_coords_of_layer(&tiling_scheme, &rectangle, 14, None, 1).unwrap();
        assert_eq!(tiles.minx, 13625);
        assert_eq!(tiles.miny, 6432);
        assert_eq!(tiles.maxx, 13626);
        assert_eq!(tiles.maxy, 6432);
    }
    #[test]
    fn test_resolution_compute_level2() {
        let resolution = Resolution::new(None, None);
        let rectangle = Rectangle::from_bboxstr(
            "2.0839091585416316%2C0.628381592444241%2C2.0839560868065337%2C0.6284199417917619",
        );
        let level = resolution.computeLevel2(&rectangle, 4326, 20);
        assert!(level == 17);
        let imagery_layer_num = 3;
        let tiling_scheme: Box<dyn TilingScheme> = Box::new(GeographicTilingScheme::default());
        let tiles = get_coords_of_layer(&tiling_scheme, &rectangle, level, None, imagery_layer_num)
            .unwrap();
        println!("{:?}", tiles);

        let rectangle = Rectangle::from_degree(
            119.38205085662787,
            36.014049750548764,
            119.41071546303122,
            36.02706241294971,
        );
        let level = resolution.computeLevel2(&rectangle, 4326, 20);
        assert!(level == 13);
    }
    #[test]
    fn test_resolution_compute_level() {
        let mut resolution = Resolution::new(None, None);
        let rectangle = Rectangle::from_bboxstr(
            "2.0839091585416316%2C0.628381592444241%2C2.0839560868065337%2C0.6284199417917619",
        );
        let level = resolution.computeLevel(&rectangle, 4326, 20);
        assert!(level == 16);
        let imagery_layer_num = 3;
        let tiling_scheme: Box<dyn TilingScheme> = Box::new(GeographicTilingScheme::default());
        let tiles = get_coords_of_layer(&tiling_scheme, &rectangle, level, None, imagery_layer_num)
            .unwrap();
        println!("{:?}", tiles);
    }
    fn make_rectangle() -> Rectangle {
        let rectangle = Rectangle::from_degree(
            119.38205085662787,
            36.014049750548764,
            119.41071546303122,
            36.02706241294971,
        );
        return rectangle;
    }
    #[test]
    fn get_coords_of_layer_geographic_tiling_scheme() {
        let rectangle = make_rectangle();
        let mut imagery_level = 13;
        let imagery_layer_num = 3;
        let tiling_scheme: Box<dyn TilingScheme> = Box::new(GeographicTilingScheme::default());
        let mut tiles = get_coords_of_layer(
            &tiling_scheme,
            &rectangle,
            imagery_level,
            None,
            imagery_layer_num,
        )
        .unwrap();
        assert!(tiles.minx == 13625);
        assert!(tiles.miny == 2456);
        assert!(tiles.maxx == 13626);
        assert!(tiles.maxy == 2456);
        assert!(tiles.data.len() == 2);

        imagery_level = 14;
        tiles = get_coords_of_layer(
            &tiling_scheme,
            &rectangle,
            imagery_level,
            None,
            imagery_layer_num,
        )
        .unwrap();
        assert!(tiles.minx == 27250);
        assert!(tiles.miny == 4912);
        assert!(tiles.maxx == 27253);
        assert!(tiles.maxy == 4913);
        assert!(tiles.data.len() == 8);

        imagery_level = 15;
        tiles = get_coords_of_layer(
            &tiling_scheme,
            &rectangle,
            imagery_level,
            None,
            imagery_layer_num,
        )
        .unwrap();
        assert!(tiles.minx == 54500);
        assert!(tiles.miny == 9825);
        assert!(tiles.maxx == 54506);
        assert!(tiles.maxy == 9827);
        assert!(tiles.data.len() == 21);

        assert!(tiles.x_num == 7);
        assert!(tiles.y_num == 3);
        assert!(tiles.layer_num == 3);
        assert!(tiles.level == 15);
        assert!(tiles.minx + tiles.x_num - 1 == tiles.maxx);
        assert!(tiles.miny + tiles.y_num - 1 == tiles.maxy);
        let tile = &tiles.data[0];
        assert!(tile.images.len() == 3);
        assert!(tile.level == 15);
        for x in 0..tiles.x_num {
            for y in 0..tiles.y_num {
                let tile = &tiles.data[(x * tiles.y_num + y) as usize];
                assert!(tile.x == tiles.minx + x);
                assert!(tile.y == tiles.miny + y);
            }
        }
    }

    #[test]
    fn get_coords_of_layer_webmercator_tiling_scheme() {
        let rectangle = make_rectangle();
        let mut imagery_level = 13;
        let imagery_layer_num = 3;
        let tiling_scheme: Box<dyn TilingScheme> = Box::new(WebMercatorTilingScheme::default());
        let mut tiles = get_coords_of_layer(
            &tiling_scheme,
            &rectangle,
            imagery_level,
            None,
            imagery_layer_num,
        )
        .unwrap();
        assert!(tiles.minx == 6812);
        assert!(tiles.miny == 3216);
        assert!(tiles.maxx == 6813);
        assert!(tiles.maxy == 3216);
        assert!(tiles.data.len() == 2);

        imagery_level = 14;
        tiles = get_coords_of_layer(
            &tiling_scheme,
            &rectangle,
            imagery_level,
            None,
            imagery_layer_num,
        )
        .unwrap();
        assert!(tiles.minx == 13625);
        assert!(tiles.miny == 6432);
        assert!(tiles.maxx == 13626);
        assert!(tiles.maxy == 6432);
        assert!(tiles.data.len() == 2);

        imagery_level = 15;
        tiles = get_coords_of_layer(
            &tiling_scheme,
            &rectangle,
            imagery_level,
            None,
            imagery_layer_num,
        )
        .unwrap();
        assert!(tiles.minx == 27250);
        assert!(tiles.miny == 12864);
        assert!(tiles.maxx == 27253);
        assert!(tiles.maxy == 12865);
        assert!(tiles.data.len() == 8);

        imagery_level = 16;
        tiles = get_coords_of_layer(
            &tiling_scheme,
            &rectangle,
            imagery_level,
            None,
            imagery_layer_num,
        )
        .unwrap();
        assert!(tiles.minx == 54500);
        assert!(tiles.miny == 25728);
        assert!(tiles.maxx == 54506);
        assert!(tiles.maxy == 25731);
        assert!(tiles.data.len() == 28);

        imagery_level = 17;
        tiles = get_coords_of_layer(
            &tiling_scheme,
            &rectangle,
            imagery_level,
            None,
            imagery_layer_num,
        )
        .unwrap();
        assert!(tiles.minx == 109001);
        assert!(tiles.miny == 51457);
        assert!(tiles.maxx == 109012);
        assert!(tiles.maxy == 51463);
        assert!(tiles.data.len() == 12 * 7);
    }
}
