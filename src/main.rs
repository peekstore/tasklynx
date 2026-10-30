mod config;
mod git;
mod model;
mod store;
mod tui;
mod web;

use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
use model::{local_due_date, Priority, Repeat, Status, Task};
use std::path::PathBuf;

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
        title: String,
        #[arg(short, long, default_value = "default")]
        project: String,
        #[arg(short, long, value_delimiter = ',')]
        tag: Vec<String>,
        #[arg(short = 'P', long, default_value = "normal")]
        priority: String,
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
    },
    Complete {
        id: String,
        #[arg(long)]
        force: bool,
    },
    Reopen {
        id: String,
    },
    Doing {
        id: String,
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
    },
}

#[derive(Subcommand)]
enum BranchCmd {
    List,
    Create { name: String },
    Switch { name: String },
    Delete { name: String },
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

fn priority_rank(p: &Priority) -> u8 {
    match p {
        Priority::Urgent => 0,
        Priority::High => 1,
        Priority::Normal => 2,
        Priority::Low => 3,
    }
}

fn next_key(t: &Task, now: chrono::DateTime<chrono::Utc>) -> (u8, u8, u8, i64, i64) {
    let today = now.with_timezone(&chrono::Local).date_naive();
    let (due_bucket, due_ts) = match t.due {
        Some(d) if d < now => (0, d.timestamp()),
        Some(d) if d.with_timezone(&chrono::Local).date_naive() == today => (1, d.timestamp()),
        Some(d) => (2, d.timestamp()),
        None => (3, i64::MAX),
    };
    let status = match t.status {
        Status::Doing => 0,
        Status::Todo => 1,
        Status::Done => 2,
    };
    (
        priority_rank(&t.priority),
        due_bucket,
        status,
        due_ts,
        t.created_at.timestamp(),
    )
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
        } => {
            let _lock = store::StoreLock::acquire()?;
            let due = due.map(|d| local_due_date(&d)).transpose()?;
            let repeat = match repeat {
                Some(v) => parse_repeat(&v)?,
                None => Repeat::None,
            };
            let repeat_until = repeat_until.map(|d| local_due_date(&d)).transpose()?;
            if repeat == Repeat::None && repeat_until.is_some() {
                anyhow::bail!("--repeat-until requires --repeat");
            }
            let mut t = Task::new(
                title,
                project,
                data.current_branch.clone(),
                priority(&pri),
                tag,
                due,
            );
            if let Some(desc) = description {
                t.description = desc;
            }
            t.estimate_minutes = estimate;
            t.commit = commit;
            t.repeat = repeat;
            t.repeat_until = repeat_until;

            let id = t.id.to_string();
            store::add(&mut data, t)?;
            println!("{}", id);
        }
        Cmd::List {
            project,
            tag,
            status,
            search,
            all,
        } => {
            for t in &data.tasks {
                if !all && t.branch != data.current_branch {
                    continue;
                }
                if project.as_ref().is_some_and(|x| &t.project != x) {
                    continue;
                }
                if tag.as_ref().is_some_and(|x| !t.tags.iter().any(|z| z == x)) {
                    continue;
                }
                if status.as_ref().is_some_and(|x| t.status.to_string() != *x) {
                    continue;
                }
                if search.as_ref().is_some_and(|x| {
                    !t.title.to_lowercase().contains(&x.to_lowercase())
                        && !t.description.to_lowercase().contains(&x.to_lowercase())
                }) {
                    continue;
                }
                let recur = if t.repeat == Repeat::None {
                    String::new()
                } else {
                    format!(" ↻ {}", t.repeat)
                };
                println!(
                    "{}  {:7} {:7} {:12} {}{}",
                    &t.id.to_string()[..8],
                    t.status,
                    t.priority,
                    t.project,
                    t.title,
                    recur
                );
            }
        }
        Cmd::Complete { id, force } => {
            let _lock = store::StoreLock::acquire()?;
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
            candidates.sort_by_key(|t| next_key(t, now));
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
            GitCmd::Status => match git::discover() {
                Some(info) => {
                    println!("repo: {}", info.root.display());
                    println!("git branch: {}", info.branch);
                    println!("head: {}", info.commit.as_deref().unwrap_or("-"));
                    println!("remote: {}", info.remote.as_deref().unwrap_or("-"));
                    println!("tasklynx branch: {}", data.current_branch);
                    if info.branch != data.current_branch {
                        println!("note: git branch differs from tasklynx branch");
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
            },
            GitCmd::Link { id, commit } => {
                let info =
                    git::discover().ok_or_else(|| anyhow::anyhow!("not in a git repository"))?;
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
                    t.updated_at = chrono::Utc::now();
                    t.id.to_string()
                };
                store::save_with_undo(&data, &previous)?;
                println!("linked {} to {} on {}", &tid[..8], commit, info.branch);
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
        Cmd::Export { format } => {
            if format == "md" {
                print!("{}", store::export_md(&data))
            } else {
                println!("{}", store::export_json(&data)?)
            }
        }
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
