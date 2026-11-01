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
    if c.ai.provider.trim().is_empty() {
        anyhow::bail!("config: ai.provider must not be empty");
    }
    if c.ai.model.trim().is_empty() {
        anyhow::bail!("config: ai.model must not be empty");
    }
    if c.ai.endpoint.trim().is_empty() {
        anyhow::bail!("config: ai.endpoint must not be empty");
    }
    if c.ai.max_context_tasks == 0 {
        anyhow::bail!("config: ai.max_context_tasks must be at least 1");
    }
    for name in c.views.keys() {
        if name.trim().is_empty() {
            anyhow::bail!("config: view names must not be empty");
        }
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
        assert!(c.views.is_empty());
        assert!(c.git.auto_branch);
        assert_eq!(c.git.branch_prefix, "tx/");
        assert!(c.git.link_commits);
        assert!(!c.git.hooks);
        assert_eq!(c.ai.provider, "none");
    }

    #[test]
    fn partial_git_config_uses_defaults() {
        let c: AppConfig = toml::from_str("[git]\nlink_commits = false\n").unwrap();
        assert!(c.git.auto_branch);
        assert_eq!(c.git.branch_prefix, "tx/");
        assert!(!c.git.link_commits);
        assert!(!c.git.hooks);
    }

    #[test]
    fn partial_ai_config_uses_defaults() {
        let c: AppConfig = toml::from_str("[ai]\nprovider = \"ollama\"\n").unwrap();
        assert_eq!(c.ai.provider, "ollama");
        assert_eq!(c.ai.model, AppConfig::default().ai.model);
        assert!(c.ai.redact);
    }

    #[test]
    fn parses_saved_views() {
        let c: AppConfig =
            toml::from_str("[views.api]\nproject = \"api\"\ntag = \"backend\"\nall = true\n")
                .unwrap();
        let view = &c.views["api"];
        assert_eq!(view.project.as_deref(), Some("api"));
        assert_eq!(view.tag.as_deref(), Some("backend"));
        assert!(view.all);
    }

    #[test]
    fn rejects_empty_view_names() {
        let mut c = AppConfig::default();
        c.views.insert(
            "  ".into(),
            crate::model::SavedView {
                ..crate::model::SavedView::default()
            },
        );
        assert!(validate(&c).is_err());
    }

    #[test]
    fn rejects_empty_ai_provider() {
        let mut c = AppConfig::default();
        c.ai.provider = " ".into();
        assert!(validate(&c).is_err());
    }

    #[test]
    fn rejects_zero_ai_context_limit() {
        let mut c = AppConfig::default();
        c.ai.max_context_tasks = 0;
        assert!(validate(&c).is_err());
    }
}
