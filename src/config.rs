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
    let config: AppConfig = toml::from_str(&fs::read_to_string(p)?)?;
    validate(&config)?;
    Ok(config)
}
pub fn save(c: &AppConfig) -> Result<()> {
    validate(c)?;
    fs::create_dir_all(dir())?;
    fs::write(path(), toml::to_string_pretty(c)?)?;
    Ok(())
}
pub fn validate(c: &AppConfig) -> Result<()> {
    if c.host.trim().is_empty() {
        anyhow::bail!("config: host must not be empty");
    }
    if c.port == 0 {
        anyhow::bail!("config: port must be between 1 and 65535");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_valid() {
        validate(&AppConfig::default()).unwrap();
    }

    #[test]
    fn rejects_empty_host() {
        let c = AppConfig {
            host: "  ".into(),
            ..AppConfig::default()
        };
        assert!(validate(&c).is_err());
    }

    #[test]
    fn rejects_port_zero() {
        let c = AppConfig {
            port: 0,
            ..AppConfig::default()
        };
        assert!(validate(&c).is_err());
    }

    #[test]
    fn partial_config_uses_defaults() {
        let c: AppConfig = toml::from_str("port = 9999").unwrap();
        assert_eq!(c.port, 9999);
        assert_eq!(c.host, AppConfig::default().host);
    }
}
