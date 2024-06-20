use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct FlareSolverrResponse {
    pub solution: Solution,
}

#[derive(Deserialize)]
pub struct Solution {
    pub url: String,
    pub response: String,
}

#[derive(Deserialize)]
pub struct AppConfig {
    pub base: BaseConfig,
    pub file: FileConfig,
    pub flaresolverr: FlareSolverrConfig,
    pub wordpress_api: WordPressApiConfig,
    pub wordpress_page: WordPressPageConfig,
}

#[derive(Deserialize)]
pub struct BaseConfig {
    pub name: String,
    pub version: String,
}

#[derive(Deserialize)]
pub struct FileConfig {
    pub source_data: String,
    pub processing_data: String,
}

#[derive(Deserialize)]
pub struct FlareSolverrConfig {
    pub flaresolverr_url: String,
}

#[derive(Deserialize)]
pub struct WordPressApiConfig {
    pub wordpress_url: String,
    pub username_api: String,
    pub password_api: String,
}

#[derive(Deserialize)]
pub struct WordPressPageConfig {
    pub template: String,
    pub status: String,
    pub parent: i32,
    pub author: i32,
}

pub fn load_config() -> Result<AppConfig, ConfigError> {
    let settings = Config::builder()
        .add_source(File::new("Settings.toml", config::FileFormat::Toml))
        .add_source(Environment::with_prefix("APP"))
        .build()?;

    settings.try_deserialize::<AppConfig>()
}
