use schemars::schema::RootSchema;
use serde::{Deserialize, Serialize};
use serde_json::{from_str as json_from_str, to_string_pretty};
use serde_yaml::from_str as yaml_from_str;
use std::fs::read_to_string;
#[derive(Serialize, Deserialize)]
pub struct GlobalConfig {
    pub host: String,
    pub port: u16,
    pub parallel_requests: usize,
    pub default_tile_size: u32,
    pub rust_log: String,
    pub rust_backtrace: String,
    pub max_capacity: Option<u64>,
    pub time_to_live: Option<u64>,
    pub time_to_idle: Option<u64>,
}
impl GlobalConfig {
    pub fn new() -> Option<Self> {
        let schema = yaml_from_str::<RootSchema>(
            &read_to_string("config.yml").expect("加载配置文件config.yml失败"),
        );
        return match schema {
            Ok(json) => {
                let data = to_string_pretty(&json)
                    .unwrap_or_else(|_| panic!("config.yml内容不对，请检查配置文件"));
                let p = json_from_str(&*data).expect("无法转换配置文件");
                return Some(p);
            }
            Err(err) => {
                println!("{}", err);
                None
            }
        };
    }
}

#[test]
fn test_load_env_conf_mysql() {
    let pro = GlobalConfig::new();
    assert!(pro.is_some());
    let pro = pro.unwrap();
    assert!(pro.host == "127.0.0.1".to_string());
    assert!(pro.port == 8080);
}
