use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct GitInfo {
    pub root: PathBuf,
    pub branch: String,
    pub commit: Option<String>,
    pub remote: Option<String>,
}

pub fn discover() -> Option<GitInfo> {
    GitInfo::discover()
}

impl GitInfo {
    pub fn discover() -> Option<Self> {
        let root = git_output(&["rev-parse", "--show-toplevel"])?;
        let branch = git_output(&["rev-parse", "--abbrev-ref", "HEAD"])
            .unwrap_or_else(|| "HEAD".to_string());
        let commit = git_output(&["rev-parse", "--short=12", "HEAD"]);
        let remote = git_output(&["remote", "get-url", "origin"]);
        Some(Self {
            root: PathBuf::from(root),
            branch,
            commit,
            remote,
        })
    }
}

fn git_output(args: &[&str]) -> Option<String> {
    let output = Command::new("git").args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8(output.stdout).ok()?.trim().to_string();
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}
