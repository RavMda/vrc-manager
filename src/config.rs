use once_cell::sync::Lazy;
use serde::Deserialize;
use std::fs;
use std::path::Path;
use tracing::error;

#[derive(Deserialize, Default, Debug)]
#[serde(default)]
pub struct AutoInvite {
    pub enabled: bool,
    pub delay_min: u64,
    pub delay_max: u64,
}

#[derive(Deserialize, Default, Debug)]
#[serde(default)]
pub struct AutoBan {
    pub enabled: bool,
    pub log_avatar_id: bool,
}

#[derive(Deserialize, Default, Debug)]
#[serde(default)]
pub struct Config {
    pub auto_invite: AutoInvite,
    pub auto_ban: AutoBan,
    pub group_id: Option<String>,
    pub avatars_file: Option<String>,
    pub custom_log_dir: Option<String>,
}

pub static CONFIG: Lazy<Config> = Lazy::new(|| {
    let config_path = Path::new("config.toml");

    match fs::read_to_string(config_path) {
        Ok(contents) => toml::from_str(&contents).unwrap_or_else(|e| {
            error!("Failed to parse config.toml: {e}");
            Config::default()
        }),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            error!("config.toml not found, using default configuration");
            Config::default()
        }
        Err(e) => {
            error!("Failed to read config.toml: {e}");
            Config::default()
        }
    }
});
