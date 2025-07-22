use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

use anyhow::Error;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Config {
    pub library_path: Option<PathBuf>,
    pub discord_presence: bool,
}

impl Config {
    pub fn set_library(&mut self) -> Option<PathBuf> {
        if let Some(path) = rfd::FileDialog::new().pick_folder() {
            self.library_path = Some(path.clone());

            if let Err(e) = save_config(self) {
                eprintln!("unable to load config file: {e}");
            };

            Some(path)
        } else {
            None
        }
    }

    pub fn set_discord_presence(&mut self, enabled: bool) {
        self.discord_presence = enabled;

        if let Err(e) = save_config(self) {
            eprintln!("unable to save config: {e}");
        };
    }
}

fn get_config_path() -> anyhow::Result<PathBuf> {
    let home_dir = match dirs::home_dir() {
        Some(d) => d,
        None => return Err(Error::msg("unable to determine home directory")),
    };

    Ok(home_dir.join(".config/attention/attention.json"))
}

pub fn save_config(config: &Config) -> anyhow::Result<()> {
    let config_path = get_config_path()?;
    let config_dir = match config_path.parent() {
        Some(d) => d,
        None => return Err(Error::msg("unable to determine config directory")),
    };

    fs::create_dir_all(config_dir)?;
    let mut file = File::create(config_path)?;
    file.write_all(serde_json::to_string_pretty(&config)?.as_bytes())?;

    Ok(())
}

pub fn load_config() -> anyhow::Result<Config> {
    let config_path = get_config_path()?;
    let serialized = fs::read_to_string(config_path)?;
    Ok(serde_json::from_str(&serialized)?)
}
