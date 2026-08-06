use image::{GenericImage, ImageBuffer, Pixel, RgbaImage};

use crate::{
    tile::{Tile, Tiles},
    vec2::Vec2,
};

pub fn merge_tile(tiles: &mut Tiles, tile_size: u32, imagery_layer_num: u32) -> Option<RgbaImage> {
    let x_num = tiles.x_num;
    let y_num = tiles.y_num;
    let img_width = x_num * tile_size;
    let img_height = y_num * tile_size;
    let mut img_merged: RgbaImage = ImageBuffer::new(img_width, img_height);
    for x in 0..x_num {
        let offset_x = x * tile_size;
        for y in 0..y_num {
            let offset_y = y * tile_size;
            let index = (x * y_num + y) as usize; //get_coords_of_layer中tile只按这个顺序push进去的
            let tile = &tiles.data[index];
            for (_, img) in (&tile.images).into_iter().enumerate() {
                if img.is_none() {
                    continue;
                }
                let img = img.as_ref().unwrap();
                // let _ = img.save(format!("out/{}-{}-{}.png", x, y, layer_index));
                for x in 0..tile_size {
                    let real_x = x + offset_x;
                    for y in 0..tile_size {
                        let pixel = img_merged.get_pixel_mut(real_x, y + offset_y);
                        let img_pixel = img.get_pixel(x, y);
                        pixel.blend(&img_pixel)
                    }
                }
            }
        }
    }
    return clip(tiles, tile_size, &mut img_merged);
}
pub fn find_value(tiles: &Vec<Tile>) -> (u32, u32, u32, u32) {
    let mut minx = 999999999;
    let mut miny = 999999999;
    let mut maxx = 0;
    let mut maxy = 0;
    for tile in tiles {
        if tile.x > maxx {
            maxx = tile.x;
        }
        if tile.y > maxy {
            maxy = tile.y;
        }
        if tile.x < minx {
            minx = tile.x;
        }
        if tile.y < miny {
            miny = tile.y;
        }
    }
    return (minx, miny, maxx, maxy);
}
/// 裁剪出多边形覆盖的图片纹理
pub fn clip(tiles: &Tiles, tile_size: u32, img_merged: &mut RgbaImage) -> Option<RgbaImage> {
    let tile_size_f64 = tile_size as f64;
    let x_num = tiles.x_num;
    let y_num = tiles.y_num;
    let mut left_top = Vec2::new(0.0, 0.0);
    let mut right_bottom = Vec2::new(1.0, 1.0);
    let first_tile = &tiles.data[0];
    let last_tile = &tiles.data[tiles.len() - 1];
    left_top.scale_and_add(&first_tile.scale, &first_tile.translation);
    left_top.scale(tile_size_f64);
    right_bottom.scale_and_add(&last_tile.scale, &last_tile.translation);
    let offset = Vec2::new(
        ((x_num as f64) - 1.0) * tile_size_f64,
        ((y_num as f64) - 1.0) * tile_size_f64,
    );
    right_bottom.scale_and_add(&Vec2::new(tile_size_f64, tile_size_f64), &offset);
    let sub_img = img_merged
        .sub_image(
            left_top.x as u32,
            left_top.y as u32,
            (right_bottom.x - left_top.x) as u32,
            (right_bottom.y - left_top.y) as u32,
        )
        .to_image();
    return Some(sub_img);
}
