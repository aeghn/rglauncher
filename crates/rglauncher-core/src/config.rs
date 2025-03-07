use std::env;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct CommonConfig {
    pub icon_paths: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub db: DatabaseConfig,
    pub dict: Option<DictConfig>,
    pub ui: Option<UI>,
    pub common: CommonConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub db_path: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DictConfig {
    pub dir_path: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UI {
    pub dark_mode: Option<bool>,
}

impl Config {
    pub fn read_from_toml_file(filepath: Option<&String>) -> Self {
        let default_config_dir = match env::var("XDG_CONFIG_PATH") {
            Ok(path) => path,
            Err(_) => format!("{}/.config", env::var("HOME").unwrap()),
        };

        let config_filepath = format!("{}/rglauncher/config.toml", default_config_dir);
        let config_filepath = match filepath {
            Some(path) => path,
            None => &config_filepath,
        };
        let config_str = std::fs::read_to_string(config_filepath).expect(&format!(
            "Unable to read config content. {}",
            config_filepath
        ));

        toml::from_str(&config_str.as_str()).expect("unable to deserialize toml config.")
    }
}
