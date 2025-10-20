use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub default_device: String,
}

fn get_config_path() -> PathBuf {
    let mut path = dirs::config_dir().expect("Could not find config directory");
    path.push("brightnessctl-rs");
    path.push("config.toml");
    path
}

pub fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    let path = get_config_path();
    let contents = fs::read_to_string(path)?;
    let config: Config = toml::from_str(&contents)?;
    Ok(config)
}
