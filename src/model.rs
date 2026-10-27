use chrono::{DateTime, Local, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Status { Todo, Doing, Done }

impl Default for Status { fn default() -> Self { Self::Todo } }
impl std::fmt::Display for Status { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{}", match self { Self::Todo => "todo", Self::Doing => "doing", Self::Done => "done" }) } }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority { Low, Normal, High, Urgent }
impl Default for Priority { fn default() -> Self { Self::Normal } }
impl std::fmt::Display for Priority { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{}", match self { Self::Low => "low", Self::Normal => "normal", Self::High => "high", Self::Urgent => "urgent" }) } }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub title: String,
    #[serde(default)] pub description: String,
    #[serde(default)] pub project: String,
    #[serde(default)] pub tags: Vec<String>,
    #[serde(default)] pub status: Status,
    #[serde(default)] pub priority: Priority,
    pub due: Option<DateTime<Utc>>,
    #[serde(default)] pub branch: String,
    pub commit: Option<String>,
    pub parent: Option<Uuid>,
    #[serde(default)] pub estimate_minutes: Option<u32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl Task {
    pub fn new(title: String, project: String, branch: String, priority: Priority, tags: Vec<String>, due: Option<DateTime<Utc>>) -> Self {
        let now = Utc::now();
        Self { id: Uuid::new_v4(), title, description: String::new(), project, tags, status: Status::Todo, priority, due, branch, commit: None, parent: None, estimate_minutes: None, created_at: now, updated_at: now, completed_at: None }
    }
    pub fn complete(&mut self) { self.status = Status::Done; self.completed_at = Some(Utc::now()); self.updated_at = Utc::now(); }
    pub fn overdue(&self) -> bool { self.due.map(|d| d < Utc::now() && self.status != Status::Done).unwrap_or(false) }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectData {
    pub version: u32,
    #[serde(default)] pub current_branch: String,
    #[serde(default)] pub branches: Vec<String>,
    #[serde(default)] pub tasks: Vec<Task>,
}
impl Default for ProjectData { fn default() -> Self { Self { version: 1, current_branch: "main".into(), branches: vec!["main".into()], tasks: vec![] } } }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub accent: String,
    pub background: String,
    pub panel: String,
    pub foreground: String,
    pub muted: String,
    pub font_family: String,
    pub default_project: String,
}
impl Default for AppConfig { fn default() -> Self { Self { host: "127.0.0.1".into(), port: 8787, accent: "#ff453a".into(), background: "#070809".into(), panel: "#101216".into(), foreground: "#f5f5f5".into(), muted: "#8f949e".into(), font_family: "ui-monospace, SFMono-Regular, Menlo, Consolas, monospace".into(), default_project: "default".into() } } }

pub fn local_due_date(s: &str) -> anyhow::Result<DateTime<Utc>> {
    let d = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")?;
    Ok(DateTime::<Local>::from_local(d.and_hms_opt(23,59,59).unwrap(), *Local::now().offset()).with_timezone(&Utc))
}
