use crate::model::AppConfig;
use anyhow::Result;
use std::{fs, path::PathBuf};

pub fn dir() -> PathBuf {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".config")
        });
    base.join("peekstore").join("tasklynx")
}
pub fn path() -> PathBuf {
    dir().join("config.toml")
}
pub fn load() -> Result<AppConfig> {
    let p = path();
    if !p.exists() {
        return Ok(AppConfig::default());
    }
    Ok(toml::from_str(&fs::read_to_string(p)?)?)
}
pub fn save(c: &AppConfig) -> Result<()> {
    fs::create_dir_all(dir())?;
    fs::write(path(), toml::to_string_pretty(c)?)?;
    Ok(())
}
