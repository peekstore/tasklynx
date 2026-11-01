mod ai;
mod config;
mod duplicates;
mod git;
mod model;
mod reminders;
mod store;
mod templates;
mod tui;
mod web;

use anyhow::{bail, Result};
use clap::{CommandFactory, Parser, Subcommand};
use model::{local_due_date, Priority, Repeat, Status, Task};
use std::path::PathBuf;
use std::process::Command;

#[derive(Parser)]
#[command(
    name = "tasklynx",
    alias = "tx",
    version,
    about = "Developer-first local task manager"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Cmd>,
}

#[derive(Subcommand)]
enum Cmd {
    Init,
    Add {
        title: Option<String>,
        #[arg(short, long, default_value = "default")]
        project: String,
        #[arg(short, long, value_delimiter = ',')]
        tag: Vec<String>,
        #[arg(short = 'P', long)]
        priority: Option<String>,
        #[arg(long)]
        due: Option<String>,
        #[arg(short, long)]
        description: Option<String>,
        #[arg(long)]
        estimate: Option<u32>,
        #[arg(long)]
        commit: Option<String>,
        #[arg(long)]
        repeat: Option<String>,
        #[arg(long)]
        repeat_until: Option<String>,
        #[arg(long)]
        template: Option<String>,
    },
    List {
        #[arg(long)]
        project: Option<String>,
        #[arg(long)]
        tag: Option<String>,
        #[arg(long)]
        status: Option<String>,
        #[arg(long)]
        search: Option<String>,
        #[arg(long)]
        all: bool,
        #[arg(long)]
        view: Option<String>,
        #[arg(long)]
        save: Option<String>,
    },
    Complete {
        id: Option<String>,
        #[arg(long)]
        force: bool,
        #[arg(long)]
        done_before: Option<String>,
        #[arg(long)]
        all: bool,
        #[arg(long)]
        commit: Option<String>,
    },
    Reopen {
        id: String,
    },
    Doing {
        id: String,
    },
    Start {
        id: String,
    },
    Stop,
    Time {
        id: Option<String>,
    },
    #[command(aliases = ["views", "filter", "filters"])]
    View {
        #[command(subcommand)]
        action: ViewCmd,
    },
    #[command(alias = "dups")]
    Duplicates {
        #[arg(long, default_value_t = 0.72)]
        threshold: f64,
        #[arg(long)]
        all: bool,
    },
    Delete {
        id: String,
    },
    Undo,
    Next {
        #[arg(long)]
        project: Option<String>,
        #[arg(long)]
        all: bool,
        #[arg(long)]
        include_blocked: bool,
        #[arg(long)]
        explain: bool,
    },
    Block {
        id: String,
        #[arg(long)]
        by: String,
    },
    Unblock {
        id: String,
        #[arg(long)]
        by: Option<String>,
    },
    Subtask {
        parent: String,
        title: String,
        #[arg(short = 'P', long)]
        priority: Option<String>,
        #[arg(long)]
        due: Option<String>,
    },
    Tree {
        id: Option<String>,
        #[arg(long)]
        all: bool,
    },
    Trash {
        #[command(subcommand)]
        action: TrashCmd,
    },
    Git {
        #[command(subcommand)]
        action: GitCmd,
    },
    Ai {
        #[command(subcommand)]
        action: AiCmd,
    },
    Edit {
        id: String,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        priority: Option<String>,
        #[arg(long)]
        due: Option<String>,
        #[arg(long)]
        project: Option<String>,
        #[arg(long)]
        estimate: Option<u32>,
        #[arg(long)]
        commit: Option<String>,
        #[arg(long)]
        add_tag: Vec<String>,
        #[arg(long)]
        remove_tag: Vec<String>,
        #[arg(long)]
        repeat: Option<String>,
        #[arg(long)]
        repeat_until: Option<String>,
    },
    Duplicate {
        id: String,
    },
    Search {
        query: String,
    },
    Today,
    Agenda {
        #[arg(long, default_value_t = 7)]
        days: u32,
    },
    Remind {
        #[arg(long, default_value_t = 3)]
        days: u32,
        #[arg(long)]
        notify: bool,
    },
    Stats,
    Branch {
        #[command(subcommand)]
        action: BranchCmd,
    },
    Export {
        #[arg(long, default_value = "json")]
        format: String,
    },
    Import {
        path: Option<PathBuf>,
        #[arg(long, default_value = "json")]
        format: String,
        #[arg(long)]
        replace: bool,
    },
    Web {
        #[arg(long)]
        host: Option<String>,
        #[arg(long)]
        port: Option<u16>,
    },
    Board,
    Config {
        #[command(subcommand)]
        action: ConfigCmd,
    },
    Completions {
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
    Man,
    #[command(name = "self")]
    #[allow(clippy::enum_variant_names)]
    SelfCmd {
        #[command(subcommand)]
        action: SelfSubCmd,
    },
}

#[derive(Subcommand)]
enum TrashCmd {
    List,
    Restore { id: String },
    Purge,
}

#[derive(Subcommand)]
enum GitCmd {
    Status,
    Link {
        id: String,
        #[arg(long)]
        commit: Option<String>,
        #[arg(long)]
        pr: Option<String>,
        #[arg(long)]
        issue: Option<String>,
    },
    Sync {
        #[arg(long, default_value_t = 100)]
        limit: usize,
    },
    Scan {
        #[arg(long)]
        apply: bool,
        #[arg(long, default_value_t = 100)]
        limit: usize,
        #[arg(short, long, default_value = "default")]
        project: String,
    },
    Open {
        id: String,
        #[arg(long)]
        print: bool,
    },
    Hook {
        #[command(subcommand)]
        action: HookCmd,
    },
    Activate {
        id: String,
    },
    Deactivate,
}

#[derive(Subcommand)]
enum HookCmd {
    Install,
    Uninstall,
}

#[derive(Subcommand)]
enum AiCmd {
    Plan {
        goal: String,
        #[arg(long)]
        apply: bool,
        #[arg(long)]
        project: Option<String>,
        #[arg(long, default_value_t = 10)]
        limit: usize,
        #[arg(long)]
        no_description: bool,
    },
    Breakdown {
        id: String,
        #[arg(long)]
        apply: bool,
        #[arg(long, default_value_t = 10)]
        limit: usize,
        #[arg(long)]
        no_description: bool,
    },
    Triage {
        #[arg(long, default_value_t = 5)]
        limit: usize,
        #[arg(long)]
        project: Option<String>,
        #[arg(long)]
        include_blocked: bool,
        #[arg(long)]
        no_description: bool,
        #[arg(long)]
        offline: bool,
    },
    Standup {
        #[arg(long, default_value_t = 1)]
        days: u32,
        #[arg(long)]
        project: Option<String>,
        #[arg(long, default_value_t = 10)]
        limit: usize,
        #[arg(long)]
        no_description: bool,
    },
    Review {
        branch: Option<String>,
        #[arg(long, default_value_t = 30)]
        days: u32,
        #[arg(long)]
        project: Option<String>,
        #[arg(long, default_value_t = 10)]
        limit: usize,
        #[arg(long)]
        no_description: bool,
    },
    Ask {
        question: String,
        #[arg(long)]
        project: Option<String>,
        #[arg(long)]
        branch: Option<String>,
        #[arg(long, default_value_t = 50)]
        limit: usize,
        #[arg(long)]
        no_description: bool,
    },
    Doctor,
}

#[derive(Subcommand)]
enum BranchCmd {
    List,
    Create { name: String },
    Switch { name: String },
    Delete { name: String },
}

#[derive(Subcommand)]
enum ViewCmd {
    Save {
        name: String,
        #[arg(long)]
        project: Option<String>,
        #[arg(long)]
        tag: Option<String>,
        #[arg(long)]
        status: Option<String>,
        #[arg(long)]
        search: Option<String>,
        #[arg(long)]
        all: bool,
    },
    List,
    Show {
        name: String,
    },
    Run {
        name: String,
    },
    Delete {
        name: String,
    },
}

#[derive(Subcommand)]
enum ConfigCmd {
    Init,
    Show,
}

#[derive(Subcommand)]
enum SelfSubCmd {
    Update,
    Uninstall {
        #[arg(long)]
        purge: bool,
    },
}

fn priority(s: &str) -> Priority {
    match s.to_lowercase().as_str() {
        "low" => Priority::Low,
        "high" => Priority::High,
        "urgent" => Priority::Urgent,
        _ => Priority::Normal,
    }
}

fn parse_repeat(s: &str) -> Result<Repeat> {
    match s.to_lowercase().as_str() {
        "none" | "off" => Ok(Repeat::None),
        "daily" | "day" => Ok(Repeat::Daily),
        "weekly" | "week" => Ok(Repeat::Weekly),
        "monthly" | "month" => Ok(Repeat::Monthly),
        "yearly" | "year" | "annual" => Ok(Repeat::Yearly),
        _ => anyhow::bail!(
            "invalid repeat value '{s}': expected none, daily, weekly, monthly, or yearly"
        ),
    }
}

fn format_duration(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;
    if hours > 0 {
        if secs > 0 {
            format!("{hours}h {minutes:02}m {secs:02}s")
        } else if minutes > 0 {
            format!("{hours}h {minutes:02}m")
        } else {
            format!("{hours}h")
        }
    } else if minutes > 0 {
        if secs > 0 {
            format!("{minutes}m {secs:02}s")
        } else {
            format!("{minutes}m")
        }
    } else {
        format!("{secs}s")
    }
}

fn validate_view_name(name: &str) -> Result<()> {
    if name.trim().is_empty() {
        bail!("view name must not be empty");
    }
    Ok(())
}

fn saved_view(
    project: Option<String>,
    tag: Option<String>,
    status: Option<String>,
    search: Option<String>,
    all: bool,
) -> model::SavedView {
    model::SavedView {
        project,
        tag,
        status,
        search,
        all,
    }
}

fn load_saved_view(name: &str) -> Result<model::SavedView> {
    let config = config::load()?;
    config.views.get(name).cloned().ok_or_else(|| {
        let names = if config.views.is_empty() {
            "none".to_string()
        } else {
            config.views.keys().cloned().collect::<Vec<_>>().join(", ")
        };
        anyhow::anyhow!("view not found: {name}; saved views: {names}")
    })
}

fn save_view(name: &str, view: &model::SavedView) -> Result<()> {
    validate_view_name(name)?;
    let mut config = config::load()?;
    config.views.insert(name.to_string(), view.clone());
    config::save(&config)
}

fn view_line(name: &str, view: &model::SavedView) -> String {
    format!(
        "{name}  project={} tag={} status={} search={} all={}",
        view.project.as_deref().unwrap_or("-"),
        view.tag.as_deref().unwrap_or("-"),
        view.status.as_deref().unwrap_or("-"),
        view.search.as_deref().unwrap_or("-"),
        view.all
    )
}

fn task_matches_view(task: &Task, view: &model::SavedView, current_branch: &str) -> bool {
    if !view.all && task.branch != current_branch {
        return false;
    }
    if view
        .project
        .as_ref()
        .is_some_and(|project| &task.project != project)
    {
        return false;
    }
    if view
        .tag
        .as_ref()
        .is_some_and(|tag| !task.tags.iter().any(|item| item == tag))
    {
        return false;
    }
    if view
        .status
        .as_ref()
        .is_some_and(|status| task.status.to_string() != *status)
    {
        return false;
    }
    if view.search.as_ref().is_some_and(|search| {
        !task.title.to_lowercase().contains(&search.to_lowercase())
            && !task
                .description
                .to_lowercase()
                .contains(&search.to_lowercase())
    }) {
        return false;
    }
    true
}

fn print_task_list(data: &model::ProjectData, view: &model::SavedView) {
    for task in &data.tasks {
        if !task_matches_view(task, view, &data.current_branch) {
            continue;
        }
        let recur = if task.repeat == Repeat::None {
            String::new()
        } else {
            format!(" ↻ {}", task.repeat)
        };
        println!(
            "{}  {:7} {:7} {:12} {}{}",
            &task.id.to_string()[..8],
            task.status,
            task.priority,
            task.project,
            task.title,
            recur
        );
    }
}

fn due_display(t: &Task) -> String {
    t.due
        .map(|d| {
            d.with_timezone(&chrono::Local)
                .format("%Y-%m-%d")
                .to_string()
        })
        .unwrap_or_else(|| "-".to_string())
}

fn print_tree(
    data: &model::ProjectData,
    task: &Task,
    depth: usize,
    seen: &mut std::collections::HashSet<uuid::Uuid>,
) {
    if !seen.insert(task.id) {
        return;
    }
    let recur = if task.repeat == Repeat::None {
        String::new()
    } else {
        format!(" ↻ {}", task.repeat)
    };
    println!(
        "{}{}  {:7} {:7} {}{}",
        "  ".repeat(depth),
        &task.id.to_string()[..8],
        task.status,
        task.priority,
        task.title,
        recur
    );
    let mut children: Vec<&Task> = data
        .tasks
        .iter()
        .filter(|t| t.parent == Some(task.id))
        .collect();
    children.sort_by_key(|t| t.created_at);
    for child in children {
        print_tree(data, child, depth + 1, seen);
    }
}

fn print_triage_picks(data: &model::ProjectData, picks: &[ai::TriagePick]) {
    for (index, pick) in picks.iter().enumerate() {
        if let Ok(task) = store::find(data, &pick.task_id.to_string()) {
            println!(
                "  {}. {}  [{}] {}  {}",
                index + 1,
                &task.id.to_string()[..8],
                task.priority,
                task.project,
                task.title
            );
            println!("     {}", pick.reason);
        }
    }
}

fn print_report(report: &ai::Report) {
    println!("{}", report.summary);
    for section in &report.sections {
        println!("\n{}", section.title);
        for item in &section.items {
            println!("- {item}");
        }
    }
}

fn announce_ai(provider: &dyn ai::Provider, config: &model::AppConfig) {
    eprintln!("ai: provider={} model={}", provider.name(), config.ai.model);
}

fn print_ai_tasks(label: &str, tasks: &[Task]) {
    println!("{label}: {} task(s)", tasks.len());
    for (index, task) in tasks.iter().enumerate() {
        print!(
            "  {}. [{}] {}  {}",
            index + 1,
            task.priority,
            task.project,
            task.title
        );
        let dependencies: Vec<String> = task
            .blocked_by
            .iter()
            .filter_map(|id| {
                tasks
                    .iter()
                    .position(|candidate| candidate.id == *id)
                    .map(|position| (position + 1).to_string())
            })
            .collect();
        if !dependencies.is_empty() {
            print!(" depends_on={}", dependencies.join(","));
        }
        println!();
    }
}

fn marker_in_line(line: &str) -> Option<(&'static str, usize, String)> {
    let mut best: Option<(&'static str, usize, String)> = None;
    for marker in ["TODO", "FIXME", "HACK"] {
        let mut from = 0;
        while let Some(relative) = line[from..].find(marker) {
            let idx = from + relative;
            let before = line[..idx].chars().next_back();
            let after = line[idx + marker.len()..].chars().next();
            let boundary =
                |c: Option<char>| c.map_or(true, |ch| !ch.is_alphanumeric() && ch != '_');
            if boundary(before) && boundary(after) {
                let rest = line[idx + marker.len()..].trim_start();
                let rest = rest.strip_prefix(':').unwrap_or(rest).trim().to_string();
                if best
                    .as_ref()
                    .map_or(true, |(_, best_idx, _)| idx < *best_idx)
                {
                    best = Some((marker, idx, rest));
                }
                break;
            }
            from = idx + marker.len();
        }
    }
    best
}

fn scan_repo(
    data: &model::ProjectData,
    info: &git::GitInfo,
    project: &str,
    limit: usize,
) -> Vec<Task> {
    let Some(files) = git::tracked_files() else {
        return Vec::new();
    };
    let mut tasks = Vec::new();
    for relative in files {
        if tasks.len() >= limit {
            break;
        }
        let path = info.root.join(&relative);
        let Ok(metadata) = std::fs::metadata(&path) else {
            continue;
        };
        if !metadata.is_file() || metadata.len() > 2 * 1024 * 1024 {
            continue;
        }
        let Ok(bytes) = std::fs::read(&path) else {
            continue;
        };
        if bytes.contains(&0) {
            continue;
        }
        let text = String::from_utf8_lossy(&bytes);
        let display_path = relative.to_string_lossy().replace('\\', "/");
        for (index, line) in text.lines().enumerate() {
            let Some((marker, _, comment)) = marker_in_line(line) else {
                continue;
            };
            let line_number = u32::try_from(index + 1).unwrap_or(u32::MAX);
            let already_found = data.tasks.iter().any(|task| {
                task.source_path.as_deref() == Some(display_path.as_str())
                    && task.source_line == Some(line_number)
            });
            if already_found {
                continue;
            }
            let title = if comment.is_empty() {
                format!("{marker} at {display_path}:{line_number}")
            } else {
                format!("{marker}: {comment}")
            };
            let mut task = Task::new(
                title,
                project.to_string(),
                data.current_branch.clone(),
                Priority::Normal,
                Vec::new(),
                None,
            );
            task.description = format!("{marker} in {display_path}:{line_number}\n{}", line.trim());
            task.commit = info.commit.clone();
            task.git_branch = info.branch.clone();
            task.git_remote = info.remote.clone();
            task.source_path = Some(display_path.clone());
            task.source_line = Some(line_number);
            tasks.push(task);
            if tasks.len() >= limit {
                break;
            }
        }
    }
    tasks
}

const HOOK_MARKER: &str = "# managed by tasklynx";
const HOOK_SCRIPT: &str = r#"#!/bin/sh
# managed by tasklynx
task_id=$(git config --get tasklynx.taskId 2>/dev/null || true)
[ -n "$task_id" ] || exit 0
if grep -q '^Tasklynx-Id:' "$1"; then
    exit 0
fi
printf '\nTasklynx-Id: %s\n' "$task_id" >> "$1"
"#;

fn install_git_hook(info: &git::GitInfo) -> Result<PathBuf> {
    let hooks = info.hooks_dir();
    std::fs::create_dir_all(&hooks)?;
    let path = hooks.join("prepare-commit-msg");
    if path.exists() {
        let existing = std::fs::read_to_string(&path).unwrap_or_default();
        if !existing.contains(HOOK_MARKER) {
            anyhow::bail!(
                "refusing to overwrite {}: hook was not installed by tasklynx",
                path.display()
            );
        }
    }
    std::fs::write(&path, HOOK_SCRIPT)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755))?;
    }
    Ok(path)
}

fn uninstall_git_hook(info: &git::GitInfo) -> Result<PathBuf> {
    let path = info.hooks_dir().join("prepare-commit-msg");
    if !path.exists() {
        anyhow::bail!("no prepare-commit-msg hook installed");
    }
    let existing = std::fs::read_to_string(&path).unwrap_or_default();
    if !existing.contains(HOOK_MARKER) {
        anyhow::bail!(
            "refusing to remove {}: hook was not installed by tasklynx",
            path.display()
        );
    }
    std::fs::remove_file(&path)?;
    Ok(path)
}

fn commit_task_id(body: &str) -> Option<String> {
    body.lines().find_map(|line| {
        let (key, value) = line.split_once(':')?;
        if !key.trim().eq_ignore_ascii_case("Tasklynx-Id") {
            return None;
        }
        let value = value.trim();
        if value.is_empty() {
            None
        } else {
            Some(value.to_string())
        }
    })
}

#[cfg(target_os = "macos")]
fn open_url(url: &str) -> Result<()> {
    let status = Command::new("open").arg(url).status()?;
    if !status.success() {
        anyhow::bail!("failed to open {url}");
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn open_url(url: &str) -> Result<()> {
    let status = Command::new("cmd")
        .args(["/C", "start", ""])
        .arg(url)
        .status()?;
    if !status.success() {
        anyhow::bail!("failed to open {url}");
    }
    Ok(())
}

#[cfg(all(unix, not(target_os = "macos")))]
fn open_url(url: &str) -> Result<()> {
    let status = Command::new("xdg-open").arg(url).status()?;
    if !status.success() {
        anyhow::bail!("failed to open {url}");
    }
    Ok(())
}

#[cfg(not(any(unix, windows)))]
fn open_url(_url: &str) -> Result<()> {
    anyhow::bail!("opening URLs is not supported on this platform");
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let cmd = cli.command.unwrap_or(Cmd::Board);
    let mut data = store::load()?;

    match cmd {
        Cmd::Init => {
            let _lock = store::StoreLock::acquire()?;
            store::save(&data)?;
            let _ = config::save(&config::load()?);
            println!("initialized {}", store::path().display());
        }
        Cmd::Add {
            title,
            project,
            tag,
            priority: pri,
            due,
            description,
            estimate,
            commit,
            repeat,
            repeat_until,
            template,
        } => {
            let _lock = store::StoreLock::acquire()?;
            let template = match template {
                Some(name) => Some(templates::find(&name).ok_or_else(|| {
                    anyhow::anyhow!(
                        "unknown template '{name}'; available templates: {}",
                        templates::names()
                    )
                })?),
                None => None,
            };
            let title = title
                .or_else(|| template.map(|template| template.title.to_string()))
                .ok_or_else(|| {
                    anyhow::anyhow!("a task title is required unless --template is used")
                })?;
            let due = due.map(|d| local_due_date(&d)).transpose()?;
            let repeat = match repeat {
                Some(v) => parse_repeat(&v)?,
                None => Repeat::None,
            };
            let repeat_until = repeat_until.map(|d| local_due_date(&d)).transpose()?;
            if repeat == Repeat::None && repeat_until.is_some() {
                anyhow::bail!("--repeat-until requires --repeat");
            }

            let mut tags = tag;
            let mut description = description;
            let mut estimate = estimate;
            if let Some(template) = template {
                for tag in template.tags {
                    if !tags.iter().any(|existing| existing == tag) {
                        tags.push((*tag).to_string());
                    }
                }
                if description.is_none() {
                    description = Some(template.description.to_string());
                }
                if estimate.is_none() {
                    estimate = template.estimate_minutes;
                }
            }
            let priority = pri
                .as_deref()
                .map(priority)
                .or_else(|| template.map(|template| template.priority.clone()))
                .unwrap_or_default();

            let config = config::load()?;
            let git_info = if config.git.auto_branch || config.git.link_commits {
                git::discover()
            } else {
                None
            };
            let target_branch = match &git_info {
                Some(info) if config.git.auto_branch => {
                    git::task_branch(&info.branch, &config.git.branch_prefix)
                        .unwrap_or_else(|| data.current_branch.clone())
                }
                _ => data.current_branch.clone(),
            };
            let auto_switch = config.git.auto_branch && target_branch != data.current_branch;

            let mut t = Task::new(title, project, target_branch.clone(), priority, tags, due);
            if let Some(desc) = description {
                t.description = desc;
            }
            t.estimate_minutes = estimate;
            t.commit = commit;
            if let Some(info) = &git_info {
                t.git_branch = info.branch.clone();
                t.git_remote = info.remote.clone();
                if t.commit.is_none() && config.git.link_commits {
                    t.commit = info.commit.clone();
                }
            }
            t.repeat = repeat;
            t.repeat_until = repeat_until;

            let warnings: Vec<(String, String, f64)> =
                duplicates::similar_to(&t, &data.tasks, &target_branch, false, 0.72)
                    .into_iter()
                    .take(3)
                    .map(|(other, score)| {
                        (
                            other.id.to_string()[..8].to_string(),
                            other.title.clone(),
                            score,
                        )
                    })
                    .collect();
            let id = t.id.to_string();
            let previous = data.clone();
            if auto_switch {
                if !data.branches.iter().any(|branch| branch == &target_branch) {
                    data.branches.push(target_branch.clone());
                }
                data.current_branch = target_branch;
            }
            data.tasks.push(t);
            store::save_with_undo(&data, &previous)?;
            println!("{}", id);
            for (other_id, title, score) in warnings {
                eprintln!(
                    "warning: possible duplicate: similar task {other_id} {title} ({:.0}% match)",
                    score * 100.0
                );
            }
        }
        Cmd::List {
            project,
            tag,
            status,
            search,
            all,
            view,
            save,
        } => {
            if view.is_some() && save.is_some() {
                anyhow::bail!("use either --view or --save, not both");
            }
            if let Some(name) = save {
                validate_view_name(&name)?;
                let filter = saved_view(project, tag, status, search, all);
                save_view(&name, &filter)?;
                println!("saved view {name}");
            } else {
                let filter = if let Some(name) = view {
                    let mut filter = load_saved_view(&name)?;
                    if let Some(project) = project {
                        filter.project = Some(project);
                    }
                    if let Some(tag) = tag {
                        filter.tag = Some(tag);
                    }
                    if let Some(status) = status {
                        filter.status = Some(status);
                    }
                    if let Some(search) = search {
                        filter.search = Some(search);
                    }
                    if all {
                        filter.all = true;
                    }
                    filter
                } else {
                    saved_view(project, tag, status, search, all)
                };
                print_task_list(&data, &filter);
            }
        }
        Cmd::Complete {
            id,
            force,
            done_before,
            all,
            commit,
        } => {
            if commit.is_some() && id.is_none() {
                anyhow::bail!("--commit requires a single task id");
            }
            if commit
                .as_deref()
                .is_some_and(|value| value.trim().is_empty())
            {
                anyhow::bail!("--commit must not be empty");
            }
            let _lock = store::StoreLock::acquire()?;
            match (id, done_before) {
                (Some(_), Some(_)) => {
                    anyhow::bail!("use either a task id or --done-before, not both")
                }
                (Some(id), None) => {
                    let (tid, open_children) = {
                        let t = store::find(&data, &id)?;
                        (t.id.to_string(), t.open_child_count(&data.tasks))
                    };
                    if !force && open_children > 0 {
                        anyhow::bail!(
                            "task has {open_children} open subtask(s); pass --force to complete it anyway"
                        );
                    }
                    let previous = data.clone();
                    let next = {
                        let t = store::find_mut(&mut data, &id)?;
                        if let Some(commit) = &commit {
                            t.commit = Some(commit.clone());
                            if let Some(info) = git::discover() {
                                t.git_branch = info.branch;
                                t.git_remote = info.remote;
                            }
                        }
                        t.set_status(Status::Done)
                    };
                    if let Some(next) = next {
                        let nid = next.id.to_string();
                        data.tasks.push(next);
                        store::save_with_undo(&data, &previous)?;
                        println!("done {}; next {}", &tid[..8], &nid[..8]);
                    } else {
                        store::save_with_undo(&data, &previous)?;
                        println!("done {}", &tid[..8]);
                    }
                }
                (None, Some(date)) => {
                    let cutoff = local_due_date(&date)?;
                    let candidates: Vec<uuid::Uuid> = data
                        .tasks
                        .iter()
                        .filter(|task| task.status != Status::Done)
                        .filter(|task| task.due.is_some_and(|due| due <= cutoff))
                        .filter(|task| all || task.branch == data.current_branch)
                        .map(|task| task.id)
                        .collect();
                    if candidates.is_empty() {
                        println!("no open tasks due on or before {date}");
                    } else {
                        let previous = data.clone();
                        let mut completed = 0;
                        let mut skipped = 0;
                        for task_id in candidates {
                            let open_children = data
                                .tasks
                                .iter()
                                .find(|task| task.id == task_id)
                                .map(|task| task.open_child_count(&data.tasks))
                                .unwrap_or(0);
                            if !force && open_children > 0 {
                                skipped += 1;
                                println!(
                                    "skipped {} ({} open subtask(s))",
                                    &task_id.to_string()[..8],
                                    open_children
                                );
                                continue;
                            }
                            let (tid, next) = {
                                let task = data
                                    .tasks
                                    .iter_mut()
                                    .find(|task| task.id == task_id)
                                    .expect("candidate task exists");
                                let next = task.set_status(Status::Done);
                                (task.id.to_string(), next)
                            };
                            if let Some(next) = next {
                                let nid = next.id.to_string();
                                data.tasks.push(next);
                                println!("done {}; next {}", &tid[..8], &nid[..8]);
                            } else {
                                println!("done {}", &tid[..8]);
                            }
                            completed += 1;
                        }
                        if completed > 0 {
                            store::save_with_undo(&data, &previous)?;
                        }
                        println!("completed {completed} task(s) due on or before {date}");
                        if skipped > 0 {
                            println!(
                                "skipped {skipped} task(s) with open subtasks; pass --force to complete them"
                            );
                        }
                    }
                }
                (None, None) => {
                    anyhow::bail!("provide a task id or --done-before <date>")
                }
            }
        }
        Cmd::Reopen { id } => {
            let _lock = store::StoreLock::acquire()?;
            let previous = data.clone();
            let tid = {
                let t = store::find_mut(&mut data, &id)?;
                t.set_status(Status::Todo);
                t.id.to_string()
            };
            store::save_with_undo(&data, &previous)?;
            println!("reopened {}", &tid[..8]);
        }
        Cmd::Doing { id } => {
            let _lock = store::StoreLock::acquire()?;
            let previous = data.clone();
            let tid = {
                let t = store::find_mut(&mut data, &id)?;
                t.set_status(Status::Doing);
                t.id.to_string()
            };
            store::save_with_undo(&data, &previous)?;
            println!("doing {}", &tid[..8]);
        }
        Cmd::Start { id } => {
            let _lock = store::StoreLock::acquire()?;
            let now = chrono::Utc::now();
            let target = store::find(&data, &id)?;
            if target.status == Status::Done {
                bail!("cannot start a completed task; run `tx reopen` first");
            }
            if let Some(active) = data
                .tasks
                .iter()
                .find(|task| task.active_time_entry().is_some())
            {
                let active_id = active.id.to_string();
                bail!(
                    "timer already running for {} {}; run `tx stop` first",
                    &active_id[..8],
                    active.title
                );
            }
            let tid = target.id.to_string();
            let title = target.title.clone();
            let previous = data.clone();
            {
                let task = store::find_mut(&mut data, &id)?;
                task.set_status(Status::Doing);
                task.start_timer(now);
            }
            store::save_with_undo(&data, &previous)?;
            println!("started {} {}", &tid[..8], title);
        }
        Cmd::Stop => {
            let _lock = store::StoreLock::acquire()?;
            let now = chrono::Utc::now();
            if !data
                .tasks
                .iter()
                .any(|task| task.active_time_entry().is_some())
            {
                bail!("no running timer");
            }
            let previous = data.clone();
            let mut stopped = Vec::new();
            for task in &mut data.tasks {
                let mut elapsed = 0;
                let mut sessions = 0;
                while task.active_time_entry().is_some() {
                    let Some(seconds) = task.stop_timer(now) else {
                        break;
                    };
                    elapsed += seconds;
                    sessions += 1;
                }
                if sessions > 0 {
                    stopped.push((task.id.to_string(), task.title.clone(), elapsed));
                }
            }
            store::save_with_undo(&data, &previous)?;
            for (id, title, elapsed) in stopped {
                println!(
                    "stopped {}  {}  {}",
                    &id[..8],
                    format_duration(elapsed),
                    title
                );
            }
        }
        Cmd::Time { id } => {
            let now = chrono::Utc::now();
            if let Some(id) = id {
                let task = store::find(&data, &id)?;
                let total = task.tracked_seconds(now);
                println!("{}", task.title);
                if let Some(estimate) = task.estimate_minutes {
                    println!("  estimate  {}", format_duration(u64::from(estimate) * 60));
                }
                println!("  actual    {}", format_duration(total));
                if task.time_entries.is_empty() {
                    println!("  no time entries");
                }
                for entry in &task.time_entries {
                    let start = entry
                        .started_at
                        .with_timezone(&chrono::Local)
                        .format("%Y-%m-%d %H:%M");
                    let elapsed = format_duration(entry.duration_seconds(now));
                    if let Some(end) = entry.ended_at {
                        let end = end.with_timezone(&chrono::Local).format("%Y-%m-%d %H:%M");
                        println!("  {start} -> {end}  {elapsed}");
                    } else {
                        println!("  {start} -> active  {elapsed}");
                    }
                }
            } else {
                let mut total = 0;
                let mut rows = 0;
                for task in &data.tasks {
                    if task.time_entries.is_empty() {
                        continue;
                    }
                    let seconds = task.tracked_seconds(now);
                    total += seconds;
                    rows += 1;
                    let active = if task.active_time_entry().is_some() {
                        "ACTIVE"
                    } else {
                        "      "
                    };
                    let estimate = task
                        .estimate_minutes
                        .map(|minutes| format!("est {}", format_duration(u64::from(minutes) * 60)))
                        .unwrap_or_else(|| "est -".to_string());
                    let id = task.id.to_string();
                    println!(
                        "{active}  {}  {:>8}  {:>10}  {}",
                        &id[..8],
                        format_duration(seconds),
                        estimate,
                        task.title
                    );
                }
                if rows == 0 {
                    println!("no time tracked");
                } else {
                    println!("total {}", format_duration(total));
                }
            }
        }
        Cmd::View { action } => match action {
            ViewCmd::Save {
                name,
                project,
                tag,
                status,
                search,
                all,
            } => {
                let filter = saved_view(project, tag, status, search, all);
                save_view(&name, &filter)?;
                println!("saved view {name}");
            }
            ViewCmd::List => {
                let config = config::load()?;
                if config.views.is_empty() {
                    println!("no saved views");
                } else {
                    for (name, view) in &config.views {
                        println!("{}", view_line(name, view));
                    }
                }
            }
            ViewCmd::Show { name } => {
                let view = load_saved_view(&name)?;
                println!("{}", view_line(&name, &view));
            }
            ViewCmd::Run { name } => {
                let view = load_saved_view(&name)?;
                print_task_list(&data, &view);
            }
            ViewCmd::Delete { name } => {
                validate_view_name(&name)?;
                let mut config = config::load()?;
                if config.views.remove(&name).is_none() {
                    bail!("view not found: {name}");
                }
                config::save(&config)?;
                println!("deleted view {name}");
            }
        },
        Cmd::Duplicates { threshold, all } => {
            if !(0.0..=1.0).contains(&threshold) {
                bail!("--threshold must be between 0.0 and 1.0");
            }
            let pairs =
                duplicates::similar_pairs(&data.tasks, &data.current_branch, all, threshold);
            if pairs.is_empty() {
                println!("no similar open tasks found");
            } else {
                for (left, right, score) in pairs {
                    println!(
                        "{:.2}  {}  {}  ~  {}  {}",
                        score,
                        &left.id.to_string()[..8],
                        left.title,
                        &right.id.to_string()[..8],
                        right.title
                    );
                }
            }
        }
        Cmd::Delete { id } => {
            let _lock = store::StoreLock::acquire()?;
            let previous = data.clone();
            let tid = store::delete_to_trash(&mut data, &id)?;
            store::save_with_undo(&data, &previous)?;
            println!(
                "deleted {}; run `tx undo` to restore",
                &tid.to_string()[..8]
            );
        }
        Cmd::Undo => {
            let _lock = store::StoreLock::acquire()?;
            let snapshot = store::load_undo()?;
            store::save(&snapshot)?;
            store::clear_undo()?;
            println!("undone");
        }
        Cmd::Next {
            project,
            all,
            include_blocked,
            explain,
        } => {
            let now = chrono::Utc::now();
            let mut candidates: Vec<&Task> = data
                .tasks
                .iter()
                .filter(|t| t.status != Status::Done)
                .filter(|t| all || t.branch == data.current_branch)
                .filter(|t| project.as_ref().map_or(true, |p| &t.project == p))
                .filter(|t| include_blocked || !t.is_blocked(&data.tasks))
                .collect();
            candidates.sort_by_key(|t| model::next_key(t, now));
            if let Some(t) = candidates.first() {
                println!(
                    "{}  {:7} {:10} {:12} {}",
                    &t.id.to_string()[..8],
                    t.priority,
                    due_display(t),
                    t.project,
                    t.title
                );
                if explain {
                    let blocked_by = if t.blocked_by.is_empty() {
                        "none".to_string()
                    } else {
                        t.blocked_by
                            .iter()
                            .map(|id| id.to_string()[..8].to_string())
                            .collect::<Vec<_>>()
                            .join(",")
                    };
                    println!(
                        "  status={} priority={} due={} blocked_by={}",
                        t.status,
                        t.priority,
                        due_display(t),
                        blocked_by
                    );
                }
            } else {
                println!("no actionable tasks");
            }
        }
        Cmd::Block { id, by } => {
            let _lock = store::StoreLock::acquire()?;
            let target = store::find(&data, &id)?.id;
            let blocker = store::find(&data, &by)?.id;
            if target == blocker {
                anyhow::bail!("a task cannot block itself");
            }
            let previous = data.clone();
            {
                let t = store::find_mut(&mut data, &id)?;
                if !t.blocked_by.contains(&blocker) {
                    t.blocked_by.push(blocker);
                }
                t.updated_at = chrono::Utc::now();
            }
            store::save_with_undo(&data, &previous)?;
            println!(
                "blocked {} by {}",
                &target.to_string()[..8],
                &blocker.to_string()[..8]
            );
        }
        Cmd::Unblock { id, by } => {
            let _lock = store::StoreLock::acquire()?;
            let blocker = match by {
                Some(by) => Some(store::find(&data, &by)?.id),
                None => None,
            };
            let previous = data.clone();
            let tid = {
                let t = store::find_mut(&mut data, &id)?;
                if let Some(blocker) = blocker {
                    t.blocked_by.retain(|x| *x != blocker);
                } else {
                    t.blocked_by.clear();
                }
                t.updated_at = chrono::Utc::now();
                t.id.to_string()
            };
            store::save_with_undo(&data, &previous)?;
            println!("unblocked {}", &tid[..8]);
        }
        Cmd::Subtask {
            parent,
            title,
            priority: pri,
            due,
        } => {
            let _lock = store::StoreLock::acquire()?;
            let (parent_id, project, branch, parent_priority) = {
                let p = store::find(&data, &parent)?;
                (
                    p.id,
                    p.project.clone(),
                    p.branch.clone(),
                    p.priority.clone(),
                )
            };
            let due = due.map(|d| local_due_date(&d)).transpose()?;
            let pri = pri.map(|v| priority(&v)).unwrap_or(parent_priority);
            let mut t = Task::new(title, project, branch, pri, vec![], due);
            t.parent = Some(parent_id);
            let id = t.id.to_string();
            store::add(&mut data, t)?;
            println!("{}", id);
        }
        Cmd::Tree { id, all } => {
            let roots: Vec<&Task> = match id {
                Some(id) => vec![store::find(&data, &id)?],
                None => data
                    .tasks
                    .iter()
                    .filter(|t| t.parent.is_none() && (all || t.branch == data.current_branch))
                    .collect(),
            };
            if roots.is_empty() {
                println!("no tasks");
            }
            let mut seen = std::collections::HashSet::new();
            for root in roots {
                print_tree(&data, root, 0, &mut seen);
            }
        }
        Cmd::Trash { action } => match action {
            TrashCmd::List => {
                if data.trash.is_empty() {
                    println!("trash is empty");
                } else {
                    for entry in &data.trash {
                        println!(
                            "{}  {}  {}",
                            &entry.task.id.to_string()[..8],
                            entry
                                .deleted_at
                                .with_timezone(&chrono::Local)
                                .format("%Y-%m-%d %H:%M"),
                            entry.task.title
                        );
                    }
                }
            }
            TrashCmd::Restore { id } => {
                let _lock = store::StoreLock::acquire()?;
                let previous = data.clone();
                let tid = store::restore_from_trash(&mut data, &id)?;
                store::save_with_undo(&data, &previous)?;
                println!("restored {}", &tid.to_string()[..8]);
            }
            TrashCmd::Purge => {
                let _lock = store::StoreLock::acquire()?;
                let count = data.trash.len();
                let previous = data.clone();
                data.trash.clear();
                store::save_with_undo(&data, &previous)?;
                println!("purged {count} task(s)");
            }
        },
        Cmd::Git { action } => match action {
            GitCmd::Status => {
                let config = config::load()?;
                match git::discover() {
                    Some(info) => {
                        println!("repo: {}", info.root.display());
                        println!("git branch: {}", info.branch);
                        println!("head: {}", info.commit.as_deref().unwrap_or("-"));
                        println!("remote: {}", info.remote.as_deref().unwrap_or("-"));
                        println!("tasklynx branch: {}", data.current_branch);
                        let mapped = git::task_branch(&info.branch, &config.git.branch_prefix);
                        println!(
                            "mapped tasklynx branch: {}",
                            mapped.as_deref().unwrap_or("-")
                        );
                        if mapped
                            .as_deref()
                            .is_some_and(|branch| branch != data.current_branch)
                        {
                            println!("note: git branch differs from tasklynx branch");
                        }
                        let commits = git::recent_commits(5).unwrap_or_default();
                        if !commits.is_empty() {
                            println!("recent commits:");
                            for commit in commits {
                                println!(
                                    "  {}  {}",
                                    &commit.sha[..commit.sha.len().min(12)],
                                    commit.body.lines().next().unwrap_or("")
                                );
                            }
                        }
                        let open: Vec<&Task> = data
                            .tasks
                            .iter()
                            .filter(|t| t.status != Status::Done && t.branch == data.current_branch)
                            .collect();
                        println!("open tasks: {}", open.len());
                        for t in open {
                            println!(
                                "  {}  {:7} {:10} {}",
                                &t.id.to_string()[..8],
                                t.priority,
                                due_display(t),
                                t.title
                            );
                        }
                    }
                    None => println!("not in a git repository"),
                }
            }
            GitCmd::Link {
                id,
                commit,
                pr,
                issue,
            } => {
                let info =
                    git::discover().ok_or_else(|| anyhow::anyhow!("not in a git repository"))?;
                for (flag, url) in [("--pr", pr.as_deref()), ("--issue", issue.as_deref())] {
                    if let Some(url) = url {
                        if !url.starts_with("http://") && !url.starts_with("https://") {
                            anyhow::bail!("{flag} must be an http(s) URL");
                        }
                    }
                }
                let commit = commit.or_else(|| info.commit.clone()).ok_or_else(|| {
                    anyhow::anyhow!("no commit to link; pass --commit or create a commit first")
                })?;
                let _lock = store::StoreLock::acquire()?;
                let previous = data.clone();
                let tid = {
                    let t = store::find_mut(&mut data, &id)?;
                    t.commit = Some(commit.clone());
                    t.git_branch = info.branch.clone();
                    t.git_remote = info.remote.clone();
                    if let Some(pr) = pr {
                        t.git_pr_url = Some(pr);
                    }
                    if let Some(issue) = issue {
                        t.git_issue_url = Some(issue);
                    }
                    t.updated_at = chrono::Utc::now();
                    t.id.to_string()
                };
                store::save_with_undo(&data, &previous)?;
                println!("linked {} to {} on {}", &tid[..8], commit, info.branch);
            }
            GitCmd::Sync { limit } => {
                let info =
                    git::discover().ok_or_else(|| anyhow::anyhow!("not in a git repository"))?;
                let commits = git::recent_commits(limit).unwrap_or_default();
                if commits.is_empty() {
                    println!("no commits found");
                } else {
                    let _lock = store::StoreLock::acquire()?;
                    let previous = data.clone();
                    let mut linked = 0usize;
                    let mut matched = 0usize;
                    for commit in commits {
                        let Some(task_id) = commit_task_id(&commit.body) else {
                            continue;
                        };
                        matched += 1;
                        let short = commit.sha.get(..12).unwrap_or(&commit.sha).to_string();
                        let Ok(task) = store::find_mut(&mut data, &task_id) else {
                            continue;
                        };
                        if task.commit.as_deref() == Some(short.as_str())
                            && task.git_branch == info.branch
                        {
                            continue;
                        }
                        task.commit = Some(short);
                        task.git_branch = info.branch.clone();
                        task.git_remote = info.remote.clone();
                        task.updated_at = chrono::Utc::now();
                        linked += 1;
                    }
                    if linked > 0 {
                        store::save_with_undo(&data, &previous)?;
                        println!("linked {linked} commit(s)");
                    } else if matched > 0 {
                        println!("no new commit links");
                    } else {
                        println!("no matching Tasklynx-Id trailers");
                    }
                }
            }
            GitCmd::Scan {
                apply,
                limit,
                project,
            } => {
                let info =
                    git::discover().ok_or_else(|| anyhow::anyhow!("not in a git repository"))?;
                let tasks = scan_repo(&data, &info, &project, limit);
                if tasks.is_empty() {
                    println!("no new TODO/FIXME/HACK comments found");
                } else {
                    for task in &tasks {
                        println!(
                            "{}:{}  {}",
                            task.source_path.as_deref().unwrap_or("?"),
                            task.source_line.unwrap_or(0),
                            task.title
                        );
                    }
                    if apply {
                        let _lock = store::StoreLock::acquire()?;
                        let count = tasks.len();
                        let previous = data.clone();
                        data.tasks.extend(tasks);
                        store::save_with_undo(&data, &previous)?;
                        println!("created {count} task(s)");
                    } else {
                        println!(
                            "found {} item(s); run with --apply to create tasks",
                            tasks.len()
                        );
                    }
                }
            }
            GitCmd::Open { id, print } => {
                let task = store::find(&data, &id)?;
                let url = match (task.git_pr_url.as_deref(), task.git_issue_url.as_deref()) {
                    (Some(url), _) | (None, Some(url)) => url.to_string(),
                    (None, None) => {
                        let commit = task.commit.as_deref().ok_or_else(|| {
                            anyhow::anyhow!(
                                "task has no commit or PR link and no issue link; run `tx git link` first"
                            )
                        })?;
                        let remote = task.git_remote.as_deref().ok_or_else(|| {
                            anyhow::anyhow!("task has no Git remote; run `tx git link` first")
                        })?;
                        git::commit_url(remote, commit).ok_or_else(|| {
                            anyhow::anyhow!("could not build a commit URL from remote '{remote}'")
                        })?
                    }
                };
                if print {
                    println!("{url}");
                } else {
                    open_url(&url)?;
                    println!("opened {url}");
                }
            }
            GitCmd::Hook { action } => match action {
                HookCmd::Install => {
                    let info = git::discover()
                        .ok_or_else(|| anyhow::anyhow!("not in a git repository"))?;
                    let path = install_git_hook(&info)?;
                    println!("installed hook: {}", path.display());
                }
                HookCmd::Uninstall => {
                    let info = git::discover()
                        .ok_or_else(|| anyhow::anyhow!("not in a git repository"))?;
                    let path = uninstall_git_hook(&info)?;
                    println!("removed hook: {}", path.display());
                }
            },
            GitCmd::Activate { id } => {
                let info =
                    git::discover().ok_or_else(|| anyhow::anyhow!("not in a git repository"))?;
                let task = store::find(&data, &id)?;
                let full = task.id.to_string();
                let short = full[..8].to_string();
                let title = task.title.clone();
                let status = Command::new("git")
                    .current_dir(&info.root)
                    .args(["config", "tasklynx.taskId", &full])
                    .status()?;
                if !status.success() {
                    anyhow::bail!("failed to set git config tasklynx.taskId");
                }
                println!("active task: {short} {title}");
            }
            GitCmd::Deactivate => {
                let info =
                    git::discover().ok_or_else(|| anyhow::anyhow!("not in a git repository"))?;
                let _ = Command::new("git")
                    .current_dir(&info.root)
                    .args(["config", "--unset", "tasklynx.taskId"])
                    .output()?;
                println!("deactivated");
            }
        },
        Cmd::Ai { action } => match action {
            AiCmd::Plan {
                goal,
                apply,
                project,
                limit,
                no_description,
            } => {
                if limit == 0 || limit > 100 {
                    anyhow::bail!("--limit must be between 1 and 100");
                }
                let config = config::load()?;
                let provider = ai::provider(&config.ai)?;
                announce_ai(provider.as_ref(), &config);
                let include_descriptions = config.ai.include_descriptions && !no_description;
                let plan = ai::request_plan(
                    provider.as_ref(),
                    &config.ai,
                    ai::PlanRequest {
                        goal: &goal,
                        tasks: &data.tasks,
                        branch: &data.current_branch,
                        project: project.as_deref(),
                        limit,
                        include_descriptions,
                    },
                )?;
                let default_project = project.as_deref().unwrap_or(&config.default_project);
                let tasks = ai::plan_tasks(&plan, &data, default_project)?;
                if let Some(plan_project) = &plan.project {
                    println!("project: {plan_project}");
                }
                print_ai_tasks(if apply { "tasks" } else { "preview" }, &tasks);
                if apply {
                    let _lock = store::StoreLock::acquire()?;
                    let count = tasks.len();
                    let previous = data.clone();
                    data.tasks.extend(tasks);
                    store::save_with_undo(&data, &previous)?;
                    println!("created {count} task(s)");
                } else {
                    println!("preview only; pass --apply to create these tasks");
                }
            }
            AiCmd::Breakdown {
                id,
                apply,
                limit,
                no_description,
            } => {
                if limit == 0 || limit > 100 {
                    anyhow::bail!("--limit must be between 1 and 100");
                }
                let parent = store::find(&data, &id)?.clone();
                let children: Vec<Task> = data
                    .tasks
                    .iter()
                    .filter(|task| task.parent == Some(parent.id))
                    .cloned()
                    .collect();
                let config = config::load()?;
                let provider = ai::provider(&config.ai)?;
                announce_ai(provider.as_ref(), &config);
                let include_descriptions = config.ai.include_descriptions && !no_description;
                let plan = ai::request_breakdown(
                    provider.as_ref(),
                    &config.ai,
                    &parent,
                    &children,
                    limit,
                    include_descriptions,
                )?;
                let tasks = ai::breakdown_tasks(&parent, &plan)?;
                println!("parent: {} {}", &parent.id.to_string()[..8], parent.title);
                print_ai_tasks(if apply { "tasks" } else { "preview" }, &tasks);
                if apply {
                    let _lock = store::StoreLock::acquire()?;
                    let count = tasks.len();
                    let previous = data.clone();
                    data.tasks.extend(tasks);
                    store::save_with_undo(&data, &previous)?;
                    println!("created {count} subtask(s)");
                } else {
                    println!("preview only; pass --apply to create these subtasks");
                }
            }
            AiCmd::Triage {
                limit,
                project,
                include_blocked,
                no_description,
                offline,
            } => {
                if limit == 0 || limit > 50 {
                    anyhow::bail!("--limit must be between 1 and 50");
                }
                let config = config::load()?;
                let include_descriptions = config.ai.include_descriptions && !no_description;
                let deterministic =
                    offline || matches!(config.ai.provider_kind()?, ai::ProviderKind::None);
                let picks = if ai::triage_candidate_count(
                    &data.tasks,
                    &data.current_branch,
                    project.as_deref(),
                    include_blocked,
                ) == 0
                {
                    println!("no eligible tasks to triage");
                    Vec::new()
                } else if deterministic {
                    let request = ai::TriageRequest {
                        tasks: &data.tasks,
                        branch: &data.current_branch,
                        project: project.as_deref(),
                        limit,
                        include_blocked,
                        include_descriptions,
                    };
                    let picks = ai::deterministic_triage(request);
                    println!("offline triage: {} task(s)", picks.len());
                    picks
                } else {
                    let provider = ai::provider(&config.ai)?;
                    announce_ai(provider.as_ref(), &config);
                    let response = ai::request_triage(
                        provider.as_ref(),
                        &config.ai,
                        ai::TriageRequest {
                            tasks: &data.tasks,
                            branch: &data.current_branch,
                            project: project.as_deref(),
                            limit,
                            include_blocked,
                            include_descriptions,
                        },
                    )?;
                    println!("{}", response.summary);
                    ai::resolve_triage(
                        &response,
                        &data.tasks,
                        &data.current_branch,
                        project.as_deref(),
                        include_blocked,
                    )?
                };
                print_triage_picks(&data, &picks);
            }
            AiCmd::Standup {
                days,
                project,
                limit,
                no_description,
            } => {
                if !(1..=365).contains(&days) {
                    anyhow::bail!("--days must be between 1 and 365");
                }
                if limit == 0 || limit > 50 {
                    anyhow::bail!("--limit must be between 1 and 50");
                }
                let config = config::load()?;
                let provider = ai::provider(&config.ai)?;
                announce_ai(provider.as_ref(), &config);
                let include_descriptions = config.ai.include_descriptions && !no_description;
                let report = ai::request_report(
                    provider.as_ref(),
                    &config.ai,
                    ai::ReportRequest {
                        kind: ai::ReportKind::Standup,
                        tasks: &data.tasks,
                        branch: &data.current_branch,
                        project: project.as_deref(),
                        days,
                        limit,
                        include_descriptions,
                    },
                )?;
                print_report(&report);
            }
            AiCmd::Review {
                branch,
                days,
                project,
                limit,
                no_description,
            } => {
                if !(1..=3650).contains(&days) {
                    anyhow::bail!("--days must be between 1 and 3650");
                }
                if limit == 0 || limit > 50 {
                    anyhow::bail!("--limit must be between 1 and 50");
                }
                let config = config::load()?;
                let provider = ai::provider(&config.ai)?;
                announce_ai(provider.as_ref(), &config);
                let include_descriptions = config.ai.include_descriptions && !no_description;
                let branch = branch.unwrap_or_else(|| data.current_branch.clone());
                let report = ai::request_report(
                    provider.as_ref(),
                    &config.ai,
                    ai::ReportRequest {
                        kind: ai::ReportKind::Review,
                        tasks: &data.tasks,
                        branch: &branch,
                        project: project.as_deref(),
                        days,
                        limit,
                        include_descriptions,
                    },
                )?;
                print_report(&report);
            }
            AiCmd::Ask {
                question,
                project,
                branch,
                limit,
                no_description,
            } => {
                if limit == 0 || limit > 100 {
                    anyhow::bail!("--limit must be between 1 and 100");
                }
                let config = config::load()?;
                let provider = ai::provider(&config.ai)?;
                announce_ai(provider.as_ref(), &config);
                let include_descriptions = config.ai.include_descriptions && !no_description;
                let branch = branch.unwrap_or_else(|| data.current_branch.clone());
                let answer = ai::request_answer(
                    provider.as_ref(),
                    &config.ai,
                    ai::AskRequest {
                        question: &question,
                        tasks: &data.tasks,
                        branch: &branch,
                        project: project.as_deref(),
                        limit,
                        include_descriptions,
                    },
                )?;
                println!("{}", answer.answer);
            }
            AiCmd::Doctor => {
                let config = config::load()?;
                let provider = ai::provider(&config.ai)?;
                println!("provider: {}", provider.name());
                println!("endpoint: {}", config.ai.endpoint);
                println!("model: {}", config.ai.model);
                for line in provider.doctor()? {
                    println!("{line}");
                }
            }
        },
        Cmd::Edit {
            id,
            title,
            description,
            priority: pri,
            due,
            project,
            estimate,
            commit,
            add_tag,
            remove_tag,
            repeat,
            repeat_until,
        } => {
            let _lock = store::StoreLock::acquire()?;
            let previous = data.clone();
            let tid = store::find_mut(&mut data, &id)?.id.to_string();
            {
                let t = store::find_mut(&mut data, &id)?;
                if let Some(v) = title {
                    t.title = v;
                }
                if let Some(v) = description {
                    t.description = v;
                }
                if let Some(v) = pri {
                    t.priority = priority(&v);
                }
                if let Some(v) = due {
                    t.due = Some(local_due_date(&v)?);
                }
                if let Some(v) = repeat {
                    t.repeat = parse_repeat(&v)?;
                    if t.repeat == Repeat::None {
                        t.repeat_until = None;
                    }
                }
                if let Some(v) = repeat_until {
                    if t.repeat == Repeat::None {
                        anyhow::bail!("--repeat-until requires a repeat value");
                    }
                    t.repeat_until = Some(local_due_date(&v)?);
                }
                if let Some(v) = project {
                    t.project = v;
                }
                if let Some(v) = estimate {
                    t.estimate_minutes = Some(v);
                }
                if let Some(v) = commit {
                    t.commit = Some(v);
                }
                for tag in add_tag {
                    if !t.tags.contains(&tag) {
                        t.tags.push(tag);
                    }
                }
                for tag in remove_tag {
                    t.tags.retain(|x| x != &tag);
                }
                t.updated_at = chrono::Utc::now();
            }
            store::save_with_undo(&data, &previous)?;
            println!("edited {}", &tid[..8]);
        }
        Cmd::Duplicate { id } => {
            let _lock = store::StoreLock::acquire()?;
            let t = store::find(&data, &id)?.clone();
            let previous = data.clone();
            let mut n = t;
            n.id = uuid::Uuid::new_v4();
            n.title = format!("Copy: {}", n.title);
            n.status = Status::Todo;
            n.completed_at = None;
            n.time_entries.clear();
            n.created_at = chrono::Utc::now();
            n.updated_at = n.created_at;
            data.tasks.push(n.clone());
            let nid = n.id.to_string();
            store::save_with_undo(&data, &previous)?;
            println!("{}", nid);
        }
        Cmd::Search { query } => {
            for t in &data.tasks {
                if t.title.to_lowercase().contains(&query.to_lowercase())
                    || t.description.to_lowercase().contains(&query.to_lowercase())
                    || t.tags
                        .iter()
                        .any(|x| x.to_lowercase().contains(&query.to_lowercase()))
                {
                    println!("{} {:12} {}", &t.id.to_string()[..8], t.project, t.title)
                }
            }
        }
        Cmd::Today => {
            for t in &data.tasks {
                if t.overdue() {
                    println!("OVERDUE {} {}", &t.id.to_string()[..8], t.title)
                } else if t
                    .due
                    .map(|d| {
                        d.with_timezone(&chrono::Local).date_naive()
                            == chrono::Local::now().date_naive()
                    })
                    .unwrap_or(false)
                {
                    println!("TODAY {} {}", &t.id.to_string()[..8], t.title)
                }
            }
        }
        Cmd::Agenda { days } => {
            let now = chrono::Utc::now();
            let horizon = now + chrono::Duration::days(i64::from(days));
            let mut entries: Vec<(chrono::DateTime<chrono::Utc>, &str, &Task)> = Vec::new();
            for t in &data.tasks {
                if let Some(due) = t.due {
                    if t.status != model::Status::Done && due <= horizon {
                        let label = if due < now { "OVERDUE" } else { "UPCOMING" };
                        entries.push((due, label, t));
                    }
                }
                if t.repeat != Repeat::None {
                    if let Some(next) = t.next_due_date() {
                        if next >= now && next <= horizon {
                            entries.push((next, "NEXT", t));
                        }
                    }
                }
            }
            entries.sort_by_key(|(due, _, _)| *due);
            for (due, label, t) in entries {
                let date = due.with_timezone(&chrono::Local).format("%Y-%m-%d");
                println!(
                    "{:8} {}  {}  {}",
                    label,
                    &t.id.to_string()[..8],
                    date,
                    t.title
                );
            }
        }
        Cmd::Remind { days, notify } => {
            if days > 3650 {
                bail!("--days must be between 0 and 3650");
            }
            let now = chrono::Utc::now();
            let due = reminders::upcoming(&data.tasks, days, now);
            if due.is_empty() {
                println!("nothing due in the next {days} day(s)");
            } else {
                for task in &due {
                    let due_at = task.due.expect("reminder tasks always have a due date");
                    let date = due_at.with_timezone(&chrono::Local).format("%Y-%m-%d");
                    println!(
                        "{:8} {}  {}  {:7}  {}",
                        reminders::reminder_label(due_at, now),
                        &task.id.to_string()[..8],
                        date,
                        task.priority,
                        task.title
                    );
                }
                if notify {
                    let (title, body) = reminders::notification_text(&due);
                    reminders::notify(&title, &body)?;
                    eprintln!("notification sent");
                }
            }
        }
        Cmd::Stats => {
            let done = data
                .tasks
                .iter()
                .filter(|x| x.status == model::Status::Done)
                .count();
            let overdue = data.tasks.iter().filter(|x| x.overdue()).count();
            println!(
                "tasks={} done={} open={} overdue={} branches={} projects={}",
                data.tasks.len(),
                done,
                data.tasks.len() - done,
                overdue,
                data.branches.len(),
                data.tasks
                    .iter()
                    .map(|x| &x.project)
                    .collect::<std::collections::HashSet<_>>()
                    .len()
            );
        }
        Cmd::Branch { action } => match action {
            BranchCmd::List => {
                for b in &data.branches {
                    println!(
                        "{} {}",
                        if b == &data.current_branch { "*" } else { " " },
                        b
                    )
                }
            }
            BranchCmd::Create { name } => {
                let _lock = store::StoreLock::acquire()?;
                if !data.branches.contains(&name) {
                    let previous = data.clone();
                    data.branches.push(name.clone());
                    store::save_with_undo(&data, &previous)?;
                }
                println!("branch {}", name)
            }
            BranchCmd::Switch { name } => {
                let _lock = store::StoreLock::acquire()?;
                if !data.branches.contains(&name) {
                    return Err(anyhow::anyhow!("branch not found"));
                }
                let previous = data.clone();
                data.current_branch = name;
                store::save_with_undo(&data, &previous)?
            }
            BranchCmd::Delete { name } => {
                let _lock = store::StoreLock::acquire()?;
                if name == data.current_branch {
                    return Err(anyhow::anyhow!("cannot delete current branch"));
                }
                if data.tasks.iter().any(|x| x.branch == name) {
                    return Err(anyhow::anyhow!("branch is not empty"));
                }
                let previous = data.clone();
                data.branches.retain(|x| x != &name);
                store::save_with_undo(&data, &previous)?
            }
        },
        Cmd::Export { format } => match format.as_str() {
            "md" => print!("{}", store::export_md(&data)),
            "json" => println!("{}", store::export_json(&data)?),
            "ics" => print!("{}", reminders::export_ics(&data.tasks, chrono::Utc::now())),
            other => bail!("unsupported export format '{other}'; expected json, md, or ics"),
        },
        Cmd::Import {
            path,
            format,
            replace,
        } => {
            let _lock = store::StoreLock::acquire()?;
            let content = match path {
                Some(p) => std::fs::read_to_string(&p)?,
                None => {
                    use std::io::Read;
                    let mut buf = String::new();
                    std::io::stdin().read_to_string(&mut buf)?;
                    buf
                }
            };
            let is_md = format == "md" || content.trim_start().starts_with('#');
            let previous = data.clone();
            let (added, updated) = if is_md {
                store::import_md(&mut data, &content, replace)?
            } else {
                store::import_json(&mut data, &content, replace)?
            };
            store::save_with_undo(&data, &previous)?;
            if replace {
                println!("replaced store with {} tasks", added);
            } else {
                println!("imported: {} new, {} updated", added, updated);
            }
        }
        Cmd::Web { host, port } => {
            let c = config::load()?;
            web::serve(host.unwrap_or(c.host), port.unwrap_or(c.port), data).await?
        }
        Cmd::Board => tui::run(&mut data)?,
        Cmd::Config { action } => match action {
            ConfigCmd::Init => {
                config::save(&config::load()?)?;
                println!("{}", config::path().display())
            }
            ConfigCmd::Show => println!("{}", toml::to_string_pretty(&config::load()?)?),
        },
        Cmd::Completions { shell } => {
            let mut cmd = Cli::command();
            clap_complete::generate(shell, &mut cmd, "tasklynx", &mut std::io::stdout());
        }
        Cmd::Man => {
            let cmd = Cli::command();
            let man = clap_mangen::Man::new(cmd);
            man.render(&mut std::io::stdout())?;
        }
        Cmd::SelfCmd { action } => match action {
            SelfSubCmd::Update => {
                println!("Checking for updates...");
                let status = if cfg!(windows) {
                    std::process::Command::new("powershell")
                        .args([
                            "-NoProfile",
                            "-ExecutionPolicy",
                            "Bypass",
                            "-Command",
                            "irm https://raw.githubusercontent.com/peekstore/tasklynx/main/installer.ps1 | iex",
                        ])
                        .status()
                } else {
                    std::process::Command::new("sh")
                        .args([
                            "-c",
                            "curl -fsSL https://raw.githubusercontent.com/peekstore/tasklynx/main/installer.sh | sh",
                        ])
                        .status()
                };
                let status = status?;
                if !status.success() {
                    anyhow::bail!("update failed with status {status}");
                }
                println!("Update finished.");
            }
            SelfSubCmd::Uninstall { purge } => {
                let exe = std::env::current_exe()?;
                println!("Removing {}", exe.display());
                if let Some(dir) = exe.parent() {
                    for name in ["tx", "tx.exe"] {
                        let alias = dir.join(name);
                        if alias.exists() {
                            let _ = std::fs::remove_file(&alias);
                        }
                    }
                }
                #[cfg(windows)]
                {
                    let command = format!("/C timeout /t 2 >nul & del /f /q \"{}\"", exe.display());
                    match std::process::Command::new("cmd").arg(command).status() {
                        Ok(s) if s.success() => {}
                        _ => eprintln!(
                            "Could not schedule removal of {}. Delete it manually.",
                            exe.display()
                        ),
                    }
                }
                #[cfg(not(windows))]
                {
                    std::fs::remove_file(&exe)?;
                    if let Some(dir) = exe.parent() {
                        let _ = std::fs::remove_dir(dir);
                    }
                }

                if purge {
                    let _ = std::fs::remove_dir_all(store::data_dir());
                    let _ = std::fs::remove_dir_all(config::dir());
                    println!("Purged data and config directories.");
                } else {
                    println!("Note: config and data directories were left in place (pass --purge to remove).");
                }
            }
        },
    }
    Ok(())
}
