//! Configuration handling for piow.
//!
//! Parsing the toml file format into a `Config` struct.
use failure::{format_err, Error};
use indexmap::IndexMap;
use log::{debug, warn};
use serde_derive::Deserialize;
use std::{io::Read, path::PathBuf};

/// Placeholder for workspace number.
const NUM: &str = "%n";
/// Placeholder for Icon.
const ICON: &str = "%i";

/// Configuration for icon to workspace mappings.
#[derive(Debug, Deserialize)]
pub struct Config {
    /// Literal/Icon used for applications not in the map.
    pub default_icon: String,
    /// Formatting string of the new name. Supports `%n` and `%i` placeholders for workspace number
    /// and icon, respectively.
    format_str: String,
    /// Literal for separating icons.
    pub icon_separator: String,
    /// Application name to icon literal name.
    pub icons: IndexMap<String, String>,
}

impl Default for Config {
    fn default() -> Self {
        warn!("Using default config.");
        let content = std::include_str!("../default.toml");
        toml::from_str(&content).expect("Parsing default works.")
    }
}

impl Config {
    /// Load the configuration file from either `path` or the default config dir
    /// `${XDG_CONFIG_HOME}`.
    ///
    /// Error: File does not exist or can't be opened, syntax errors and other parsing errors.
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

    /// Generate the workspace name from the format string in the config by replacing all
    /// placeholders with their desired values.
    ///
    /// Currently supported placeholders:
    ///
    ///  * `%n`: workspace number
    ///  * `%i`: icon
    ///
    /// Example:
    ///
    /// ```text,ignore
    /// # Workspace 1 with icons   
    /// "%n: %i" -> '1:   '
    /// ```
    pub fn format(&self, num: String, icon: String) -> String {
        self.format_str.replace(NUM, &num).replace(ICON, &icon)
    }
}
