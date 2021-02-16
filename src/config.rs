use failure::{format_err, Error};
use indexmap::IndexMap;
use log::{debug, trace};
use serde_derive::Deserialize;
use std::{io::Read, path::PathBuf};

#[derive(Debug, Deserialize)]
pub struct Config {
    pub default_icon: String,
    pub icon_separator: String,
    pub icons: IndexMap<String, String>,
}

impl Default for Config {
    fn default() -> Self {
        trace!("Using default config.");
        let content = std::include_str!("../default.toml");
        toml::from_str(&content).expect("Parsing default works.")
    }
}

impl Config {
    pub fn load(path: Option<PathBuf>) -> Result<Self, Error> {
        let cfg_path: PathBuf = if let Some(p) = path {
            p
        } else {
            dirs::config_dir()
                .ok_or_else(|| format_err!("Can't access default config dir."))?
                .join(env!("CARGO_PKG_NAME"))
                .join("config.toml")
        };
        debug!("Loading config file '{}'", &cfg_path.to_string_lossy());
        let mut f = std::fs::File::open(cfg_path)?;
        let mut content = String::new();
        f.read_to_string(&mut content)?;
        toml::from_str(&content).map_err(|e| format_err!("Failed to parse config: {}", e))
    }
}
