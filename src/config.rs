use once_cell::sync::Lazy;
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Deserialize, Default)]
#[serde(default)]
pub struct Config {
    pub auto_invite: bool,
    pub auto_ban: bool,
    pub group_id: Option<String>,
    pub avatars_file: Option<String>,
    pub custom_log_dir: Option<String>,
}

pub static CONFIG: Lazy<Config> = Lazy::new(|| {
    let config_path = Path::new("config.toml");

    match fs::read_to_string(config_path) {
        Ok(contents) => toml::from_str(&contents).unwrap_or_else(|e| {
            eprintln!("Failed to parse config.toml: {e}");
            Config::default()
        }),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            eprintln!("config.toml not found, using default configuration");
            Config::default()
        }
        Err(e) => {
            eprintln!("Failed to read config.toml: {e}");
            Config::default()
        }
    }
});
