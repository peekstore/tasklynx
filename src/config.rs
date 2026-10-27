use anyhow::Result;
use std::{fs, path::PathBuf};
use crate::model::AppConfig;

pub fn dir() -> PathBuf { dirs::config_dir().unwrap_or_else(|| PathBuf::from(".")) .join("tasklynx") }
pub fn path() -> PathBuf { dir().join("config.toml") }
pub fn load() -> Result<AppConfig> { let p=path(); if !p.exists() { return Ok(AppConfig::default()) } Ok(toml::from_str(&fs::read_to_string(p)?)?) }
pub fn save(c: &AppConfig) -> Result<()> { fs::create_dir_all(dir())?; fs::write(path(), toml::to_string_pretty(c)?)?; Ok(()) }
