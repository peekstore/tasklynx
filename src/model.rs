use chrono::{DateTime, Datelike, Days, Local, NaiveDate, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum Status {
    #[default]
    Todo,
    Doing,
    Done,
}
impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Todo => "todo",
                Self::Doing => "doing",
                Self::Done => "done",
            }
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum Priority {
    Low,
    #[default]
    Normal,
    High,
    Urgent,
}
impl std::fmt::Display for Priority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Low => "low",
                Self::Normal => "normal",
                Self::High => "high",
                Self::Urgent => "urgent",
            }
        )
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum Repeat {
    #[default]
    None,
    Daily,
    Weekly,
    Monthly,
    Yearly,
}
impl std::fmt::Display for Repeat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::None => "none",
                Self::Daily => "daily",
                Self::Weekly => "weekly",
                Self::Monthly => "monthly",
                Self::Yearly => "yearly",
            }
        )
    }
}
impl Repeat {
    #[allow(dead_code)]
    pub fn next_after(self, base: DateTime<Utc>) -> Option<DateTime<Utc>> {
        let local = base.with_timezone(&Local);
        let date = local.date_naive();
        let next_date = match self {
            Self::None => return None,
            Self::Daily => date.checked_add_days(Days::new(1))?,
            Self::Weekly => date.checked_add_days(Days::new(7))?,
            Self::Monthly => add_months_clamped(date, 1),
            Self::Yearly => add_months_clamped(date, 12),
        };
        let naive = next_date.and_time(local.time());
        Some(match Local.from_local_datetime(&naive) {
            chrono::LocalResult::Single(dt) => dt.with_timezone(&Utc),
            chrono::LocalResult::Ambiguous(dt, _) => dt.with_timezone(&Utc),
            chrono::LocalResult::None => naive.and_utc(),
        })
    }
}

#[allow(dead_code)]
fn add_months_clamped(date: NaiveDate, months: u32) -> NaiveDate {
    let total = date.month0() + months;
    let year = date.year() + (total / 12) as i32;
    let month = total % 12 + 1;
    let day = date.day().min(days_in_month(year, month));
    NaiveDate::from_ymd_opt(year, month, day).expect("clamped date is valid")
}

#[allow(dead_code)]
fn days_in_month(year: i32, month: u32) -> u32 {
    let first = NaiveDate::from_ymd_opt(year, month, 1).expect("valid month");
    let next = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    }
    .expect("valid month");
    (next - first).num_days() as u32
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub project: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub status: Status,
    #[serde(default)]
    pub priority: Priority,
    pub due: Option<DateTime<Utc>>,
    #[serde(default)]
    pub repeat: Repeat,
    #[serde(default)]
    pub repeat_until: Option<DateTime<Utc>>,
    #[serde(default)]
    pub series_id: Option<Uuid>,
    #[serde(default)]
    pub branch: String,
    pub commit: Option<String>,
    pub parent: Option<Uuid>,
    #[serde(default)]
    pub estimate_minutes: Option<u32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl Task {
    pub fn new(
        title: String,
        project: String,
        branch: String,
        priority: Priority,
        tags: Vec<String>,
        due: Option<DateTime<Utc>>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            title,
            description: String::new(),
            project,
            tags,
            status: Status::Todo,
            priority,
            due,
            repeat: Repeat::None,
            repeat_until: None,
            series_id: None,
            branch,
            commit: None,
            parent: None,
            estimate_minutes: None,
            created_at: now,
            updated_at: now,
            completed_at: None,
        }
    }
    pub fn complete(&mut self) {
        self.status = Status::Done;
        self.completed_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }
    #[allow(dead_code)]
    pub fn next_due_date(&self) -> Option<DateTime<Utc>> {
        let base = self
            .due
            .unwrap_or_else(|| local_end_of_day(Local::now().date_naive()));
        self.repeat.next_after(base)
    }
    #[allow(dead_code)]
    pub fn next_occurrence(&self) -> Option<Task> {
        let next_due = self.next_due_date()?;
        if self.repeat_until.is_some_and(|until| next_due > until) {
            return None;
        }
        let now = Utc::now();
        let mut next = self.clone();
        next.id = Uuid::new_v4();
        next.status = Status::Todo;
        next.due = Some(next_due);
        next.series_id = Some(self.series_id.unwrap_or(self.id));
        next.created_at = now;
        next.updated_at = now;
        next.completed_at = None;
        Some(next)
    }
    pub fn overdue(&self) -> bool {
        self.due
            .map(|d| d < Utc::now() && self.status != Status::Done)
            .unwrap_or(false)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectData {
    pub version: u32,
    #[serde(default)]
    pub current_branch: String,
    #[serde(default)]
    pub branches: Vec<String>,
    #[serde(default)]
    pub tasks: Vec<Task>,
}
impl Default for ProjectData {
    fn default() -> Self {
        Self {
            version: 1,
            current_branch: "main".into(),
            branches: vec!["main".into()],
            tasks: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
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
impl Default for AppConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".into(),
            port: 8787,
            accent: "#ff453a".into(),
            background: "#070809".into(),
            panel: "#101216".into(),
            foreground: "#f5f5f5".into(),
            muted: "#8f949e".into(),
            font_family: "ui-monospace, SFMono-Regular, Menlo, Consolas, monospace".into(),
            default_project: "default".into(),
        }
    }
}

pub fn local_due_date(s: &str) -> anyhow::Result<DateTime<Utc>> {
    let d = NaiveDate::parse_from_str(s, "%Y-%m-%d")?;
    Ok(local_end_of_day(d))
}

pub fn local_end_of_day(d: NaiveDate) -> DateTime<Utc> {
    let naive = d.and_hms_opt(23, 59, 59).unwrap();
    match Local.from_local_datetime(&naive) {
        chrono::LocalResult::Single(local) => local.with_timezone(&Utc),
        chrono::LocalResult::Ambiguous(a, _) => a.with_timezone(&Utc),
        chrono::LocalResult::None => naive.and_utc(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn task() -> Task {
        Task::new(
            "Test".into(),
            "default".into(),
            "main".into(),
            Priority::Normal,
            vec![],
            None,
        )
    }

    #[test]
    fn parses_iso_due_date() {
        assert!(local_due_date("2026-12-31").is_ok());
    }

    #[test]
    fn rejects_invalid_due_date() {
        assert!(local_due_date("31-12-2026").is_err());
        assert!(local_due_date("tomorrow").is_err());
    }

    #[test]
    fn overdue_task_is_detected() {
        let mut t = task();
        t.due = Some(Utc::now() - chrono::Duration::days(1));
        assert!(t.overdue());
    }

    #[test]
    fn completed_task_is_not_overdue() {
        let mut t = task();
        t.due = Some(Utc::now() - chrono::Duration::days(1));
        t.complete();
        assert!(!t.overdue());
        assert!(t.completed_at.is_some());
    }

    fn due_on(date: &str) -> DateTime<Utc> {
        local_due_date(date).unwrap()
    }

    fn local_date(dt: DateTime<Utc>) -> NaiveDate {
        dt.with_timezone(&Local).date_naive()
    }

    #[test]
    fn daily_repeat_advances_one_day() {
        let mut t = task();
        t.repeat = Repeat::Daily;
        t.due = Some(due_on("2026-12-31"));

        let next = t.next_occurrence().unwrap();
        assert_eq!(
            local_date(next.due.unwrap()),
            due_on("2027-01-01").with_timezone(&Local).date_naive()
        );
    }

    #[test]
    fn weekly_repeat_advances_seven_days() {
        let mut t = task();
        t.repeat = Repeat::Weekly;
        t.due = Some(due_on("2026-12-31"));

        let next = t.next_occurrence().unwrap();
        assert_eq!(
            local_date(next.due.unwrap()),
            due_on("2027-01-07").with_timezone(&Local).date_naive()
        );
    }

    #[test]
    fn monthly_repeat_clamps_to_month_end() {
        let mut t = task();
        t.repeat = Repeat::Monthly;
        t.due = Some(due_on("2026-01-31"));

        let next = t.next_occurrence().unwrap();
        assert_eq!(
            local_date(next.due.unwrap()),
            NaiveDate::from_ymd_opt(2026, 2, 28).unwrap()
        );
    }

    #[test]
    fn monthly_repeat_handles_leap_year() {
        let mut t = task();
        t.repeat = Repeat::Monthly;
        t.due = Some(due_on("2028-01-31"));

        let next = t.next_occurrence().unwrap();
        assert_eq!(
            local_date(next.due.unwrap()),
            NaiveDate::from_ymd_opt(2028, 2, 29).unwrap()
        );
    }

    #[test]
    fn yearly_repeat_handles_leap_day() {
        let mut t = task();
        t.repeat = Repeat::Yearly;
        t.due = Some(due_on("2028-02-29"));

        let next = t.next_occurrence().unwrap();
        assert_eq!(
            local_date(next.due.unwrap()),
            NaiveDate::from_ymd_opt(2029, 2, 28).unwrap()
        );
    }

    #[test]
    fn non_repeating_task_has_no_next_occurrence() {
        let t = task();
        assert!(t.next_occurrence().is_none());
    }

    #[test]
    fn repeat_until_stops_generation() {
        let mut t = task();
        t.repeat = Repeat::Daily;
        t.due = Some(due_on("2026-01-01"));
        t.repeat_until = Some(due_on("2026-01-01"));
        assert!(t.next_occurrence().is_none());

        t.repeat_until = Some(due_on("2026-01-02"));
        assert!(t.next_occurrence().is_some());
    }

    #[test]
    fn next_occurrence_preserves_task_and_links_series() {
        let mut t = task();
        t.repeat = Repeat::Weekly;
        t.due = Some(due_on("2026-01-01"));
        t.project = "api".into();
        t.tags = vec!["backend".into()];

        let next = t.next_occurrence().unwrap();
        assert_ne!(next.id, t.id);
        assert_eq!(next.series_id, Some(t.id));
        assert_eq!(next.title, t.title);
        assert_eq!(next.project, "api");
        assert_eq!(next.tags, vec!["backend"]);
        assert_eq!(next.status, Status::Todo);
        assert!(next.completed_at.is_none());
    }

    #[test]
    fn legacy_task_json_defaults_recurrence() {
        let json = r#"{
            "id": "aaaaaaaa-0000-0000-0000-000000000001",
            "title": "Legacy task",
            "due": null,
            "created_at": "2026-01-01T00:00:00Z",
            "updated_at": "2026-01-01T00:00:00Z"
        }"#;
        let t: Task = serde_json::from_str(json).unwrap();
        assert_eq!(t.repeat, Repeat::None);
        assert_eq!(t.repeat_until, None);
        assert_eq!(t.series_id, None);
    }

    #[test]
    fn recurring_task_without_due_starts_from_today() {
        let before = Local::now().date_naive();
        let mut t = task();
        t.repeat = Repeat::Daily;

        let next = t.next_due_date().unwrap();
        let after = Local::now().date_naive();
        assert!(
            local_date(next) == before + Days::new(1) || local_date(next) == after + Days::new(1)
        );
    }
}
