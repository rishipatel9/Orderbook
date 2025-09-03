use serde::Deserialize;
use lazy_static::lazy_static;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub server_addr: String,
    pub redis_url: String,
    pub ws_addr:String
}
lazy_static! {
    pub static ref SETTINGS: config::Config = config::Config::builder()
        .add_source(config::File::with_name("./config.yaml"))
        .build()
        .unwrap();

    pub static ref APP_CONFIG: AppConfig = SETTINGS.clone().try_deserialize().unwrap();
}
