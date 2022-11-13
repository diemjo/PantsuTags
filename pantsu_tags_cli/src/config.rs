use std::path::PathBuf;
use directories::{BaseDirs};
use figment::{Figment};
use figment::providers::{Format, Serialized, Yaml};
use serde_derive::{Deserialize,Serialize};
use crate::AppError;

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct AppConfig {
    pub library_path: PathBuf,
    pub database_path: PathBuf,
    pub config_path: PathBuf
}

impl AppConfig {

    pub fn load_config() -> Self {
        Figment::from(Serialized::defaults(AppConfig::default()))
            .merge(Yaml::file("/etc/pantsu_tags/config.yaml"))
            .merge(Yaml::file(BaseDirs::new().unwrap().config_dir().join("pantsu_tags.yaml").as_path()))
            .merge(Yaml::file("./pantsu_tags.yaml"))
            .extract()
            .or_else(|e|Err(AppError::ConfigError(e)))
            .unwrap()
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            library_path: PathBuf::from("./test_image_lib"), //file_handler::default_lib_dir(),
            database_path: PathBuf::from("./pantsu_tags.db"), //file_handler::default_db_dir()
            config_path: PathBuf::from("./pantsu_tags.log")
        }
    }
}