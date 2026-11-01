use chrono::{Duration, Local};
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
        cmd.current_dir(&self.root)
            .env("HOME", &self.root)
            .env("XDG_CONFIG_HOME", self.root.join("config"))
            .env("TASKLYNX_HOME", self.root.join("data"))
            .env_remove("TERMUX_VERSION");
        cmd
    }

    fn run(&self, args: &[&str]) -> Output {
        self.command().args(args).output().unwrap()
    }

    fn command_in(&self, dir: &std::path::Path) -> Command {
        let mut cmd = self.command();
        cmd.current_dir(dir);
        cmd
    }

    fn ok_in(&self, dir: &std::path::Path, args: &[&str]) -> String {
        let out = self.command_in(dir).args(args).output().unwrap();
        assert!(
            out.status.success(),
            "command {:?} failed:\n{}",
            args,
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).into_owned()
    }

    fn fail_in(&self, dir: &std::path::Path, args: &[&str]) -> String {
        let out = self.command_in(dir).args(args).output().unwrap();
        assert!(
            !out.status.success(),
            "command {:?} unexpectedly succeeded",
            args
        );
        String::from_utf8_lossy(&out.stderr).into_owned()
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
        "start",
        "stop",
        "time",
        "view",
        "duplicates",
        "complete",
        "reopen",
        "delete",
        "undo",
        "next",
        "block",
        "unblock",
        "subtask",
        "tree",
        "trash",
        "git",
        "ai",
        "edit",
        "duplicate",
        "search",
        "today",
        "agenda",
        "remind",
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

    let due = (Local::now().date_naive() + Duration::days(2))
        .format("%Y-%m-%d")
        .to_string();
    let ics_date = (Local::now().date_naive() + Duration::days(2))
        .format("%Y%m%d")
        .to_string();
    env.ok(&["add", "Calendar task", "--due", due.as_str()]);
    let ics = env.ok(&["export", "--format", "ics"]);
    assert!(ics.contains("BEGIN:VCALENDAR"), "{ics}");
    assert!(ics.contains("SUMMARY:Calendar task"), "{ics}");
    assert!(
        ics.contains(&format!("DTSTART;VALUE=DATE:{ics_date}")),
        "{ics}"
    );
    assert!(!ics.contains("Exported task"), "{ics}");
    let err = env.fail(&["export", "--format", "bad"]);
    assert!(err.contains("unsupported export format"), "{err}");

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

#[test]
fn recurring_task_completion_creates_next_occurrence() {
    let env = TestEnv::new("recurring");
    env.ok(&["init"]);
    let id = env
        .ok(&[
            "add",
            "Daily standup",
            "--due",
            "2026-01-01",
            "--repeat",
            "daily",
        ])
        .trim()
        .to_string();
    let prefix = &id[..8];

    assert!(env.ok(&["list"]).contains("daily"));

    let first = env.ok(&["complete", prefix]);
    assert!(first.contains("next"), "{first}");
    let next_id = first.split("next ").nth(1).unwrap().trim().to_string();
    assert_eq!(next_id.len(), 8);
    assert!(env
        .ok(&["list", "--status", "todo"])
        .contains("Daily standup"));

    let again = env.ok(&["complete", prefix]);
    assert!(!again.contains("next"), "{again}");
    assert!(env.ok(&["stats"]).contains("tasks=2"));

    let second = env.ok(&["complete", &next_id]);
    assert!(second.contains("next"), "{second}");
    assert!(env.ok(&["stats"]).contains("tasks=3"));
}

#[test]
fn repeat_until_stops_follow_up_generation() {
    let env = TestEnv::new("repeat-until");
    env.ok(&["init"]);
    let id = env
        .ok(&[
            "add",
            "Limited series",
            "--due",
            "2026-01-01",
            "--repeat",
            "daily",
            "--repeat-until",
            "2026-01-01",
        ])
        .trim()
        .to_string();

    let out = env.ok(&["complete", &id[..8]]);
    assert!(!out.contains("next"), "{out}");
    assert!(env.ok(&["stats"]).contains("tasks=1"));
}

#[test]
fn edit_can_set_and_clear_recurrence() {
    let env = TestEnv::new("edit-repeat");
    env.ok(&["init"]);
    let id = env.ok(&["add", "Repeat edit"]).trim().to_string();
    let prefix = &id[..8];

    env.ok(&["edit", prefix, "--repeat", "weekly"]);
    assert!(env.ok(&["list"]).contains("↻ weekly"));

    env.ok(&["edit", prefix, "--repeat", "none"]);
    assert!(!env.ok(&["list"]).contains("↻"));
}

#[test]
fn invalid_repeat_fails() {
    let env = TestEnv::new("bad-repeat");
    env.ok(&["init"]);
    let err = env.fail(&["add", "Bad repeat", "--repeat", "hourly"]);
    assert!(err.contains("invalid repeat"), "{err}");
}

#[test]
fn agenda_lists_upcoming_overdue_and_recurring() {
    let env = TestEnv::new("agenda");
    env.ok(&["init"]);

    let due = (Local::now().date_naive() + Duration::days(2))
        .format("%Y-%m-%d")
        .to_string();
    env.ok(&[
        "add",
        "Upcoming task",
        "--due",
        due.as_str(),
        "--repeat",
        "daily",
    ]);
    env.ok(&["add", "Old task", "--due", "2000-01-01"]);

    let out = env.ok(&["agenda", "--days", "7"]);
    assert!(out.contains("UPCOMING"), "{out}");
    assert!(out.contains("NEXT"), "{out}");
    assert!(out.contains("OVERDUE"), "{out}");
    assert!(out.contains("Upcoming task"), "{out}");
    assert!(out.contains("Old task"), "{out}");
}

#[test]
fn remind_lists_due_tasks_and_honors_window() {
    let env = TestEnv::new("remind");
    env.ok(&["init"]);

    let today = Local::now().date_naive();
    let yesterday = (today - Duration::days(1)).format("%Y-%m-%d").to_string();
    let today_str = today.format("%Y-%m-%d").to_string();
    let soon = (today + Duration::days(2)).format("%Y-%m-%d").to_string();
    let far = (today + Duration::days(10)).format("%Y-%m-%d").to_string();

    env.ok(&["add", "Overdue reminder", "--due", yesterday.as_str()]);
    env.ok(&["add", "Today reminder", "--due", today_str.as_str()]);
    env.ok(&["add", "Soon reminder", "--due", soon.as_str()]);
    env.ok(&["add", "Far reminder", "--due", far.as_str()]);
    env.ok(&["add", "No due reminder"]);
    let done = env
        .ok(&["add", "Done reminder", "--due", today_str.as_str()])
        .trim()
        .to_string();
    env.ok(&["complete", &done[..8]]);

    let out = env.ok(&["remind", "--days", "3"]);
    assert!(out.contains("OVERDUE"), "{out}");
    assert!(out.contains("TODAY"), "{out}");
    assert!(out.contains("UPCOMING"), "{out}");
    assert!(out.contains("Overdue reminder"), "{out}");
    assert!(out.contains("Today reminder"), "{out}");
    assert!(out.contains("Soon reminder"), "{out}");
    assert!(!out.contains("Far reminder"), "{out}");
    assert!(!out.contains("No due reminder"), "{out}");
    assert!(!out.contains("Done reminder"), "{out}");

    let today_only = env.ok(&["remind", "--days", "0"]);
    assert!(today_only.contains("Overdue reminder"), "{today_only}");
    assert!(today_only.contains("Today reminder"), "{today_only}");
    assert!(!today_only.contains("Soon reminder"), "{today_only}");

    let err = env.fail(&["remind", "--days", "99999"]);
    assert!(err.contains("--days must be between 0 and 3650"), "{err}");
}

#[cfg(target_os = "linux")]
#[test]
fn remind_sends_desktop_notification() {
    use std::os::unix::fs::PermissionsExt;

    let env = TestEnv::new("remind-notify");
    env.ok(&["init"]);
    let today = Local::now().date_naive().format("%Y-%m-%d").to_string();
    env.ok(&["add", "Notify me", "--due", today.as_str()]);

    let bin = env.root.join("bin");
    std::fs::create_dir_all(&bin).unwrap();
    let script = bin.join("notify-send");
    std::fs::write(
        &script,
        "#!/bin/sh\nprintf '%s\\n' \"$@\" > \"$HOME/notify.args\"\n",
    )
    .unwrap();
    let mut permissions = std::fs::metadata(&script).unwrap().permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(&script, permissions).unwrap();

    let path = format!(
        "{}:{}",
        bin.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let out = env
        .command()
        .env("PATH", path)
        .args(["remind", "--notify"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "remind --notify failed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let notification = std::fs::read_to_string(env.root.join("notify.args")).unwrap();
    assert!(
        notification.contains("Tasklynx: 1 task(s) due"),
        "{notification}"
    );
    assert!(notification.contains("Notify me"), "{notification}");
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("notification sent"),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn time_tracking_flow() {
    let env = TestEnv::new("time");
    env.ok(&["init"]);
    let id = env
        .ok(&["add", "Timed task", "--estimate", "60"])
        .trim()
        .to_string();
    let prefix = &id[..8];

    let started = env.ok(&["start", prefix]);
    assert!(started.contains("started"), "{started}");
    assert!(started.contains("Timed task"), "{started}");
    assert!(env
        .ok(&["list", "--status", "doing"])
        .contains("Timed task"));

    let active = env.ok(&["time"]);
    assert!(active.contains("ACTIVE"), "{active}");
    assert!(active.contains("Timed task"), "{active}");
    assert!(active.contains("est 1h"), "{active}");

    let detail = env.ok(&["time", prefix]);
    assert!(detail.contains("estimate  1h"), "{detail}");
    assert!(detail.contains("actual"), "{detail}");
    assert!(detail.contains("-> active"), "{detail}");

    let err = env.fail(&["start", prefix]);
    assert!(err.contains("timer already running"), "{err}");

    let stopped = env.ok(&["stop"]);
    assert!(stopped.contains("stopped"), "{stopped}");
    assert!(stopped.contains("Timed task"), "{stopped}");

    let after = env.ok(&["time", prefix]);
    assert!(after.contains("estimate  1h"), "{after}");
    assert!(!after.contains("-> active"), "{after}");

    let err = env.fail(&["stop"]);
    assert!(err.contains("no running timer"), "{err}");

    env.ok(&["start", prefix]);
    env.ok(&["complete", prefix]);
    let completed = env.ok(&["time", prefix]);
    assert!(!completed.contains("-> active"), "{completed}");
    let err = env.fail(&["start", prefix]);
    assert!(err.contains("completed task"), "{err}");

    let duplicate = env.ok(&["duplicate", prefix]).trim().to_string();
    let duplicate_time = env.ok(&["time", &duplicate[..8]]);
    assert!(
        duplicate_time.contains("no time entries"),
        "{duplicate_time}"
    );
}

#[test]
fn templates_apply_defaults_and_allow_overrides() {
    let env = TestEnv::new("templates");
    env.ok(&["init"]);

    let id = env.ok(&["add", "--template", "review"]).trim().to_string();
    assert_eq!(id.len(), 36);
    let list = env.ok(&["list"]);
    assert!(list.contains("Review changes"), "{list}");
    assert!(list.contains("high"), "{list}");
    assert!(list.contains("default"), "{list}");
    assert!(env
        .ok(&["list", "--tag", "review"])
        .contains("Review changes"));
    assert!(env
        .ok(&["export", "--format", "json"])
        .contains("Review checklist"));

    let custom = env
        .ok(&[
            "add",
            "Custom audit",
            "--template",
            "review",
            "--priority",
            "low",
            "--tag",
            "custom",
        ])
        .trim()
        .to_string();
    assert_eq!(custom.len(), 36);
    let custom_list = env.ok(&["list", "--tag", "custom"]);
    assert!(custom_list.contains("Custom audit"), "{custom_list}");
    assert!(custom_list.contains("low"), "{custom_list}");

    let err = env.fail(&["add", "--template", "missing"]);
    assert!(err.contains("unknown template"), "{err}");
    assert!(err.contains("review"), "{err}");
    let err = env.fail(&["add"]);
    assert!(err.contains("title is required"), "{err}");
}

#[test]
fn saved_views_filter_run_and_delete() {
    let env = TestEnv::new("views");
    env.ok(&["init"]);
    env.ok(&["add", "API task", "--project", "api", "--tag", "backend"]);
    env.ok(&["add", "Web task", "--project", "web"]);

    env.ok(&["view", "save", "api", "--project", "api"]);
    let list = env.ok(&["list", "--view", "api"]);
    assert!(list.contains("API task"), "{list}");
    assert!(!list.contains("Web task"), "{list}");
    let run = env.ok(&["view", "run", "api"]);
    assert!(run.contains("API task"), "{run}");
    assert!(!run.contains("Web task"), "{run}");

    let views = env.ok(&["view", "list"]);
    assert!(views.contains("api"), "{views}");
    assert!(views.contains("project=api"), "{views}");
    let show = env.ok(&["view", "show", "api"]);
    assert!(show.contains("project=api"), "{show}");

    env.ok(&["list", "--save", "todo", "--status", "todo"]);
    let views = env.ok(&["view", "list"]);
    assert!(views.contains("todo"), "{views}");
    let todo = env.ok(&["list", "--view", "todo"]);
    assert!(todo.contains("API task"), "{todo}");
    assert!(todo.contains("Web task"), "{todo}");

    env.ok(&["view", "delete", "api"]);
    let err = env.fail(&["list", "--view", "api"]);
    assert!(err.contains("view not found"), "{err}");
    let err = env.fail(&["view", "delete", "api"]);
    assert!(err.contains("view not found"), "{err}");
}

#[test]
fn bulk_complete_respects_due_dates_and_branches() {
    let env = TestEnv::new("bulk-complete");
    env.ok(&["init"]);
    let today = Local::now().date_naive().format("%Y-%m-%d").to_string();
    let future = (Local::now().date_naive() + Duration::days(10))
        .format("%Y-%m-%d")
        .to_string();

    env.ok(&["add", "Old task", "--due", "2000-01-01"]);
    env.ok(&["add", "Today task", "--due", today.as_str()]);
    env.ok(&["add", "Future task", "--due", future.as_str()]);
    env.ok(&["add", "No due task"]);
    env.ok(&["branch", "create", "feature"]);
    env.ok(&["branch", "switch", "feature"]);
    env.ok(&["add", "Other branch task", "--due", "2000-01-01"]);
    env.ok(&["branch", "switch", "main"]);

    let err = env.fail(&["complete"]);
    assert!(err.contains("provide a task id"), "{err}");

    let out = env.ok(&["complete", "--done-before", today.as_str()]);
    assert!(out.contains("completed 2 task(s)"), "{out}");
    let todo = env.ok(&["list", "--status", "todo"]);
    assert!(todo.contains("Future task"), "{todo}");
    assert!(todo.contains("No due task"), "{todo}");
    assert!(!todo.contains("Old task"), "{todo}");
    let done = env.ok(&["list", "--status", "done"]);
    assert!(done.contains("Old task"), "{done}");
    assert!(done.contains("Today task"), "{done}");
    assert!(!done.contains("Other branch task"), "{done}");

    env.ok(&["undo"]);
    let out = env.ok(&["complete", "--done-before", today.as_str(), "--all"]);
    assert!(out.contains("completed 3 task(s)"), "{out}");
    let done = env.ok(&["list", "--status", "done", "--all"]);
    assert!(done.contains("Other branch task"), "{done}");
}

#[test]
fn bulk_complete_skips_parents_with_open_subtasks() {
    let env = TestEnv::new("bulk-parent");
    env.ok(&["init"]);
    let parent = env
        .ok(&["add", "Parent task", "--due", "2000-01-01"])
        .trim()
        .to_string();
    env.ok(&["subtask", &parent[..8], "Child task"]);

    let out = env.ok(&["complete", "--done-before", "2000-01-02"]);
    assert!(out.contains("skipped 1 task(s)"), "{out}");
    assert!(env.ok(&["list"]).contains("Parent task"));

    let out = env.ok(&["complete", "--done-before", "2000-01-02", "--force"]);
    assert!(out.contains("completed 1 task(s)"), "{out}");
    assert!(env
        .ok(&["list", "--status", "done"])
        .contains("Parent task"));
}

#[test]
fn duplicate_detection_warns_and_lists_similar_titles() {
    let env = TestEnv::new("duplicates");
    env.ok(&["init"]);
    env.ok(&["add", "Fix login bug"]);

    let out = env.run(&["add", "Fix the login issue"]);
    assert!(
        out.status.success(),
        "add failed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("similar task"), "{stderr}");
    assert!(stderr.contains("100% match"), "{stderr}");

    let duplicates = env.ok(&["duplicates"]);
    assert!(duplicates.contains("Fix login bug"), "{duplicates}");
    assert!(duplicates.contains("Fix the login issue"), "{duplicates}");
    assert!(duplicates.contains("1.00"), "{duplicates}");

    let err = env.fail(&["duplicates", "--threshold", "2"]);
    assert!(
        err.contains("--threshold must be between 0.0 and 1.0"),
        "{err}"
    );
}

#[test]
fn next_prefers_unblocked_priority_tasks() {
    let env = TestEnv::new("next");
    env.ok(&["init"]);

    env.ok(&["add", "Low overdue", "-P", "low", "--due", "2000-01-01"]);
    let blocker = env
        .ok(&["add", "Blocker task", "-P", "normal"])
        .trim()
        .to_string();
    let blocked = env
        .ok(&["add", "Blocked urgent", "-P", "urgent"])
        .trim()
        .to_string();

    env.ok(&["block", &blocked[..8], "--by", &blocker[..8]]);

    let next = env.ok(&["next"]);
    assert!(next.contains("Blocker task"), "{next}");
    assert!(!next.contains("Blocked urgent"), "{next}");

    let explained = env.ok(&["next", "--explain"]);
    assert!(explained.contains("priority=normal"), "{explained}");

    let blocked_next = env.ok(&["next", "--include-blocked"]);
    assert!(blocked_next.contains("Blocked urgent"), "{blocked_next}");

    env.ok(&["complete", &blocker[..8]]);
    let next = env.ok(&["next"]);
    assert!(next.contains("Blocked urgent"), "{next}");
}

#[test]
fn undo_reverts_last_mutation() {
    let env = TestEnv::new("undo");
    env.ok(&["init"]);
    let id = env.ok(&["add", "Original title"]).trim().to_string();
    let prefix = &id[..8];

    env.ok(&["edit", prefix, "--title", "Changed title"]);
    assert!(env.ok(&["list"]).contains("Changed title"));

    env.ok(&["undo"]);
    let list = env.ok(&["list"]);
    assert!(list.contains("Original title"), "{list}");
    assert!(!list.contains("Changed title"), "{list}");

    env.ok(&["delete", prefix]);
    assert!(!env.ok(&["list"]).contains("Original title"));
    env.ok(&["undo"]);
    assert!(env.ok(&["list"]).contains("Original title"));
}

#[test]
fn trash_restore_and_purge() {
    let env = TestEnv::new("trash");
    env.ok(&["init"]);
    let id = env.ok(&["add", "Trash me"]).trim().to_string();
    let prefix = &id[..8];

    env.ok(&["delete", prefix]);
    assert!(!env.ok(&["list"]).contains("Trash me"));
    assert!(env.ok(&["trash", "list"]).contains("Trash me"));

    env.ok(&["trash", "restore", prefix]);
    assert!(env.ok(&["list"]).contains("Trash me"));
    assert!(env.ok(&["trash", "list"]).contains("trash is empty"));

    env.ok(&["delete", prefix]);
    env.ok(&["trash", "purge"]);
    assert!(env.ok(&["trash", "list"]).contains("trash is empty"));
}

#[test]
fn subtasks_tree_and_completion_guard() {
    let env = TestEnv::new("subtasks");
    env.ok(&["init"]);
    let parent = env.ok(&["add", "Parent task"]).trim().to_string();
    let parent_prefix = &parent[..8];
    let child = env
        .ok(&["subtask", parent_prefix, "Child task"])
        .trim()
        .to_string();
    let child_prefix = &child[..8];

    let tree = env.ok(&["tree"]);
    assert!(tree.contains("Parent task"), "{tree}");
    assert!(tree.contains("Child task"), "{tree}");

    let err = env.fail(&["complete", parent_prefix]);
    assert!(err.contains("open subtask"), "{err}");

    env.ok(&["complete", child_prefix]);
    env.ok(&["complete", parent_prefix]);
    assert!(env.ok(&["stats"]).contains("done=2"));
}

#[test]
fn git_status_and_commit_link() {
    if Command::new("git").arg("--version").output().is_err() {
        return;
    }

    let env = TestEnv::new("git");
    env.ok(&["init"]);
    let repo = env.root.join("repo");
    std::fs::create_dir_all(&repo).unwrap();
    let git = |args: &[&str]| {
        let out = Command::new("git")
            .current_dir(&repo)
            .args(args)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "git {:?} failed:\n{}",
            args,
            String::from_utf8_lossy(&out.stderr)
        );
        out
    };
    git(&["init"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test User"]);
    std::fs::write(repo.join("file.txt"), "hello\n").unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "initial"]);

    let status = env.ok_in(&repo, &["git", "status"]);
    assert!(status.contains("repo:"), "{status}");
    assert!(status.contains("git branch:"), "{status}");
    assert!(status.contains("head:"), "{status}");

    let id = env
        .ok_in(&repo, &["add", "Linked to commit"])
        .trim()
        .to_string();
    let linked = env.ok_in(&repo, &["git", "link", &id[..8]]);
    assert!(linked.contains("linked"), "{linked}");

    let json = env.ok_in(&repo, &["export", "--format", "json"]);
    assert!(json.contains("\"git_branch\""), "{json}");
    assert!(json.contains("\"commit\": \""), "{json}");

    let outside = env.ok_in(&env.root, &["git", "status"]);
    assert!(outside.contains("not in a git repository"), "{outside}");
}

#[test]
fn git_add_maps_and_switches_branch() {
    if !git_available() {
        return;
    }

    let env = TestEnv::new("git-auto-branch");
    env.ok(&["init"]);
    let repo = env.root.join("auto-branch-repo");
    init_git_repo(&repo);
    git_ok(&repo, &["checkout", "-qb", "tx/auth"]);
    git_ok(
        &repo,
        &["remote", "add", "origin", "git@github.com:owner/repo.git"],
    );
    let config_dir = env.root.join("config").join("peekstore").join("tasklynx");
    std::fs::create_dir_all(&config_dir).unwrap();
    std::fs::write(
        config_dir.join("config.toml"),
        "[git]\nauto_branch = true\nbranch_prefix = \"tx/\"\nlink_commits = true\n",
    )
    .unwrap();

    let id = env
        .ok_in(&repo, &["add", "Feature branch task"])
        .trim()
        .to_string();
    assert_eq!(id.len(), 36);

    let branches = env.ok(&["branch", "list"]);
    assert!(branches.contains("* auth"), "{branches}");
    assert!(env.ok(&["list"]).contains("Feature branch task"));

    let json = env.ok(&["export", "--format", "json"]);
    assert!(json.contains("\"branch\": \"auth\""), "{json}");
    assert!(json.contains("\"git_branch\": \"tx/auth\""), "{json}");
    assert!(
        json.contains("\"git_remote\": \"git@github.com:owner/repo.git\""),
        "{json}"
    );
    assert!(json.contains("\"commit\": \""), "{json}");

    let status = env.ok_in(&repo, &["git", "status"]);
    assert!(status.contains("mapped tasklynx branch: auth"), "{status}");
    assert!(status.contains("recent commits:"), "{status}");
    assert!(status.contains("Feature branch task"), "{status}");
}

#[test]
fn complete_with_commit_links_task() {
    let env = TestEnv::new("complete-commit");
    env.ok(&["init"]);
    let id = env.ok(&["add", "Commit shortcut task"]).trim().to_string();

    let out = env.ok(&["complete", &id[..8], "--commit", "deadbeef"]);
    assert!(out.contains("done"), "{out}");
    let json = env.ok(&["export", "--format", "json"]);
    assert!(json.contains("\"commit\": \"deadbeef\""), "{json}");

    let err = env.fail(&[
        "complete",
        "--done-before",
        "2026-01-01",
        "--commit",
        "deadbeef",
    ]);
    assert!(err.contains("--commit requires a single task id"), "{err}");
}

fn git_available() -> bool {
    Command::new("git")
        .arg("--version")
        .output()
        .map(|out| out.status.success())
        .unwrap_or(false)
}

fn git_ok(dir: &std::path::Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .current_dir(dir)
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {:?} failed:\n{}",
        args,
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn init_git_repo(dir: &std::path::Path) {
    std::fs::create_dir_all(dir).unwrap();
    git_ok(dir, &["init", "-q"]);
    git_ok(dir, &["config", "user.email", "test@example.com"]);
    git_ok(dir, &["config", "user.name", "Test User"]);
    git_ok(dir, &["config", "commit.gpgsign", "false"]);
    std::fs::write(dir.join("README.md"), "initial\n").unwrap();
    git_ok(dir, &["add", "."]);
    git_ok(dir, &["commit", "-qm", "initial"]);
}

#[test]
fn git_help_lists_new_commands() {
    let env = TestEnv::new("git-help");
    let help = env.ok(&["git", "--help"]);
    for cmd in [
        "status",
        "link",
        "sync",
        "scan",
        "open",
        "hook",
        "activate",
        "deactivate",
    ] {
        assert!(help.contains(cmd), "git help is missing {cmd}: {help}");
    }
    let hook_help = env.ok(&["git", "hook", "--help"]);
    assert!(hook_help.contains("install"), "{hook_help}");
    assert!(hook_help.contains("uninstall"), "{hook_help}");
}

#[test]
fn git_scan_previews_applies_and_dedupes() {
    if !git_available() {
        return;
    }

    let env = TestEnv::new("git-scan");
    env.ok(&["init"]);
    let repo = env.root.join("scan-repo");
    init_git_repo(&repo);
    std::fs::write(
        repo.join("src.rs"),
        "fn main() {}\n// TODO: implement auth\n// FIXME: handle errors\n",
    )
    .unwrap();
    git_ok(&repo, &["add", "."]);
    git_ok(&repo, &["commit", "-qm", "add todos"]);

    let preview = env.ok_in(&repo, &["git", "scan"]);
    assert!(preview.contains("src.rs:2"), "{preview}");
    assert!(preview.contains("TODO: implement auth"), "{preview}");
    assert!(preview.contains("FIXME: handle errors"), "{preview}");
    assert!(preview.contains("found 2 item(s)"), "{preview}");
    assert!(!env.ok_in(&repo, &["list"]).contains("implement auth"));

    let applied = env.ok_in(&repo, &["git", "scan", "--apply", "--project", "backend"]);
    assert!(applied.contains("created 2 task(s)"), "{applied}");

    let list = env.ok_in(&repo, &["list", "--project", "backend"]);
    assert!(list.contains("TODO: implement auth"), "{list}");
    assert!(list.contains("FIXME: handle errors"), "{list}");

    let json = env.ok_in(&repo, &["export", "--format", "json"]);
    assert!(json.contains("\"source_path\": \"src.rs\""), "{json}");
    assert!(json.contains("\"source_line\": 2"), "{json}");
    assert!(json.contains("\"git_branch\": \""), "{json}");

    let again = env.ok_in(&repo, &["git", "scan", "--apply"]);
    assert!(
        again.contains("no new TODO/FIXME/HACK comments found"),
        "{again}"
    );
    assert!(env.ok_in(&repo, &["stats"]).contains("tasks=2"));
}

#[test]
fn git_hook_activate_sync_and_uninstall() {
    if !git_available() {
        return;
    }

    let env = TestEnv::new("git-hook");
    env.ok(&["init"]);
    let repo = env.root.join("hook-repo");
    init_git_repo(&repo);
    let id = env
        .ok_in(&repo, &["add", "Hook linked task"])
        .trim()
        .to_string();

    let activated = env.ok_in(&repo, &["git", "activate", &id[..8]]);
    assert!(activated.contains("active task"), "{activated}");

    let installed = env.ok_in(&repo, &["git", "hook", "install"]);
    assert!(installed.contains("installed hook"), "{installed}");
    let hook = repo.join(".git").join("hooks").join("prepare-commit-msg");
    assert!(hook.exists());
    let hook_body = std::fs::read_to_string(&hook).unwrap();
    assert!(hook_body.contains("managed by tasklynx"), "{hook_body}");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&hook).unwrap().permissions().mode();
        assert_ne!(mode & 0o111, 0, "hook should be executable");
    }

    std::fs::write(repo.join("work.txt"), "work\n").unwrap();
    git_ok(&repo, &["add", "."]);
    git_ok(&repo, &["commit", "-qm", "work on hook task"]);
    let body = git_ok(&repo, &["log", "-1", "--format=%B"]);
    assert!(body.contains("Tasklynx-Id:"), "{body}");
    assert!(body.contains(&id), "{body}");

    let synced = env.ok_in(&repo, &["git", "sync"]);
    assert!(synced.contains("linked 1 commit(s)"), "{synced}");
    let json = env.ok_in(&repo, &["export", "--format", "json"]);
    assert!(json.contains("\"commit\": \""), "{json}");
    assert!(json.contains("\"git_branch\": \""), "{json}");

    let deactivated = env.ok_in(&repo, &["git", "deactivate"]);
    assert!(deactivated.contains("deactivated"), "{deactivated}");
    let config = Command::new("git")
        .current_dir(&repo)
        .args(["config", "--get", "tasklynx.taskId"])
        .output()
        .unwrap();
    assert!(!config.status.success());

    let uninstalled = env.ok_in(&repo, &["git", "hook", "uninstall"]);
    assert!(uninstalled.contains("removed hook"), "{uninstalled}");
    assert!(!hook.exists());

    std::fs::write(&hook, "#!/bin/sh\nexit 0\n").unwrap();
    let err = env.fail_in(&repo, &["git", "hook", "install"]);
    assert!(err.contains("refusing to overwrite"), "{err}");
    let err = env.fail_in(&repo, &["git", "hook", "uninstall"]);
    assert!(err.contains("refusing to remove"), "{err}");
}

#[test]
fn git_open_prints_commit_and_pr_urls() {
    if !git_available() {
        return;
    }

    let env = TestEnv::new("git-open");
    env.ok(&["init"]);
    let repo = env.root.join("open-repo");
    init_git_repo(&repo);
    git_ok(
        &repo,
        &["remote", "add", "origin", "git@github.com:owner/repo.git"],
    );
    let id = env
        .ok_in(&repo, &["add", "Open linked task"])
        .trim()
        .to_string();

    let linked = env.ok_in(&repo, &["git", "link", &id[..8]]);
    assert!(linked.contains("linked"), "{linked}");
    let url = env
        .ok_in(&repo, &["git", "open", &id[..8], "--print"])
        .trim()
        .to_string();
    assert!(
        url.starts_with("https://github.com/owner/repo/commit/"),
        "{url}"
    );

    let pr = "https://github.com/owner/repo/pull/7";
    env.ok_in(&repo, &["git", "link", &id[..8], "--pr", pr]);
    let pr_url = env
        .ok_in(&repo, &["git", "open", &id[..8], "--print"])
        .trim()
        .to_string();
    assert_eq!(pr_url, pr);

    let err = env.fail_in(&repo, &["git", "link", &id[..8], "--pr", "ftp://bad"]);
    assert!(err.contains("--pr must be an http(s) URL"), "{err}");

    let issue_id = env
        .ok_in(&repo, &["add", "Issue linked task"])
        .trim()
        .to_string();
    let issue = "https://github.com/owner/repo/issues/9";
    env.ok_in(&repo, &["git", "link", &issue_id[..8], "--issue", issue]);
    let issue_url = env
        .ok_in(&repo, &["git", "open", &issue_id[..8], "--print"])
        .trim()
        .to_string();
    assert_eq!(issue_url, issue);
    let json = env.ok_in(&repo, &["export", "--format", "json"]);
    assert!(
        json.contains("\"git_issue_url\": \"https://github.com/owner/repo/issues/9\""),
        "{json}"
    );
    let err = env.fail_in(
        &repo,
        &["git", "link", &issue_id[..8], "--issue", "ftp://bad"],
    );
    assert!(err.contains("--issue must be an http(s) URL"), "{err}");

    let unlinked = env.ok(&["add", "Nothing linked"]).trim().to_string();
    let err = env.fail_in(&repo, &["git", "open", &unlinked[..8], "--print"]);
    assert!(err.contains("task has no commit or PR link"), "{err}");
}

#[test]
fn git_scan_from_subdirectory_uses_repo_relative_paths() {
    if !git_available() {
        return;
    }

    let env = TestEnv::new("git-scan-subdir");
    env.ok(&["init"]);
    let repo = env.root.join("scan-subdir-repo");
    init_git_repo(&repo);
    let nested = repo.join("nested");
    std::fs::create_dir_all(&nested).unwrap();
    std::fs::write(nested.join("nested.rs"), "// TODO: nested task\n").unwrap();
    git_ok(&repo, &["add", "."]);
    git_ok(&repo, &["commit", "-qm", "add nested todo"]);

    let applied = env.ok_in(&nested, &["git", "scan", "--apply"]);
    assert!(applied.contains("nested/nested.rs:1"), "{applied}");
    let json = env.ok_in(&nested, &["export", "--format", "json"]);
    assert!(
        json.contains("\"source_path\": \"nested/nested.rs\""),
        "{json}"
    );
}

#[test]
fn ai_commands_available_offline() {
    let env = TestEnv::new("ai-offline");
    env.ok(&["init"]);

    let help = env.ok(&["ai", "--help"]);
    for command in [
        "plan",
        "breakdown",
        "triage",
        "standup",
        "review",
        "ask",
        "doctor",
    ] {
        assert!(help.contains(command), "ai help is missing {command}");
    }
    let plan_help = env.ok(&["ai", "plan", "--help"]);
    for flag in ["--apply", "--project", "--limit", "--no-description"] {
        assert!(plan_help.contains(flag), "ai plan help is missing {flag}");
    }
    let breakdown_help = env.ok(&["ai", "breakdown", "--help"]);
    for flag in ["--apply", "--limit", "--no-description"] {
        assert!(
            breakdown_help.contains(flag),
            "ai breakdown help is missing {flag}"
        );
    }
    let triage_help = env.ok(&["ai", "triage", "--help"]);
    for flag in ["--limit", "--project", "--include-blocked", "--offline"] {
        assert!(
            triage_help.contains(flag),
            "ai triage help is missing {flag}"
        );
    }
    let standup_help = env.ok(&["ai", "standup", "--help"]);
    for flag in ["--days", "--project", "--limit", "--no-description"] {
        assert!(
            standup_help.contains(flag),
            "ai standup help is missing {flag}"
        );
    }
    let review_help = env.ok(&["ai", "review", "--help"]);
    for flag in ["--days", "--project", "--limit", "--no-description"] {
        assert!(
            review_help.contains(flag),
            "ai review help is missing {flag}"
        );
    }
    let ask_help = env.ok(&["ai", "ask", "--help"]);
    for flag in ["--project", "--branch", "--limit", "--no-description"] {
        assert!(ask_help.contains(flag), "ai ask help is missing {flag}");
    }

    let config = env.ok(&["config", "show"]);
    assert!(config.contains("[ai]"), "{config}");
    assert!(config.contains("provider = \"none\""), "{config}");

    let doctor = env.ok(&["ai", "doctor"]);
    assert!(doctor.contains("provider: none"), "{doctor}");
    assert!(doctor.contains("offline"), "{doctor}");

    let id = env.ok(&["add", "Parent task"]).trim().to_string();
    let prefix = &id[..8];
    let error = env.fail(&["ai", "plan", "Ship the feature"]);
    assert!(error.contains("AI provider is not configured"), "{error}");
    let error = env.fail(&["ai", "breakdown", prefix]);
    assert!(error.contains("AI provider is not configured"), "{error}");
    let error = env.fail(&["ai", "standup"]);
    assert!(error.contains("AI provider is not configured"), "{error}");
    let error = env.fail(&["ai", "review"]);
    assert!(error.contains("AI provider is not configured"), "{error}");
    let error = env.fail(&["ai", "ask", "What should I do next?"]);
    assert!(error.contains("AI provider is not configured"), "{error}");
}

#[test]
fn offline_triage_ranks_and_filters_tasks() {
    let env = TestEnv::new("ai-triage-offline");
    env.ok(&["init"]);
    let normal = env.ok(&["add", "Normal task"]).trim().to_string();
    let urgent = env
        .ok(&["add", "Urgent task", "--priority", "urgent"])
        .trim()
        .to_string();
    let doing = env
        .ok(&["add", "Doing task", "--priority", "high"])
        .trim()
        .to_string();
    env.ok(&["doing", &doing[..8]]);
    let blocker = env.ok(&["add", "Blocker"]).trim().to_string();
    let blocked = env
        .ok(&["add", "Blocked task", "--priority", "urgent"])
        .trim()
        .to_string();
    env.ok(&["block", &blocked[..8], "--by", &blocker[..8]]);

    let out = env.ok(&["ai", "triage"]);
    assert!(out.contains("offline triage"), "{out}");
    assert!(out.contains("Doing task"), "{out}");
    assert!(out.contains("Urgent task"), "{out}");
    assert!(out.contains(&urgent[..8]), "{out}");
    assert!(out.contains("Normal task"), "{out}");
    assert!(out.contains("Blocker"), "{out}");
    assert!(!out.contains("Blocked task"), "{out}");
    assert!(out.contains(&normal[..8]), "{out}");

    let out = env.ok(&["ai", "triage", "--limit", "1"]);
    assert!(out.contains("Urgent task"), "{out}");
    assert!(!out.contains("Doing task"), "{out}");

    let out = env.ok(&["ai", "triage", "--include-blocked"]);
    assert!(out.contains("Blocked task"), "{out}");
    assert!(out.contains("blocked by 1"), "{out}");

    let out = env.ok(&["ai", "triage", "--offline", "--project", "default"]);
    assert!(out.contains("offline triage"), "{out}");
    assert!(out.contains("Urgent task"), "{out}");
    assert!(!out.contains("Blocked task"), "{out}");

    write_ai_config(&env, "http://127.0.0.1:9");
    let out = env.ok(&["ai", "triage", "--offline"]);
    assert!(out.contains("offline triage"), "{out}");
    assert!(out.contains("Urgent task"), "{out}");
}

fn write_ai_config(env: &TestEnv, endpoint: &str) {
    let dir = env.root.join("config").join("peekstore").join("tasklynx");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("config.toml"),
        format!(
            "[ai]\nprovider = \"ollama\"\nendpoint = \"{endpoint}\"\nmodel = \"fixture\"\nmax_context_tasks = 5\n"
        ),
    )
    .unwrap();
}

#[cfg(feature = "ai")]
fn mock_ollama(response: &str) -> (String, std::thread::JoinHandle<()>) {
    use std::io::{Read, Write};

    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let payload = serde_json::json!({ "response": response }).to_string();
    let handle = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(2)))
            .unwrap();
        let mut request = Vec::new();
        let mut buffer = [0u8; 4096];
        loop {
            let read = stream.read(&mut buffer).unwrap_or(0);
            if read == 0 {
                break;
            }
            request.extend_from_slice(&buffer[..read]);
            if let Some(end) = request.windows(4).position(|window| window == b"\r\n\r\n") {
                let headers = String::from_utf8_lossy(&request[..end]);
                let body_length = headers
                    .lines()
                    .find_map(|line| {
                        let (key, value) = line.split_once(':')?;
                        if key.eq_ignore_ascii_case("content-length") {
                            value.trim().parse::<usize>().ok()
                        } else {
                            None
                        }
                    })
                    .unwrap_or(0);
                if request.len() >= end + 4 + body_length {
                    break;
                }
            }
        }
        let body = payload.as_bytes();
        write!(
            stream,
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        )
        .unwrap();
        stream.write_all(body).unwrap();
    });
    (format!("http://{address}"), handle)
}

#[cfg(not(feature = "ai"))]
#[test]
fn ai_network_requires_ai_feature() {
    let env = TestEnv::new("ai-no-feature");
    env.ok(&["init"]);
    write_ai_config(&env, "http://127.0.0.1:9");
    let error = env.fail(&["ai", "doctor"]);
    assert!(error.contains("AI networking is disabled"), "{error}");
}

#[cfg(feature = "ai")]
#[test]
fn ai_plan_and_breakdown_preview_and_apply() {
    let env = TestEnv::new("ai-mock");
    env.ok(&["init"]);

    let plan_response = serde_json::json!({
        "project": "ai",
        "tasks": [
            {"title": "Design schema", "priority": "high", "tags": ["db"]},
            {"title": "Build endpoint", "depends_on": [0]}
        ]
    })
    .to_string();

    let (endpoint, handle) = mock_ollama(&plan_response);
    write_ai_config(&env, &endpoint);
    let preview = env.ok(&["ai", "plan", "Ship the API", "--limit", "5"]);
    assert!(preview.contains("preview: 2 task(s)"), "{preview}");
    assert!(preview.contains("Design schema"), "{preview}");
    assert!(!env.ok(&["list"]).contains("Design schema"));
    handle.join().unwrap();

    let (endpoint, handle) = mock_ollama(&plan_response);
    write_ai_config(&env, &endpoint);
    let applied = env.ok(&["ai", "plan", "Ship the API", "--apply", "--limit", "5"]);
    assert!(applied.contains("created 2 task(s)"), "{applied}");
    handle.join().unwrap();

    let list = env.ok(&["list", "--project", "ai"]);
    assert!(list.contains("Design schema"), "{list}");
    assert!(list.contains("Build endpoint"), "{list}");
    let json = env.ok(&["export", "--format", "json"]);
    let data: serde_json::Value = serde_json::from_str(&json).unwrap();
    let endpoint_task = data["tasks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|task| task["title"] == "Build endpoint")
        .unwrap();
    assert_eq!(endpoint_task["blocked_by"].as_array().unwrap().len(), 1);
    let parent = data["tasks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|task| task["title"] == "Design schema")
        .unwrap();
    let parent_id = parent["id"].as_str().unwrap().to_string();

    let breakdown_response = serde_json::json!({
        "tasks": [
            {"title": "Write migration"},
            {"title": "Add tests", "depends_on": [0]}
        ]
    })
    .to_string();
    let (endpoint, handle) = mock_ollama(&breakdown_response);
    write_ai_config(&env, &endpoint);
    let preview = env.ok(&["ai", "breakdown", &parent_id]);
    assert!(preview.contains("preview: 2 task(s)"), "{preview}");
    assert!(!env
        .ok(&["tree", &parent_id[..8]])
        .contains("Write migration"));
    handle.join().unwrap();

    let (endpoint, handle) = mock_ollama(&breakdown_response);
    write_ai_config(&env, &endpoint);
    let applied = env.ok(&["ai", "breakdown", &parent_id, "--apply"]);
    assert!(applied.contains("created 2 subtask(s)"), "{applied}");
    handle.join().unwrap();

    let tree = env.ok(&["tree", &parent_id[..8]]);
    assert!(tree.contains("Write migration"), "{tree}");
    assert!(tree.contains("Add tests"), "{tree}");

    env.ok(&["undo"]);
    let tree = env.ok(&["tree", &parent_id[..8]]);
    assert!(!tree.contains("Write migration"), "{tree}");
    assert!(!tree.contains("Add tests"), "{tree}");
}

#[cfg(feature = "ai")]
#[test]
fn ai_triage_reports_and_ask_use_provider() {
    let env = TestEnv::new("ai-context");
    env.ok(&["init"]);
    let id = env
        .ok(&["add", "Fix login bug", "--priority", "urgent"])
        .trim()
        .to_string();

    let triage_response = serde_json::json!({
        "summary": "Fix the login bug first.",
        "picks": [{"id": &id[..8], "reason": "urgent and blocking users"}]
    })
    .to_string();
    let (endpoint, handle) = mock_ollama(&triage_response);
    write_ai_config(&env, &endpoint);
    let out = env.ok(&["ai", "triage"]);
    assert!(out.contains("Fix the login bug first."), "{out}");
    assert!(out.contains("Fix login bug"), "{out}");
    assert!(out.contains("urgent and blocking users"), "{out}");
    handle.join().unwrap();

    let standup_response = serde_json::json!({
        "summary": "One urgent task is open.",
        "sections": [{"title": "Doing", "items": ["Fix login bug"]}]
    })
    .to_string();
    let (endpoint, handle) = mock_ollama(&standup_response);
    write_ai_config(&env, &endpoint);
    let out = env.ok(&["ai", "standup", "--days", "1"]);
    assert!(out.contains("One urgent task is open."), "{out}");
    assert!(out.contains("Doing"), "{out}");
    assert!(out.contains("- Fix login bug"), "{out}");
    handle.join().unwrap();

    let review_response = serde_json::json!({
        "summary": "The branch has one open task.",
        "sections": [{"title": "Open", "items": ["Fix login bug"]}]
    })
    .to_string();
    let (endpoint, handle) = mock_ollama(&review_response);
    write_ai_config(&env, &endpoint);
    let out = env.ok(&["ai", "review", "main", "--days", "7"]);
    assert!(out.contains("The branch has one open task."), "{out}");
    assert!(out.contains("- Fix login bug"), "{out}");
    handle.join().unwrap();

    let ask_response = serde_json::json!({"answer": "Start with Fix login bug."}).to_string();
    let (endpoint, handle) = mock_ollama(&ask_response);
    write_ai_config(&env, &endpoint);
    let out = env.ok(&["ai", "ask", "What should I do next?"]);
    assert!(out.contains("Start with Fix login bug."), "{out}");
    handle.join().unwrap();

    let bad_response = serde_json::json!({
        "summary": "bad",
        "picks": [{"id": "00000000", "reason": "missing"}]
    })
    .to_string();
    let (endpoint, handle) = mock_ollama(&bad_response);
    write_ai_config(&env, &endpoint);
    let error = env.fail(&["ai", "triage"]);
    assert!(error.contains("unknown task id"), "{error}");
    handle.join().unwrap();
}
