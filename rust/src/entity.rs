use crate::{
    fetch_img::fetch_image,
    geographic_tiling_scehme::GeographicTilingScheme,
    merge_tile::merge_tile,
    rectangle::Rectangle,
    resolution::Resolution,
    tile::get_coords_of_layer,
    tiling_scheme::{self, TilingScheme},
    webmercator_tiling_scehme::WebMercatorTilingScheme,
    AppState, GLOBAL_CONFIG,
};
use actix_web::{
    http::{header::ContentType, StatusCode},
    web::{self, Data, Query},
    HttpResponse, ResponseError,
};
use derive_more::{Display, Error};
use futures::{stream, StreamExt}; // 0.3.5
use image::RgbaImage;
use new_string_template::template::Template;
use rand::seq::SliceRandom; //
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
    time::{Duration, Instant},
    vec,
};
pub const DEFAULT_TILE_SIZE: u32 = 256;
#[derive(Deserialize, Serialize, PartialEq, Eq, Clone)]
pub struct GetMapParams {
    pub epsg: u32,
    #[serde(alias = "maxLevel")]
    pub max_level: u32,
    pub bbox: String,
    pub url: String,
    #[serde(alias = "levelOffset")]
    pub level_offset: i32,
}
impl Hash for GetMapParams {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.url.hash(state);
        self.max_level.hash(state);
        self.bbox.hash(state);
        self.epsg.hash(state);
        self.level_offset.hash(state);
    }
}
impl Into<GetMapRequest> for Query<GetMapParams> {
    fn into(self) -> GetMapRequest {
        let res: Vec<_> = self
            .bbox
            .split(",")
            .map(|x| x.parse::<f64>().unwrap())
            .collect();
        let r = Rectangle::new(res[0], res[1], res[2], res[3]);
        let mut resolution = Resolution::new(Some(DEFAULT_TILE_SIZE), None);
        let level = resolution.computeLevel(&r, self.epsg, self.max_level);
        let tiling_scheme: Box<dyn TilingScheme>;
        if self.epsg == 4326 {
            tiling_scheme = Box::new(GeographicTilingScheme::default());
        } else {
            tiling_scheme = Box::new(WebMercatorTilingScheme::default());
        }
        GetMapRequest {
            tile_size: Some(256),
            rectangle: r,
            urls: split_urls(&self.url),
            level: level,
            epsg: self.epsg,
            max_level: self.max_level,
            tiling_scheme: tiling_scheme,
            level_offset: self.level_offset,
        }
    }
}
pub struct GetMapRequest {
    pub tile_size: Option<u32>,
    pub rectangle: Rectangle,
    pub urls: Vec<String>,
    pub level: u32,
    pub epsg: u32,
    pub max_level: u32,
    pub tiling_scheme: Box<dyn TilingScheme>,
    pub level_offset: i32,
}
#[derive(Debug, Display, Error)]
pub enum GetMapUserError {
    #[display(fmt = "服务器内部错误，请稍后再试.")]
    InternalError,
    #[display(fmt = "没找到图片，请更换图层或区域.")]
    NotFound,
    #[display(fmt = "参数错误，请修正参数.")]
    ParamsError,
    #[display(fmt = "图片请求失败.")]
    FetchImageError,
}

impl ResponseError for GetMapUserError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::html())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            GetMapUserError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            GetMapUserError::NotFound => StatusCode::NOT_FOUND,
            GetMapUserError::ParamsError => StatusCode::NOT_FOUND,
            GetMapUserError::FetchImageError => StatusCode::NOT_FOUND,
        }
    }
}
impl GetMapRequest {
    pub async fn requst_image(
        &self,
        app_state: Data<AppState>,
    ) -> Result<RgbaImage, GetMapUserError> {
        if self.urls.is_empty() {
            return Err(GetMapUserError::ParamsError);
        }
        let tile_size = self.tile_size.unwrap_or(GLOBAL_CONFIG.default_tile_size);
        //图层rectangle默认是全球，tilingScheme默认是经纬度投影，所以这里不需要按图层请求不同的tiles
        let tiles_option = get_coords_of_layer(
            &self.tiling_scheme,
            &self.rectangle,
            self.level,
            None,
            self.urls.len(),
        );
        if tiles_option.is_none() {
            return Err(GetMapUserError::ParamsError);
        }
        let mut tiles = tiles_option.unwrap();
        // let now = Instant::now();
        let mut tasks = vec![];
        for (tile_index, tile) in tiles.data.iter_mut().enumerate() {
            for (img_index, url) in (&self.urls).into_iter().enumerate() {
                let request_url = make_request_url(
                    &url,
                    tile.x,
                    tile.y,
                    ((tile.level as i32) + self.level_offset) as u32,
                )
                .unwrap();
                let app_state_cloned = app_state.clone();
                tasks.push(async move {
                    let img_arc = fetch_image(app_state_cloned, &request_url, tile_size).await;
                    (tile_index, img_index, img_arc)
                });
            }
        }
        let results: Vec<_> = stream::iter(tasks)
            .buffer_unordered(GLOBAL_CONFIG.parallel_requests)
            .collect()
            .await;
        for item in results {
            tiles.data[item.0].images[item.1] = Some(item.2);
        }
        // log::info!("{:?}", tiles);
        // let duration = now.elapsed().as_millis();
        // log::info!("请求耗时：{} 毫秒", duration);
        // let now = Instant::now();
        if let Some(merged_img) = merge_tile(&mut tiles, tile_size, self.urls.len() as u32) {
            // let duration = now.elapsed().as_micros();
            // log::info!("合并耗时：{} 微秒", duration);
            return Ok(merged_img);
        } else {
            return Err(GetMapUserError::NotFound);
        };
    }
}
const VARIALBE_NAME_LIST: [&str; 30] = [
    "s1", "s2", "s3", "s4", "s5", "s6", "s7", "s8", "s9", "s10", "s11", "s12", "s13", "s14", "s15",
    "s16", "s17", "s18", "s19", "s20", "s21", "s22", "s23", "s24", "s25", "s26", "s27", "s28",
    "s29", "s30",
];
fn make_request_url(url_template: &String, x: u32, y: u32, level: u32) -> Option<String> {
    let mut url_template_cloned = url_template.clone();
    let mut args = HashMap::new();
    let re = Regex::new(r"\{(\d+\-\d+)\}|\{((\d+\,?)*)\}").unwrap();
    let mut i = 0;
    // let mut url_template_to_regex = url_template_cloned.clone();
    while re.is_match(&url_template_cloned) {
        let cap = re.captures(&url_template_cloned).unwrap();
        //是否是逗号模式
        let is_second_mode = cap.get(1).is_none();
        let mut num_list;
        if is_second_mode {
            num_list = cap[2]
                .to_string()
                .split(",")
                .map(|x| x.parse::<u32>().unwrap())
                .collect::<Vec<u32>>();
        } else {
            num_list = vec![];
            let v = cap[1]
                .to_string()
                .split("-")
                .map(|x| x.parse::<u32>().unwrap())
                .collect::<Vec<u32>>();
            for i in v[0]..v[1] {
                num_list.push(i);
            }
        }
        let value = num_list.choose(&mut rand::thread_rng()).unwrap();
        let variable_name = VARIALBE_NAME_LIST[i];
        let matched = if is_second_mode {
            cap.get(2).unwrap()
        } else {
            cap.get(1).unwrap()
        };
        url_template_cloned.replace_range(matched.start()..matched.end(), variable_name);
        args.insert(variable_name, value.to_string());
        i = i + 1;
    }
    let template = Template::new(url_template_cloned);
    let level = level.to_string();
    let x = x.to_string();
    let y = y.to_string();
    args.insert("z", level);
    args.insert("x", x);
    args.insert("y", y);
    return Some(template.render_nofail(&args));
}
fn split_urls(url_str: &String) -> Vec<String> {
    let re = fancy_regex::Regex::new(r"(http[s]?:\/\/.*?)(?=(,http)|$)").unwrap();
    re.captures_iter(url_str)
        .map(|x| x.unwrap().get(1).unwrap().as_str().to_string())
        .collect::<Vec<_>>()
}
#[cfg(test)]
mod tests {
    use crate::geographic_tiling_scehme::GeographicTilingScheme;

    use super::*;
    use actix_web::rt::time::sleep;
    use mini_moka::sync::Cache;
    use std::time::{Duration, Instant};
    #[test]
    fn test_urls_split() {
        let  s0 = "http://intenal.geoway-atlas.com:31280/mapserver/vmap/osm_landuse_zhucheng/getMap?styleId=osm_landuse_zhucheng&x={x}&y={y}&l={z}";
        let s1 = r#"http://intenal.geoway-atlas.com:31280/mapserver/vmap/hl_china/getMap?styleId=new&x={x}&y={y}&l={z}&control={"otherDisplay":true,"layers":[{"id":"osm_landuse_zhucheng","filterStr":"name!=房屋"}]}"#;
        let s2 = "http://intenal.geoway-atlas.com:31280/mapserver/vmap/osm_road_zhucheng/getMap?styleId=osm_road_zhucheng&x={x}&y={y}&l={z}";
        let s = format!("{},{},{}", s0, s1, s2);
        let urls = split_urls(&s);
        assert_eq!(urls.len(), 3);
        assert_eq!(urls[0], s0);
        assert_eq!(urls[1], s1);
        assert_eq!(urls[2], s2);

        let  s0 = "https://intenal.geoway-atlas.com:31280/mapserver/vmap/osm_landuse_zhucheng/getMap?styleId=osm_landuse_zhucheng&x={x}&y={y}&l={z}";
        let s1 = r#"http://intenal.geoway-atlas.com:31280/mapserver/vmap/hl_china/getMap?styleId=new&x={x}&y={y}&l={z}&control={"otherDisplay":true,"http":[{"id":"osm_landuse_zhucheng","filterStr":"name!=房屋"}]}"#;
        let s2 = "https://intenal.geoway-atlas.com:31280/mapserver/vmap/osm_road_zhucheng/getMap?styleId=osm_road_zhucheng&x={x}&y={y}&l={z}";
        let s = format!("{},{},{}", s0, s1, s2);
        let urls = split_urls(&s);
        assert_eq!(urls.len(), 3);
        assert_eq!(urls[0], s0);
        assert_eq!(urls[1], s1);
        assert_eq!(urls[2], s2);
    }
    #[test]
    fn test_url_split_render() {
        let  url = r#"http://intenal.geoway-atlas.com:31280/mapserver/vmap/hl_china/getMap?styleId=new&x={x}&y={y}&l={z}&control={"otherDisplay":true,"layers":[{"id":"osm_landuse_zhucheng","filterStr":"name!=房屋"}]}"#.to_string();
        let  expect_url = r#"http://intenal.geoway-atlas.com:31280/mapserver/vmap/hl_china/getMap?styleId=new&x=1&y=1&l=1&control={"otherDisplay":true,"layers":[{"id":"osm_landuse_zhucheng","filterStr":"name!=房屋"}]}"#.to_string();
        let re = Regex::new(r"\{(\d+\-\d+)\}|\{((\d+\,?)*)\}").unwrap();
        assert!(re.is_match(&url) == false);

        let custom_regex = Regex::new(r"(?mi)\{\s*(\S+?)\s*\}").unwrap();
        let templ = Template::new(&url).with_regex(&custom_regex);
        let data = {
            let mut map = HashMap::new();
            map.insert("x", "1");
            map.insert("y", "1");
            map.insert("z", "1");
            map
        };
        let rendered = templ.render_nofail(&data);
        println!("{}", rendered);
        assert!(rendered == expect_url);

        let request_url = make_request_url(&url, 1, 1, 1).unwrap();
        println!("{}", expect_url);
        assert!(request_url == expect_url);
    }
    #[test]
    fn test_regex() {
        {
            let url = "https://t{2-10}.tianditu.gov.cn{1-5}/img_c/wmts?SERVICE=WMTS&REQUEST=GetTile&VERSION=1.0.0&LAYER=img&STYLE=default&TILEMATRIXSET=w&FORMAT=tiles&TILECOL={x}&TILEROW={y}&TILEMATRIX={z}&tk=75f0434f240669f4a2df6359275146d2";
            let re = Regex::new(r"\{(\d+\-\d+)\}").unwrap();
            assert!(re.is_match(url));
            let mut caps = re.captures_iter(url);
            let mut cap = caps.next().unwrap();
            assert_eq!("{2-10}", &cap[0]);
            assert_eq!("2-10", &cap[1]);

            let mut new_url = url.to_string();
            let matched = cap.get(1).unwrap();
            new_url.replace_range(matched.start()..matched.end(), "s");
            assert_eq!(new_url,"https://t{s}.tianditu.gov.cn{1-5}/img_c/wmts?SERVICE=WMTS&REQUEST=GetTile&VERSION=1.0.0&LAYER=img&STYLE=default&TILEMATRIXSET=w&FORMAT=tiles&TILECOL={x}&TILEROW={y}&TILEMATRIX={z}&tk=75f0434f240669f4a2df6359275146d2".to_string());

            cap = caps.next().unwrap();
            assert_eq!("{1-5}", &cap[0]);
            assert_eq!("1-5", &cap[1]);
        }
        {
            let url = "https://t{2,3,4}.tianditu.gov.cn{4,5,6}/img_c/wmts?SERVICE=WMTS&REQUEST=GetTile&VERSION=1.0.0&LAYER=img&STYLE=default&TILEMATRIXSET=w&FORMAT=tiles&TILECOL={x}&TILEROW={y}&TILEMATRIX={z}&tk=75f0434f240669f4a2df6359275146d2";
            let re = Regex::new(r"\{((\d+,?)*)\}").unwrap();
            assert!(re.is_match(url));
            let mut caps = re.captures_iter(url);
            let mut cap = caps.next().unwrap();
            assert_eq!("{2,3,4}", &cap[0]);
            assert_eq!("2,3,4", &cap[1]);

            cap = caps.next().unwrap();
            assert_eq!("{4,5,6}", &cap[0]);
            assert_eq!("4,5,6", &cap[1]);
        }
        {
            let url = "https://t{2-10}.tianditu.gov.cn{2,3,4}/{2,3,4}img_c/wmts?SERVICE=WMTS&REQUEST=GetTile&VERSION=1.0.0&LAYER=img&STYLE=default&TILEMATRIXSET=w&FORMAT=tiles&TILECOL={x}&TILEROW={y}&TILEMATRIX={z}&tk=75f0434f240669f4a2df6359275146d2";
            let re = Regex::new(r"\{(\d+\-\d+)\}|\{((\d+\,?)*)\}").unwrap();
            assert!(re.is_match(url));
            let mut caps = re.captures_iter(url);
            let mut cap = caps.next().unwrap();
            assert_eq!("{2-10}", &cap[0]);
            assert_eq!("2-10", &cap[1]);
            cap = caps.next().unwrap();
            assert_eq!("{2,3,4}", &cap[0]);
            assert!(cap.get(1).is_none());
            assert_eq!("2,3,4", &cap[2]);
        }
        {
            let url = "https://t2.tianditu.gov.cn/img_c/wmts?SERVICE=WMTS&REQUEST=GetTile&VERSION=1.0.0&LAYER=img&STYLE=default&TILEMATRIXSET=w&FORMAT=tiles&TILECOL={x}&TILEROW={y}&TILEMATRIX={z}&tk=75f0434f240669f4a2df6359275146d2";
            let re = Regex::new(r"\{(\d+\-\d+)\}|\{((\d+\,?)*)\}").unwrap();
            assert!(!re.is_match(url));
        }
    }
    #[test]
    fn test_make_request_url() {
        let mut url = "http://intenal.geoway-atlas.com:31280/mapserver/vmap/osm_landuse_zhucheng/getMap?styleId=osm_landuse_zhucheng&x={x}&y={y}&l={z}".to_string();
        let mut request_url = make_request_url(&url, 1, 1, 1).unwrap();
        let  expect_url = "http://intenal.geoway-atlas.com:31280/mapserver/vmap/osm_landuse_zhucheng/getMap?styleId=osm_landuse_zhucheng&x=1&y=1&l=1".to_string();
        assert!(request_url == expect_url);
        url = "https://t{2-10}.tianditu.gov.cn/img_c{1,2,3}/wmts?SERVICE=WMTS&REQUEST=GetTile&VERSION=1.0.0&LAYER=img&STYLE=default&TILEMATRIXSET=w&FORMAT=tiles&TILECOL={x}&TILEROW={y}&TILEMATRIX={z}&tk=75f0434f240669f4a2df6359275146d2".to_string();
        request_url = make_request_url(&url, 1, 1, 1).unwrap();
        let re = Regex::new(r"https://t\d+\.tianditu\.gov\.cn\/img\_c[1,2,3]{1}").unwrap();
        assert!(re.is_match(&request_url));
    }
    #[actix_web::test]
    async fn test_request_image() {
        let app_state = init_log();
        let urls = make_urls();
        let rectangle = make_rectangle();
        let request = GetMapRequest {
            urls: urls,
            rectangle: rectangle,
            level: 13,
            tile_size: Some(GLOBAL_CONFIG.default_tile_size),
            epsg: 4326,
            max_level: 24,
            tiling_scheme: Box::new(GeographicTilingScheme::default()),
            level_offset: 1,
        };
        let img = request.requst_image(app_state).await.unwrap();
        let size = img.dimensions();
        let _ = img.save("out/test_request_image.png");
        assert!(size.0 == 333);
        assert!(size.1 == 151);
    }
    #[test]
    fn test_get_coord_of_layer() {
        let rectangle = make_rectangle();
        let tiling_scheme: Box<dyn TilingScheme> = Box::new(GeographicTilingScheme::default());
        let tiles = get_coords_of_layer(&tiling_scheme, &rectangle, 13, None, 3).unwrap();
        assert!(tiles.data[0].x == tiles.minx);
        assert!(tiles.data[tiles.len() - 1].x == tiles.maxx);
        assert!(tiles.len() == 2);
        assert!(tiles.maxx == 13626);
        assert!(tiles.minx == 13625);
        assert!(tiles.maxy == 2456);
        assert!(tiles.miny == 2456);
    }
    #[actix_web::test]
    async fn test_async() {
        let mut tasks = vec![];
        for i in 0..10 {
            let now = Instant::now();
            tasks.push(async move {
                sleep(Duration::from_secs(2)).await;
                let duration = now.elapsed().as_secs();
                println!("index is {},seconds is {}", i, duration);
            })
        }
        let result: Vec<_> = stream::iter(tasks).buffer_unordered(5).collect().await;
    }
    fn init_log() -> web::Data<AppState> {
        std::env::set_var("RUST_LOG", "warn");
        std::env::set_var("RUST_BACKTRACE", "1");
        env_logger::init();
        let mut cache_builder = Cache::builder();
        if let Some(time_to_live) = GLOBAL_CONFIG.time_to_live {
            cache_builder = cache_builder.time_to_live(Duration::from_secs(time_to_live * 60));
        }

        if let Some(time_to_idle) = GLOBAL_CONFIG.time_to_idle {
            cache_builder = cache_builder.time_to_idle(Duration::from_secs(time_to_idle * 60));
        }

        if let Some(max_capacity) = GLOBAL_CONFIG.max_capacity {
            cache_builder = cache_builder.max_capacity(max_capacity * 1024 * 1024);
        }

        let app_state = web::Data::new(AppState {
            cache: cache_builder.build(),
        });
        return app_state;
    }
    fn make_urls() -> Vec<String> {
        let layer1:String = "http://intenal.geoway-atlas.com:31280/mapserver/vmap/osm_landuse_zhucheng/getMap?styleId=osm_landuse_zhucheng&x={x}&y={y}&l={z}&tileSize=512".to_string();
        let layer2:String = "http://intenal.geoway-atlas.com:31280/mapserver/vmap/hl_china/getMap?styleId=new&x={x}&y={y}&l={z}&tileSize=512".to_string();
        let layer3:String = "http://intenal.geoway-atlas.com:31280/mapserver/vmap/osm_road_zhucheng/getMap?styleId=osm_road_zhucheng&x={x}&y={y}&l={z}&tileSize=512".to_string();
        let urls: Vec<String> = vec![layer1, layer2, layer3];
        // let layer1:String = "https://t3.tianditu.gov.cn/img_c/wmts?SERVICE=WMTS&REQUEST=GetTile&VERSION=1.0.0&LAYER=img&STYLE=default&TILEMATRIXSET=w&FORMAT=tiles&TILECOL={x}&TILEROW={y}&TILEMATRIX={z}&tk=75f0434f240669f4a2df6359275146d2".to_string();
        // let layer2:String = "https://t3.tianditu.gov.cn/cia_c/wmts?SERVICE=WMTS&REQUEST=GetTile&VERSION=1.0.0&LAYER=img&STYLE=default&TILEMATRIXSET=w&FORMAT=tiles&TILECOL={x}&TILEROW={y}&TILEMATRIX={z}&tk=75f0434f240669f4a2df6359275146d2".to_string();
        // let urls: Vec<String> = vec![layer1, layer2];
        return urls;
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
}
