use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WorkspaceState {
    pub changed_files: usize,
    pub untracked_files: usize,
}

impl WorkspaceState {
    pub fn is_dirty(&self) -> bool {
        self.changed_files > 0 || self.untracked_files > 0
    }
}

#[derive(Debug, Clone)]
pub struct GitInfo {
    pub root: PathBuf,
    pub git_dir: PathBuf,
    pub branch: String,
    pub commit: Option<String>,
    pub remote: Option<String>,
    pub workspace: WorkspaceState,
}

#[derive(Debug, Clone)]
pub struct CommitInfo {
    pub sha: String,
    pub body: String,
}

pub fn discover() -> Option<GitInfo> {
    GitInfo::discover()
}

impl GitInfo {
    pub fn discover() -> Option<Self> {
        let root = git_output(&["rev-parse", "--show-toplevel"])?;
        let git_dir = git_output(&["rev-parse", "--absolute-git-dir"])?;
        let branch = git_output(&["rev-parse", "--abbrev-ref", "HEAD"])
            .unwrap_or_else(|| "HEAD".to_string());
        let commit = git_output(&["rev-parse", "--short=12", "HEAD"]);
        let remote = git_output(&["remote", "get-url", "origin"]);
        Some(Self {
            root: PathBuf::from(root),
            git_dir: PathBuf::from(git_dir),
            branch,
            commit,
            remote,
            workspace: workspace_state(),
        })
    }

    pub fn hooks_dir(&self) -> PathBuf {
        self.git_dir.join("hooks")
    }
}

pub fn task_branch(git_branch: &str, prefix: &str) -> Option<String> {
    let branch = git_branch.trim();
    if branch.is_empty() || branch == "HEAD" {
        return None;
    }
    let prefix = prefix.trim();
    let mapped = if prefix.is_empty() {
        branch
    } else {
        branch.strip_prefix(prefix).unwrap_or(branch)
    };
    let mapped = mapped.trim();
    if mapped.is_empty() {
        None
    } else {
        Some(mapped.to_string())
    }
}

fn workspace_state() -> WorkspaceState {
    let Ok(output) = Command::new("git").args(["status", "--porcelain"]).output() else {
        return WorkspaceState::default();
    };
    if !output.status.success() {
        return WorkspaceState::default();
    }
    parse_workspace_status(&String::from_utf8_lossy(&output.stdout))
}

fn parse_workspace_status(output: &str) -> WorkspaceState {
    let mut state = WorkspaceState::default();
    for line in output.lines() {
        if line.len() < 2 {
            continue;
        }
        if line.starts_with("??") {
            state.untracked_files += 1;
        } else {
            state.changed_files += 1;
        }
    }
    state
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

pub fn tracked_files() -> Option<Vec<PathBuf>> {
    let output = Command::new("git")
        .args(["ls-files", "-z", "--full-name"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let mut files = Vec::new();
    for part in output.stdout.split(|byte| *byte == 0) {
        if part.is_empty() {
            continue;
        }
        if let Ok(path) = std::str::from_utf8(part) {
            files.push(PathBuf::from(path));
        }
    }
    Some(files)
}

pub fn recent_commits(limit: usize) -> Option<Vec<CommitInfo>> {
    let limit = limit.to_string();
    let raw = git_output(&["log", "-n", &limit, "--format=%H%x1f%B%x1e"])?;
    let mut commits = Vec::new();
    for record in raw.split('\u{1e}') {
        let record = record.trim();
        if record.is_empty() {
            continue;
        }
        let (sha, body) = record.split_once('\u{1f}')?;
        commits.push(CommitInfo {
            sha: sha.trim().to_string(),
            body: body.trim().to_string(),
        });
    }
    Some(commits)
}

pub fn remote_web_url(remote: &str) -> Option<String> {
    let remote = remote.trim();
    if remote.is_empty() {
        return None;
    }
    let remote = remote.strip_suffix(".git").unwrap_or(remote);

    if let Some(rest) = remote.strip_prefix("git@") {
        let (host, path) = rest.split_once(':')?;
        if host.is_empty() || path.is_empty() {
            return None;
        }
        return Some(format!("https://{host}/{}", path.trim_matches('/')));
    }

    if let Some(rest) = remote.strip_prefix("ssh://") {
        let rest = rest.split_once('@').map_or(rest, |(_, value)| value);
        let (host, path) = rest.split_once('/')?;
        if host.is_empty() || path.is_empty() {
            return None;
        }
        return Some(format!("https://{host}/{}", path.trim_matches('/')));
    }

    if remote.starts_with("https://") || remote.starts_with("http://") {
        return Some(remote.to_string());
    }

    None
}

pub fn commit_url(remote: &str, commit: &str) -> Option<String> {
    let base = remote_web_url(remote)?;
    let commit = commit.trim();
    if commit.is_empty() {
        return None;
    }
    if base.contains("github.com") {
        Some(format!("{base}/commit/{commit}"))
    } else if base.contains("gitlab") {
        Some(format!("{base}/-/commit/{commit}"))
    } else if base.contains("bitbucket.org") {
        Some(format!("{base}/commits/{commit}"))
    } else {
        Some(format!("{base}/commit/{commit}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_scp_style_remotes() {
        assert_eq!(
            remote_web_url("git@github.com:peekstore/tasklynx.git").as_deref(),
            Some("https://github.com/peekstore/tasklynx")
        );
        assert_eq!(
            remote_web_url("git@gitlab.com:team/project.git").as_deref(),
            Some("https://gitlab.com/team/project")
        );
        assert_eq!(
            remote_web_url("git@bitbucket.org:team/project.git").as_deref(),
            Some("https://bitbucket.org/team/project")
        );
    }

    #[test]
    fn parses_https_and_ssh_remotes() {
        assert_eq!(
            remote_web_url("https://github.com/peekstore/tasklynx.git").as_deref(),
            Some("https://github.com/peekstore/tasklynx")
        );
        assert_eq!(
            remote_web_url("ssh://git@github.com/peekstore/tasklynx.git").as_deref(),
            Some("https://github.com/peekstore/tasklynx")
        );
    }

    #[test]
    fn parses_porcelain_workspace_state() {
        let state = parse_workspace_status(" M src/main.rs\n?? new.txt\nA  added.txt\n");
        assert_eq!(state.changed_files, 2);
        assert_eq!(state.untracked_files, 1);
        assert!(state.is_dirty());
        assert_eq!(parse_workspace_status(""), WorkspaceState::default());
        assert!(!WorkspaceState::default().is_dirty());
    }

    #[test]
    fn maps_git_branches_to_task_branches() {
        assert_eq!(
            task_branch("tx/feature-auth", "tx/").as_deref(),
            Some("feature-auth")
        );
        assert_eq!(
            task_branch("feature/auth", "tx/").as_deref(),
            Some("feature/auth")
        );
        assert_eq!(task_branch("main", "tx/").as_deref(), Some("main"));
        assert!(task_branch("HEAD", "tx/").is_none());
        assert!(task_branch("tx/", "tx/").is_none());
    }

    #[test]
    fn rejects_unknown_remotes() {
        assert!(remote_web_url("").is_none());
        assert!(remote_web_url("/local/path/repo").is_none());
    }

    #[test]
    fn builds_commit_urls_for_hosts() {
        assert_eq!(
            commit_url("git@github.com:peekstore/tasklynx.git", "abc123").as_deref(),
            Some("https://github.com/peekstore/tasklynx/commit/abc123")
        );
        assert_eq!(
            commit_url("git@gitlab.com:team/project.git", "abc123").as_deref(),
            Some("https://gitlab.com/team/project/-/commit/abc123")
        );
        assert_eq!(
            commit_url("git@bitbucket.org:team/project.git", "abc123").as_deref(),
            Some("https://bitbucket.org/team/project/commits/abc123")
        );
    }
}
