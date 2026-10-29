use std::path::PathBuf;
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

struct TestEnv {
    root: PathBuf,
}

impl TestEnv {
    fn new(name: &str) -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir()
            .join("tasklynx-tests")
            .join(format!("{name}-{}-{nanos}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        Self { root }
    }

    fn command(&self) -> Command {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_tasklynx"));
        cmd.env("HOME", &self.root)
            .env("XDG_CONFIG_HOME", self.root.join("config"))
            .env("TASKLYNX_HOME", self.root.join("data"))
            .env_remove("TERMUX_VERSION");
        cmd
    }

    fn run(&self, args: &[&str]) -> Output {
        self.command().args(args).output().unwrap()
    }

    fn ok(&self, args: &[&str]) -> String {
        let out = self.run(args);
        assert!(
            out.status.success(),
            "command {:?} failed:\n{}",
            args,
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).into_owned()
    }

    fn fail(&self, args: &[&str]) -> String {
        let out = self.run(args);
        assert!(
            !out.status.success(),
            "command {:?} unexpectedly succeeded",
            args
        );
        String::from_utf8_lossy(&out.stderr).into_owned()
    }
}

impl Drop for TestEnv {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

#[test]
fn help_and_version() {
    let env = TestEnv::new("help");
    let version = env.ok(&["--version"]);
    assert!(version.contains(env!("CARGO_PKG_VERSION")));

    let help = env.ok(&["--help"]);
    for cmd in [
        "add",
        "list",
        "doing",
        "complete",
        "reopen",
        "delete",
        "edit",
        "duplicate",
        "search",
        "today",
        "stats",
        "branch",
        "export",
        "web",
        "board",
        "config",
        "self",
    ] {
        assert!(help.contains(cmd), "help is missing {cmd}");
    }
}

#[test]
fn task_lifecycle() {
    let env = TestEnv::new("lifecycle");
    env.ok(&["init"]);

    let id = env
        .ok(&[
            "add",
            "Write docs",
            "--project",
            "api",
            "-t",
            "docs",
            "-P",
            "high",
            "--due",
            "2026-12-31",
        ])
        .trim()
        .to_string();
    assert_eq!(id.len(), 36, "add should print a full UUID");

    let list = env.ok(&["list"]);
    assert!(list.contains("Write docs"));
    assert!(list.contains("high"));
    assert!(list.contains("api"));

    let prefix = &id[..8];
    assert!(env.ok(&["doing", prefix]).contains("doing"));
    assert!(env
        .ok(&["list", "--status", "doing"])
        .contains("Write docs"));

    env.ok(&["complete", prefix]);
    assert!(env.ok(&["list", "--status", "done"]).contains("Write docs"));

    env.ok(&["reopen", prefix]);
    assert!(env.ok(&["list", "--status", "todo"]).contains("Write docs"));

    env.ok(&["delete", prefix]);
    assert!(!env.ok(&["list"]).contains("Write docs"));

    let stats = env.ok(&["stats"]);
    assert!(stats.contains("tasks=0"));
}

#[test]
fn edit_and_duplicate() {
    let env = TestEnv::new("edit");
    env.ok(&["init"]);
    let id = env
        .ok(&["add", "Original", "--project", "api"])
        .trim()
        .to_string();
    let prefix = &id[..8];

    env.ok(&[
        "edit",
        prefix,
        "--title",
        "Renamed",
        "--description",
        "Details",
        "--priority",
        "urgent",
        "--project",
        "backend",
        "--due",
        "2027-01-01",
        "--add-tag",
        "one",
        "--add-tag",
        "two",
    ]);

    let list = env.ok(&["list", "--project", "backend", "--tag", "one"]);
    assert!(list.contains("Renamed"));
    assert!(env.ok(&["search", "renamed"]).contains("Renamed"));
    assert!(env.ok(&["search", "two"]).contains("Renamed"));

    env.ok(&["edit", prefix, "--remove-tag", "one"]);
    assert!(!env.ok(&["list", "--tag", "one"]).contains("Renamed"));

    let duplicate = env.ok(&["duplicate", prefix]).trim().to_string();
    assert_ne!(duplicate, id);
    assert!(env.ok(&["list"]).contains("Copy: Renamed"));
}

#[test]
fn branch_flow() {
    let env = TestEnv::new("branches");
    env.ok(&["init"]);

    env.ok(&["branch", "create", "feature"]);
    assert!(env.ok(&["branch", "list"]).contains("feature"));

    env.ok(&["branch", "switch", "feature"]);
    let id = env.ok(&["add", "Feature work"]).trim().to_string();
    assert!(env.ok(&["list"]).contains("Feature work"));

    env.ok(&["branch", "switch", "main"]);
    assert!(!env.ok(&["list"]).contains("Feature work"));
    assert!(env.ok(&["list", "--all"]).contains("Feature work"));

    let err = env.fail(&["branch", "delete", "feature"]);
    assert!(err.contains("not empty"));

    let current = env.fail(&["branch", "delete", "main"]);
    assert!(current.contains("current"));

    env.ok(&["delete", &id[..8]]);
    env.ok(&["branch", "delete", "feature"]);
    assert!(!env.ok(&["branch", "list"]).contains("feature"));
}

#[test]
fn export_and_config() {
    let env = TestEnv::new("export");
    env.ok(&["init"]);
    env.ok(&["add", "Exported task"]);

    let json = env.ok(&["export", "--format", "json"]);
    assert!(json.contains("\"tasks\""));
    assert!(json.contains("Exported task"));

    let md = env.ok(&["export", "--format", "md"]);
    assert!(md.contains("# Tasklynx"));
    assert!(md.contains("**Exported task**"));

    let path = env.ok(&["config", "init"]).trim().to_string();
    let normalized = path.replace('\\', "/");
    assert!(normalized.ends_with("peekstore/tasklynx/config.toml"));
    assert!(std::path::Path::new(&path).exists());

    let shown = env.ok(&["config", "show"]);
    assert!(shown.contains("port = 8787"));
}

#[test]
fn today_reports_overdue() {
    let env = TestEnv::new("today");
    env.ok(&["init"]);
    env.ok(&["add", "Old task", "--due", "2000-01-01"]);

    let out = env.ok(&["today"]);
    assert!(out.contains("OVERDUE"));
    assert!(out.contains("Old task"));
}

#[test]
fn invalid_due_date_fails() {
    let env = TestEnv::new("bad-date");
    env.ok(&["init"]);
    let err = env.fail(&["add", "Bad task", "--due", "not-a-date"]);
    assert!(!err.is_empty());
}

#[test]
fn partial_config_is_accepted() {
    let env = TestEnv::new("partial-config");
    let dir = env.root.join("config").join("peekstore").join("tasklynx");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("config.toml"), "port = 9999\n").unwrap();

    let shown = env.ok(&["config", "show"]);
    assert!(shown.contains("port = 9999"));
    assert!(shown.contains("host = \"127.0.0.1\""));
}

#[test]
fn completions_and_man_page() {
    let env = TestEnv::new("docs-gen");
    let comp = env.ok(&["completions", "bash"]);
    assert!(comp.contains("complete -F"));
    assert!(comp.contains("tasklynx"));

    let man = env.ok(&["man"]);
    assert!(man.contains(".TH tasklynx"));
}

#[test]
fn import_flow() {
    let env = TestEnv::new("import");
    env.ok(&["init"]);

    let md_file = env.root.join("tasks.md");
    std::fs::write(
        &md_file,
        "# Tasks\n\n## todo\n- [ ] **Imported md task** `high` [backend] #cool\n",
    )
    .unwrap();

    let out = env.ok(&["import", md_file.to_str().unwrap()]);
    assert!(out.contains("imported: 1 new"));

    let list = env.ok(&["list"]);
    assert!(list.contains("Imported md task"));
    assert!(list.contains("high"));
    assert!(list.contains("backend"));
}

#[test]
fn version_matches_dot_version() {
    let dot_version = std::fs::read_to_string(".version")
        .unwrap()
        .trim()
        .to_string();
    assert_eq!(dot_version, env!("CARGO_PKG_VERSION"));
}
