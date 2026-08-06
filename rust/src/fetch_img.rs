use crate::AppState;
use actix_web::{rt::time::sleep, web::Data};
use derive_more::{Display, Error};
use futures::future::Ready;
use image::{load_from_memory, DynamicImage, RgbaImage};
use lazy_static::lazy_static;
use log::{info, warn};
use reqwest::Response;
use reqwest::{Client, ClientBuilder};
use std::{
    future::Future,
    sync::Arc,
    task::Poll,
    time::{Duration, Instant},
};
pub type FetchReady = Arc<RgbaImage>;
#[derive(Clone)]
pub enum Fetch {
    Loading,
    Ready(FetchReady),
}
#[derive(Debug, Display, Error, Clone)]
pub enum FetchImageError {
    #[display(fmt = "网络错误,请检查网络!")]
    NetError,
    #[display(fmt = "加载错误,请检查图片格式")]
    LoadError,
}
pub struct WatingFetch<'a> {
    app_state: Data<AppState>,
    request_url: &'a String,
}
impl<'a> Future for WatingFetch<'a> {
    type Output = FetchReady;
    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        if let Fetch::Ready(e) = self.app_state.cache.get(self.request_url).unwrap() {
            Poll::Ready(e.clone())
        } else {
            Poll::Pending
        }
    }
}
lazy_static! {
    pub static ref CLIENT: Client = ClientBuilder::new()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();
}
pub async fn fetch_image(
    app_state: Data<AppState>,
    request_url: &String,
    tile_size: u32,
) -> FetchReady {
    let cached = app_state.cache.get(request_url);
    if cached.is_none() {
        app_state.cache.insert(request_url.clone(), Fetch::Loading);
        //发出请求并解析图片
        let img = match CLIENT.get(request_url.clone()).send().await {
            Ok(resp) => match load_from_response(resp).await {
                Ok(img) => img,
                Err(e2) => {
                    warn!("{:?},请求地址是:{}", e2, request_url);
                    RgbaImage::new(tile_size, tile_size)
                }
            },
            Err(e) => {
                warn!("{:?},请求地址是:{}", e, request_url);
                RgbaImage::new(tile_size, tile_size)
            }
        };
        //更新缓存值
        app_state.cache.invalidate(request_url);
        app_state
            .cache
            .insert(request_url.clone(), Fetch::Ready(Arc::new(img)));
    }
    // return WatingFetch {//会挂起浏览器发起的前几个请求
    //     app_state,
    //     request_url,
    // }
    // .await;
    return async {
        loop {
            if let Fetch::Ready(e) = app_state.cache.get(request_url).unwrap() {
                break e.clone();
            } else {
                sleep(Duration::from_millis(10)).await;
            };
        }
    }
    .await;
}
async fn load_from_response(resp: Response) -> Result<RgbaImage, FetchImageError> {
    let content = resp.bytes().await.map_err(|_| FetchImageError::LoadError)?;
    let dynamic_img = image::load_from_memory(&content).map_err(|_| FetchImageError::LoadError)?;
    let img = if let DynamicImage::ImageRgba8(value) = dynamic_img {
        value
    } else {
        dynamic_img.to_rgba8()
    };
    return Ok(img);
}

#[cfg(test)]
mod tests {
    use std::{io::Cursor, vec};

    use super::*;
    use actix_web::web;
    use futures::{stream, StreamExt}; // 0.3.5
    use image::GenericImageView;
    use mini_moka::sync::Cache;
    #[test]
    fn test_load_from_memory() {
        let img = image::open("out/test_request_image.png").unwrap();
        let mut bytes: Vec<u8> = Vec::new();
        img.write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png)
            .unwrap();

        let img2 = image::load_from_memory(&bytes).unwrap();
        assert_eq!(img.dimensions(), img2.dimensions());
    }
    #[actix_web::test]
    async fn test_channel() {
        let now = Instant::now();
        let request_url = "http://intenal.geoway-atlas.com:31280/mapserver/vmap/osm_road_zhucheng/getMap?styleId=osm_road_zhucheng&x=54504&y=9852&l=16".to_string();
        let app_state = web::Data::new(AppState {
            cache: Cache::builder().build(),
        });
        let mut tasks = vec![];
        for i in 0..3 {
            let cloned = app_state.clone();
            let request_url_cloned = request_url.clone();
            tasks.push(async move {
                let img = fetch_image(cloned, &request_url_cloned, 256).await;
                (i, img)
            })
        }
        let results: Vec<_> = stream::iter(tasks).buffer_unordered(5).collect().await;
        for item in results {
            let (w, h) = item.1.dimensions();
            println!("{},{},{}", item.0, w, h);
        }
        let duration = now.elapsed().as_millis();
        println!("请求耗时：{} 毫秒", duration);
    }
    #[actix_web::test]
    async fn test_request_time() {
        let now = Instant::now();
        let request_url = "http://intenal.geoway-atlas.com:31280/mapserver/vmap/osm_road_zhucheng/getMap?styleId=osm_road_zhucheng&x=13623&y=2460&l=14".to_string();
        let _ = reqwest::get(&request_url).await.unwrap();
        let duration = now.elapsed().as_millis();
        println!("请求耗时：{} 毫秒", duration);
    }
    #[test]
    fn test_request_time_blocking() {
        let now = Instant::now();
        let request_url = "http://intenal.geoway-atlas.com:31280/mapserver/vmap/osm_road_zhucheng/getMap?styleId=osm_road_zhucheng&x=13623&y=2460&l=14".to_string();
        let _ = reqwest::blocking::get(&request_url).unwrap();
        let duration = now.elapsed().as_millis();
        println!("请求耗时：{} 毫秒", duration);
    }
}
