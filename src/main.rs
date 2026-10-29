mod config;
mod model;
mod store;
mod tui;
mod web;

use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
use model::{local_due_date, Priority, Task};
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
    },
    Duplicate {
        id: String,
    },
    Search {
        query: String,
    },
    Today,
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
        } => {
            let _lock = store::StoreLock::acquire()?;
            let due = due.map(|d| local_due_date(&d)).transpose()?;
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
                println!(
                    "{}  {:7} {:7} {:12} {}",
                    &t.id.to_string()[..8],
                    t.status,
                    t.priority,
                    t.project,
                    t.title
                );
            }
        }
        Cmd::Complete { id } => {
            let _lock = store::StoreLock::acquire()?;
            let tid = store::find_mut(&mut data, &id)?.id.to_string();
            {
                let t = store::find_mut(&mut data, &id)?;
                t.complete();
            }
            store::save(&data)?;
            println!("done {}", &tid[..8]);
        }
        Cmd::Reopen { id } => {
            let _lock = store::StoreLock::acquire()?;
            let tid = store::find_mut(&mut data, &id)?.id.to_string();
            {
                let t = store::find_mut(&mut data, &id)?;
                t.status = model::Status::Todo;
                t.completed_at = None;
                t.updated_at = chrono::Utc::now();
            }
            store::save(&data)?;
            println!("reopened {}", &tid[..8]);
        }
        Cmd::Doing { id } => {
            let _lock = store::StoreLock::acquire()?;
            let tid = store::find_mut(&mut data, &id)?.id.to_string();
            {
                let t = store::find_mut(&mut data, &id)?;
                t.status = model::Status::Doing;
                t.updated_at = chrono::Utc::now();
            }
            store::save(&data)?;
            println!("doing {}", &tid[..8]);
        }
        Cmd::Delete { id } => {
            let _lock = store::StoreLock::acquire()?;
            let target_id = store::find(&data, &id)?.id;
            data.tasks.retain(|t| t.id != target_id);
            store::save(&data)?;
            println!("deleted");
        }
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
        } => {
            let _lock = store::StoreLock::acquire()?;
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
            store::save(&data)?;
            println!("edited {}", &tid[..8]);
        }
        Cmd::Duplicate { id } => {
            let _lock = store::StoreLock::acquire()?;
            let t = store::find(&data, &id)?.clone();
            let mut n = t.clone();
            n.id = uuid::Uuid::new_v4();
            n.title = format!("Copy: {}", n.title);
            n.status = model::Status::Todo;
            n.completed_at = None;
            n.created_at = chrono::Utc::now();
            n.updated_at = n.created_at;
            data.tasks.push(n.clone());
            let nid = n.id.to_string();
            store::save(&data)?;
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
                    data.branches.push(name.clone());
                    store::save(&data)?;
                }
                println!("branch {}", name)
            }
            BranchCmd::Switch { name } => {
                let _lock = store::StoreLock::acquire()?;
                if !data.branches.contains(&name) {
                    return Err(anyhow::anyhow!("branch not found"));
                }
                data.current_branch = name;
                store::save(&data)?
            }
            BranchCmd::Delete { name } => {
                let _lock = store::StoreLock::acquire()?;
                if name == data.current_branch {
                    return Err(anyhow::anyhow!("cannot delete current branch"));
                }
                if data.tasks.iter().any(|x| x.branch == name) {
                    return Err(anyhow::anyhow!("branch is not empty"));
                }
                data.branches.retain(|x| x != &name);
                store::save(&data)?
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
            let (added, updated) = if is_md {
                store::import_md(&mut data, &content, replace)?
            } else {
                store::import_json(&mut data, &content, replace)?
            };
            store::save(&data)?;
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
