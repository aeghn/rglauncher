use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub db: Option<DbConfig>,
    pub dict: Option<DictConfig>,
    pub ui: Option<UI>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DbConfig {
    pub db_path: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DictConfig {
    pub dir_path: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UI {
    pub icon_theme: Option<String>,
    pub dark_mode: Option<bool>,
}

const TEMPLATE_CONFIG: &str = include_str!("./template_config.toml");

impl Config {
    pub fn read_from_toml_file(filepath: Option<&String>) -> Self {
        let default_config_dir = match env::var("XDG_CONFIG_PATH") {
            Ok(path) => path,
            Err(_) => format!("{}/.config", env::var("HOME").unwrap()),
        };

        let config_filepath = format!("{}/rglauncher/config.toml", default_config_dir);
        let config_filepath = match filepath {
            Some(path) => Path::new(path),
            None => Path::new(&config_filepath),
        };

        if !config_filepath.exists() {
            let config_dir = config_filepath.parent().unwrap();
            if !config_dir.exists() {
                fs::create_dir(config_dir).expect("Unable to create the config directory");
            }

            let mut f =
                fs::File::create(config_filepath).expect("Unable to create the config file");
            let data = TEMPLATE_CONFIG.replace("{HOME}", env::var("HOME").unwrap().as_str());
            f.write_all(data.as_bytes())
                .expect("Unable to write to the config file");
        }

        let config_str = std::fs::read_to_string(config_filepath).expect(&format!(
            "Unable to read config content. {:?}",
            config_filepath
        ));

        toml::from_str(&config_str.as_str()).expect("unable to deserialize toml config.")
    }
}
