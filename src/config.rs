use std::fs;

use serde::Deserialize;
use toml;

#[derive(Deserialize)]
pub struct Config {
    pub access_key: String,
    pub secret_key: String,
}

impl Config {
    pub fn build() -> Self {
        // I don't want a result here since its happening at startup so it doesn't
        // really matter if it panics, at least at the moment
        let config_contents = fs::read_to_string("config.toml").unwrap();
        let conf: Self = toml::from_str(&config_contents).unwrap();
        conf
    }
}
