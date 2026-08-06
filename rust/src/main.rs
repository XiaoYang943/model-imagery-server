use actix_cors::Cors;
use actix_web::{
    get,
    http::{header, KeepAlive},
    middleware::Logger,
    web::{self, Data},
    App, HttpResponse, HttpServer, Responder, ResponseError,
};
use config::GlobalConfig;
use fetch_img::Fetch;
use lazy_static::lazy_static;
use std::{
    io::Cursor,
    sync::Arc,
    time::{Duration, Instant},
};
#[cfg(target_os = "windows")]
use winapi::um::{
    consoleapi::{GetConsoleMode, SetConsoleMode},
    processenv::GetStdHandle,
    winbase::STD_INPUT_HANDLE,
    wincon::{ENABLE_EXTENDED_FLAGS, ENABLE_QUICK_EDIT_MODE},
};

mod cartographic;
mod config;
mod ellipsoid;
mod ellipsoidal_geodesic;
mod entity;
mod fetch_img;
mod geographic_projection;
mod geographic_tiling_scehme;
mod math;
mod merge_tile;
mod projection;
mod rectangle;
mod resolution;
mod tile;
mod tiling_scheme;
mod to_radians;
mod vec2;
mod web_mercator_projection;
mod webmercator_tiling_scehme;
use entity::{GetMapParams, GetMapRequest};
use mini_moka::sync::Cache;
lazy_static! {
    pub static ref GLOBAL_CONFIG: GlobalConfig = GlobalConfig::new().unwrap();
}
#[get("/")]
async fn hello() -> impl Responder {
    "hello,world"
}
pub struct AppState {
    cache: Cache<String, Fetch>,
}
#[get("/getMap")]
async fn get_map(params: web::Query<GetMapParams>, app_state: Data<AppState>) -> HttpResponse {
    // sleep(Duration::from_secs(1)).await;
    let now = Instant::now();
    let cache_key = serde_json::to_string(&params.0).unwrap();
    let request: GetMapRequest = params.into();
    let mut img_option = None;
    //有缓存直接返回
    if let Some(fetch) = app_state.cache.get(&cache_key) {
        if let Fetch::Ready(img) = fetch {
            let mut bytes: Vec<u8> = Vec::new();
            match img.write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png) {
                Ok(_) => {
                    return HttpResponse::Ok().content_type("image/png").body(bytes);
                }
                Err(e) => {
                    log::error!("写入响应失败,{:?}", e);
                    return HttpResponse::InternalServerError().finish();
                }
            };
        }
    }
    //否则请求图片
    if img_option.is_none() {
        match request.requst_image(app_state.clone()).await {
            Ok(image) => {
                img_option = Some(image);
            }
            Err(e) => {
                return e.error_response();
            }
        }
    }
    if let Some(img) = img_option {
        let mut bytes: Vec<u8> = Vec::new();
        match img.write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png) {
            Ok(_) => {
                app_state
                    .cache
                    .insert(cache_key, Fetch::Ready(Arc::new(img)));
                let duration = now.elapsed().as_millis();
                log::info!("getMap耗时：{} 毫秒", duration);
                return HttpResponse::Ok().content_type("image/png").body(bytes);
            }
            Err(e) => {
                log::error!("写入响应失败,{:?}", e);
                return HttpResponse::InternalServerError().finish();
            }
        };
    } else {
        return HttpResponse::NotFound().finish();
    }
}
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 在目标平台为 Windows 时禁用快速编辑模式
    #[cfg(target_os = "windows")]
    {
        let h_input = unsafe { GetStdHandle(STD_INPUT_HANDLE) };
        let mut mode: u32 = 0;
        unsafe { GetConsoleMode(h_input, &mut mode) };
        mode &= !ENABLE_QUICK_EDIT_MODE;
        mode |= ENABLE_EXTENDED_FLAGS;
        unsafe { SetConsoleMode(h_input, mode) };
    }
    std::env::set_var("RUST_LOG", GLOBAL_CONFIG.rust_log.as_str());
    std::env::set_var("RUST_BACKTRACE", GLOBAL_CONFIG.rust_backtrace.as_str());
    env_logger::init();
    let mut cache_builder = Cache::builder();
    if let Some(time_to_live) = GLOBAL_CONFIG.time_to_live {
        cache_builder = cache_builder.time_to_live(Duration::from_secs(time_to_live * 60));
    }

    if let Some(time_to_idle) = GLOBAL_CONFIG.time_to_idle {
        cache_builder = cache_builder.time_to_idle(Duration::from_secs(time_to_idle * 60));
    }

    if let Some(max_capacity) = GLOBAL_CONFIG.max_capacity {
        cache_builder = cache_builder.max_capacity(max_capacity);
    }

    let app_state = web::Data::new(AppState {
        cache: cache_builder.build(),
    });
    println!(
        "服务启动,地址为 http://{}:{}",
        GLOBAL_CONFIG.host, GLOBAL_CONFIG.port
    );
    HttpServer::new(move || {
        let logger = Logger::default();
        let cors = Cors::default()
            .allowed_origin_fn(|_, _req_head| true)
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
            .allowed_header(header::CONTENT_TYPE)
            .max_age(3600);
        App::new()
            .app_data(app_state.clone())
            .wrap(cors)
            .wrap(logger)
            .service(hello)
            .service(get_map)
    })
    .keep_alive(KeepAlive::Os)
    .bind((GLOBAL_CONFIG.host.as_str(), GLOBAL_CONFIG.port))?
    .run()
    .await
}
