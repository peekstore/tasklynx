mod config;
mod model;
mod store;
mod tui;
mod web;
use anyhow::Result;
use clap::{Parser, Subcommand};
use model::{local_due_date, Priority, Task};
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
        #[arg(short, long, default_value = "normal")]
        priority: String,
        #[arg(long)]
        due: Option<String>,
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
    #[command(name = "self")]
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
    Uninstall,
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
        } => {
            let due = due.map(|d| local_due_date(&d)).transpose()?;
            let t = Task::new(
                title,
                project,
                data.current_branch.clone(),
                priority(&pri),
                tag,
                due,
            );
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
            let tid = store::find_mut(&mut data, &id)?.id.to_string();
            {
                let t = store::find_mut(&mut data, &id)?;
                t.complete();
            }
            store::save(&data)?;
            println!("done {}", &tid[..8]);
        }
        Cmd::Reopen { id } => {
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
            let before = data.tasks.len();
            data.tasks.retain(|t| !t.id.to_string().starts_with(&id));
            if data.tasks.len() == before {
                return Err(anyhow::anyhow!("task not found: {id}"));
            }
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
            add_tag,
            remove_tag,
        } => {
            let tid = store::find_mut(&mut data, &id)?.id.to_string();
            {
                let t = store::find_mut(&mut data, &id)?;
                if let Some(v) = title {
                    t.title = v
                }
                if let Some(v) = description {
                    t.description = v
                }
                if let Some(v) = pri {
                    t.priority = priority(&v)
                }
                if let Some(v) = due {
                    t.due = Some(local_due_date(&v)?)
                }
                if let Some(v) = project {
                    t.project = v
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
            let t = store::find(&data, &id)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("task not found"))?;
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
                if !data.branches.contains(&name) {
                    data.branches.push(name.clone());
                    store::save(&data)?;
                }
                println!("branch {}", name)
            }
            BranchCmd::Switch { name } => {
                if !data.branches.contains(&name) {
                    return Err(anyhow::anyhow!("branch not found"));
                }
                data.current_branch = name;
                store::save(&data)?
            }
            BranchCmd::Delete { name } => {
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
        Cmd::Web { host, port } => {
            let c = config::load()?;
            web::serve(host.unwrap_or(c.host), port.unwrap_or(c.port), data).await?
        }
        Cmd::Board => tui::run(&data)?,
        Cmd::Config { action } => match action {
            ConfigCmd::Init => {
                config::save(&config::load()?)?;
                println!("{}", config::path().display())
            }
            ConfigCmd::Show => println!("{}", toml::to_string_pretty(&config::load()?)?),
        },
        Cmd::SelfCmd { action } => match action {
            SelfSubCmd::Update => {
                println!("Checking for updates...");
                let _ = std::process::Command::new("sh")
                    .arg("-c")
                    .arg("curl -sL https://raw.githubusercontent.com/peekstore/tasklynx/main/install.sh | bash")
                    .status();
            }
            SelfSubCmd::Uninstall => {
                let exe = std::env::current_exe()?;
                println!("Removing {}", exe.display());
                println!("Note: config/data in config and data dirs remain.");
                let _ = std::fs::remove_file(exe);
            }
        },
    }
    Ok(())
}
