use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::util::e;

#[derive(Serialize, Deserialize, Clone)]
pub struct Settings {
    #[serde(default)]
    pub client_dir: Option<String>,
    #[serde(default)]
    pub server_dir: Option<String>,
    #[serde(default = "yes")]
    pub autologin: bool,
    #[serde(default = "eight")]
    pub autologin_delay: u64,
}

fn yes() -> bool {
    true
}
fn eight() -> u64 {
    8
}

impl Default for Settings {
    fn default() -> Self {
        Settings { client_dir: None, server_dir: None, autologin: true, autologin_delay: 8 }
    }
}

pub fn config_dir() -> PathBuf {
    let base = std::env::var_os("APPDATA")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("RA Realm Launcher")
}

/// By default the server files live in a folder named "server" next to the launcher program.
pub fn default_server_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|x| x.join("server")))
        .unwrap_or_else(|| PathBuf::from("server"))
}

impl Settings {
    pub fn load() -> Self {
        fs::read_to_string(config_dir().join("settings.json"))
            .ok()
            .and_then(|t| serde_json::from_str(&t).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> Result<(), String> {
        let dir = config_dir();
        fs::create_dir_all(&dir).map_err(e)?;
        let text = serde_json::to_string_pretty(self).map_err(e)?;
        fs::write(dir.join("settings.json"), text).map_err(e)
    }

    pub fn server_root(&self) -> PathBuf {
        match &self.server_dir {
            Some(d) if !d.trim().is_empty() => PathBuf::from(d),
            _ => default_server_dir(),
        }
    }
}
