use crate::model::{local_due_date, next_key, AiConfig, Priority, ProjectData, Status, Task};
use anyhow::{anyhow, bail, Context, Result};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

const MAX_TITLE_CHARS: usize = 200;
const MAX_DESCRIPTION_CHARS: usize = 4000;
const MAX_PROJECT_CHARS: usize = 100;
const MAX_TAG_CHARS: usize = 50;
const MAX_TAGS: usize = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderKind {
    None,
    Ollama,
    OpenAiCompatible,
}

impl AiConfig {
    pub fn provider_kind(&self) -> Result<ProviderKind> {
        match self.provider.trim().to_ascii_lowercase().as_str() {
            "none" | "off" => Ok(ProviderKind::None),
            "ollama" => Ok(ProviderKind::Ollama),
            "openai-compatible" | "openai" => Ok(ProviderKind::OpenAiCompatible),
            other => {
                bail!("unknown ai.provider '{other}'; expected none, ollama, or openai-compatible")
            }
        }
    }
}

pub trait Provider: Send + Sync {
    fn name(&self) -> &'static str;
    fn generate(&self, prompt: &str) -> Result<String>;
    fn doctor(&self) -> Result<Vec<String>>;
}

struct OfflineProvider;

impl Provider for OfflineProvider {
    fn name(&self) -> &'static str {
        "none"
    }

    fn generate(&self, _prompt: &str) -> Result<String> {
        bail!(
            "AI provider is not configured; set ai.provider to \"ollama\" or \"openai-compatible\" in config.toml"
        )
    }

    fn doctor(&self) -> Result<Vec<String>> {
        Ok(vec![
            "No AI provider configured; AI commands stay offline.".into(),
            "Set ai.provider in config.toml to enable tx ai plan and tx ai breakdown.".into(),
        ])
    }
}

pub fn provider(config: &AiConfig) -> Result<Box<dyn Provider>> {
    let kind = config.provider_kind()?;
    match kind {
        ProviderKind::None => Ok(Box::new(OfflineProvider)),
        _ => network_provider(config, kind),
    }
}

#[cfg(feature = "ai")]
fn network_provider(config: &AiConfig, kind: ProviderKind) -> Result<Box<dyn Provider>> {
    Ok(Box::new(HttpProvider::new(config, kind)?))
}

#[cfg(not(feature = "ai"))]
fn network_provider(_config: &AiConfig, _kind: ProviderKind) -> Result<Box<dyn Provider>> {
    bail!(
        "AI networking is disabled in this build; rebuild with `cargo build --features ai` to enable ollama or openai-compatible providers"
    )
}

#[cfg(feature = "ai")]
struct HttpProvider {
    kind: ProviderKind,
    endpoint: String,
    model: String,
    api_key: Option<String>,
}

#[cfg(feature = "ai")]
impl HttpProvider {
    fn new(config: &AiConfig, kind: ProviderKind) -> Result<Self> {
        let endpoint = config.endpoint.trim().trim_end_matches('/').to_string();
        if endpoint.is_empty() {
            bail!("ai.endpoint must not be empty");
        }
        let model = config.model.trim().to_string();
        if model.is_empty() {
            bail!("ai.model must not be empty");
        }
        let api_key = std::env::var("TASKLYNX_AI_API_KEY")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        Ok(Self {
            kind,
            endpoint,
            model,
            api_key,
        })
    }

    fn url(&self, path: &str) -> String {
        format!("{}{path}", self.endpoint)
    }

    fn request(
        &self,
        method: &str,
        url: &str,
        body: Option<serde_json::Value>,
    ) -> Result<serde_json::Value> {
        let mut request = match method {
            "GET" => ureq::get(url),
            "POST" => ureq::post(url),
            _ => bail!("unsupported HTTP method {method}"),
        }
        .timeout(std::time::Duration::from_secs(60));
        if let Some(key) = &self.api_key {
            request = request.set("Authorization", &format!("Bearer {key}"));
        }
        let result = match body {
            Some(body) => request
                .set("Content-Type", "application/json")
                .send_string(&serde_json::to_string(&body).context("failed to encode AI request")?),
            None => request.call(),
        };
        match result {
            Ok(response) => {
                let text = response
                    .into_string()
                    .context("failed to read AI endpoint response")?;
                serde_json::from_str(&text).context("AI endpoint returned invalid JSON")
            }
            Err(ureq::Error::Status(code, response)) => {
                let body = response.into_string().unwrap_or_default();
                let hint = if code == 401 {
                    " (check TASKLYNX_AI_API_KEY if authentication is required)"
                } else {
                    ""
                };
                bail!(
                    "AI endpoint returned HTTP {code}{hint}: {}",
                    short_text(body.trim(), 300)
                )
            }
            Err(error) => bail!("AI endpoint request failed: {error}"),
        }
    }
}

#[cfg(feature = "ai")]
impl Provider for HttpProvider {
    fn name(&self) -> &'static str {
        match self.kind {
            ProviderKind::Ollama => "ollama",
            ProviderKind::OpenAiCompatible => "openai-compatible",
            ProviderKind::None => "none",
        }
    }

    fn generate(&self, prompt: &str) -> Result<String> {
        match self.kind {
            ProviderKind::Ollama => {
                let body = serde_json::json!({
                    "model": self.model,
                    "prompt": prompt,
                    "stream": false,
                    "format": "json"
                });
                let response = self.request("POST", &self.url("/api/generate"), Some(body))?;
                response
                    .get("response")
                    .and_then(serde_json::Value::as_str)
                    .map(str::to_string)
                    .context("ollama response did not contain a response string")
            }
            ProviderKind::OpenAiCompatible => {
                let body = serde_json::json!({
                    "model": self.model,
                    "messages": [
                        {
                            "role": "system",
                            "content": "You are a task planner. Return only valid JSON and no markdown."
                        },
                        {
                            "role": "user",
                            "content": prompt
                        }
                    ],
                    "temperature": 0.2
                });
                let response = self.request("POST", &self.url("/chat/completions"), Some(body))?;
                response
                    .pointer("/choices/0/message/content")
                    .and_then(serde_json::Value::as_str)
                    .map(str::to_string)
                    .context("OpenAI-compatible response did not contain message content")
            }
            ProviderKind::None => unreachable!("offline provider is handled separately"),
        }
    }

    fn doctor(&self) -> Result<Vec<String>> {
        match self.kind {
            ProviderKind::Ollama => {
                let response = self.request("GET", &self.url("/api/tags"), None)?;
                let names: Vec<String> = response
                    .get("models")
                    .and_then(serde_json::Value::as_array)
                    .map(|models| {
                        models
                            .iter()
                            .filter_map(|model| model.get("name"))
                            .filter_map(serde_json::Value::as_str)
                            .map(str::to_string)
                            .collect()
                    })
                    .unwrap_or_default();
                if names.is_empty() {
                    bail!("ollama is reachable but has no models installed");
                }
                if !model_available(&names, &self.model) {
                    bail!(
                        "model '{}' is not available; installed models: {}",
                        self.model,
                        sample_models(&names)
                    );
                }
                Ok(vec![
                    format!("reachable: {}", self.endpoint),
                    format!("model available: {}", self.model),
                ])
            }
            ProviderKind::OpenAiCompatible => {
                let response = self.request("GET", &self.url("/models"), None)?;
                let names: Vec<String> = response
                    .get("data")
                    .and_then(serde_json::Value::as_array)
                    .map(|models| {
                        models
                            .iter()
                            .filter_map(|model| model.get("id"))
                            .filter_map(serde_json::Value::as_str)
                            .map(str::to_string)
                            .collect()
                    })
                    .unwrap_or_default();
                if names.is_empty() {
                    bail!("endpoint is reachable but did not list any models");
                }
                if !openai_model_available(&names, &self.model) {
                    bail!(
                        "model '{}' is not available; listed models: {}",
                        self.model,
                        sample_models(&names)
                    );
                }
                Ok(vec![
                    format!("reachable: {}", self.endpoint),
                    format!("model available: {}", self.model),
                ])
            }
            ProviderKind::None => unreachable!("offline provider is handled separately"),
        }
    }
}

#[cfg(feature = "ai")]
fn model_available(names: &[String], model: &str) -> bool {
    names
        .iter()
        .any(|name| name == model || name.starts_with(&format!("{model}:")))
}

#[cfg(feature = "ai")]
fn sample_models(names: &[String]) -> String {
    let mut sample = names
        .iter()
        .take(10)
        .cloned()
        .collect::<Vec<_>>()
        .join(", ");
    if names.len() > 10 {
        sample.push_str(", ...");
    }
    sample
}

#[cfg(feature = "ai")]
fn openai_model_available(names: &[String], model: &str) -> bool {
    names
        .iter()
        .any(|name| name == model || name.strip_prefix("models/") == Some(model))
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DraftTask {
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub project: Option<String>,
    #[serde(default)]
    pub priority: Option<String>,
    #[serde(default)]
    pub due: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub depends_on: Vec<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DraftPlan {
    #[serde(default)]
    pub project: Option<String>,
    #[serde(default)]
    pub tasks: Vec<DraftTask>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DraftTriagePick {
    pub id: String,
    #[serde(default)]
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TriageResponse {
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub picks: Vec<DraftTriagePick>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TriagePick {
    pub task_id: uuid::Uuid,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReportSection {
    pub title: String,
    #[serde(default)]
    pub items: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Report {
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub sections: Vec<ReportSection>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AiAnswer {
    #[serde(default)]
    pub answer: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportKind {
    Standup,
    Review,
}

impl ReportKind {
    fn label(self) -> &'static str {
        match self {
            Self::Standup => "standup",
            Self::Review => "review",
        }
    }
}

pub struct TriageRequest<'a> {
    pub tasks: &'a [Task],
    pub branch: &'a str,
    pub project: Option<&'a str>,
    pub limit: usize,
    pub include_blocked: bool,
    pub include_descriptions: bool,
}

pub struct ReportRequest<'a> {
    pub kind: ReportKind,
    pub tasks: &'a [Task],
    pub branch: &'a str,
    pub project: Option<&'a str>,
    pub days: u32,
    pub limit: usize,
    pub include_descriptions: bool,
}

pub struct AskRequest<'a> {
    pub question: &'a str,
    pub tasks: &'a [Task],
    pub branch: &'a str,
    pub project: Option<&'a str>,
    pub limit: usize,
    pub include_descriptions: bool,
}

pub struct PlanRequest<'a> {
    pub goal: &'a str,
    pub tasks: &'a [Task],
    pub branch: &'a str,
    pub project: Option<&'a str>,
    pub limit: usize,
    pub include_descriptions: bool,
}

pub fn request_plan(
    provider: &dyn Provider,
    config: &AiConfig,
    request: PlanRequest<'_>,
) -> Result<DraftPlan> {
    let prompt = plan_prompt(
        config,
        request.goal,
        request.tasks,
        request.branch,
        request.project,
        request.limit,
        request.include_descriptions,
    );
    let raw = generate(provider, &prompt)?;
    parse_response(&raw, request.limit)
}

pub fn request_breakdown(
    provider: &dyn Provider,
    config: &AiConfig,
    parent: &Task,
    children: &[Task],
    limit: usize,
    include_descriptions: bool,
) -> Result<DraftPlan> {
    let prompt = breakdown_prompt(config, parent, children, limit, include_descriptions);
    let raw = generate(provider, &prompt)?;
    parse_response(&raw, limit)
}

fn generate(provider: &dyn Provider, prompt: &str) -> Result<String> {
    let started = std::time::Instant::now();
    let result = provider.generate(prompt);
    if std::env::var_os("TASKLYNX_AI_LOG").is_some() {
        match &result {
            Ok(raw) => eprintln!(
                "ai: provider={} prompt={}B response={}B latency={}ms",
                provider.name(),
                prompt.len(),
                raw.len(),
                started.elapsed().as_millis()
            ),
            Err(_) => eprintln!(
                "ai: provider={} prompt={}B error latency={}ms",
                provider.name(),
                prompt.len(),
                started.elapsed().as_millis()
            ),
        }
    }
    result
}

pub fn deterministic_triage(request: TriageRequest<'_>) -> Vec<TriagePick> {
    let now = Utc::now();
    let mut candidates = triage_candidates(
        request.tasks,
        request.branch,
        request.project,
        request.include_blocked,
    );
    candidates.truncate(request.limit);
    candidates
        .into_iter()
        .map(|task| TriagePick {
            task_id: task.id,
            reason: triage_reason(task, request.tasks, now),
        })
        .collect()
}

fn triage_candidates<'a>(
    tasks: &'a [Task],
    branch: &str,
    project: Option<&str>,
    include_blocked: bool,
) -> Vec<&'a Task> {
    let now = Utc::now();
    let mut candidates: Vec<&Task> = tasks
        .iter()
        .filter(|task| task.status != Status::Done)
        .filter(|task| task.branch == branch)
        .filter(|task| project.map_or(true, |project| task.project == project))
        .filter(|task| include_blocked || !task.is_blocked(tasks))
        .collect();
    candidates.sort_by_key(|task| next_key(task, now));
    candidates
}

pub fn triage_candidate_count(
    tasks: &[Task],
    branch: &str,
    project: Option<&str>,
    include_blocked: bool,
) -> usize {
    triage_candidates(tasks, branch, project, include_blocked).len()
}

fn triage_reason(task: &Task, tasks: &[Task], now: chrono::DateTime<Utc>) -> String {
    let mut reasons = Vec::new();
    if let Some(due) = task.due {
        let due_date = due.with_timezone(&chrono::Local).date_naive();
        if due < now {
            reasons.push(format!("overdue since {due_date}"));
        } else if due_date == now.with_timezone(&chrono::Local).date_naive() {
            reasons.push("due today".into());
        } else {
            reasons.push(format!("due {due_date}"));
        }
    }
    if task.status == Status::Doing {
        reasons.push("already in progress".into());
    }
    match task.priority {
        Priority::Urgent => reasons.push("urgent priority".into()),
        Priority::High => reasons.push("high priority".into()),
        _ => {}
    }
    let blockers = task
        .blocked_by
        .iter()
        .filter(|id| {
            tasks
                .iter()
                .any(|candidate| candidate.id == **id && candidate.status != Status::Done)
        })
        .count();
    if blockers > 0 {
        reasons.push(format!("blocked by {blockers} open task(s)"));
    }
    if reasons.is_empty() {
        reasons.push("next by priority and due date".into());
    }
    reasons.join("; ")
}

pub fn request_triage(
    provider: &dyn Provider,
    config: &AiConfig,
    request: TriageRequest<'_>,
) -> Result<TriageResponse> {
    let prompt = triage_prompt(config, &request);
    let raw = generate(provider, &prompt)?;
    parse_triage_response(&raw, request.limit)
}

fn triage_prompt(config: &AiConfig, request: &TriageRequest<'_>) -> String {
    let candidates = triage_candidates(
        request.tasks,
        request.branch,
        request.project,
        request.include_blocked,
    );
    let include_detail = config.include_descriptions && request.include_descriptions;
    let context = if candidates.is_empty() {
        "(none)".to_string()
    } else {
        candidates
            .iter()
            .take(config.max_context_tasks)
            .map(|task| triage_context_line(task, request.tasks, include_detail, config.redact))
            .collect::<Vec<_>>()
            .join("\n")
    };
    let project = request.project.unwrap_or("all");
    format!(
        r#"You are Tasklynx, a local developer task triage assistant.
Return exactly one JSON object with this shape:
{{"summary":"one short sentence","picks":[{{"id":"task id","reason":"short reason"}}]}}

Rules:
- Pick at most {limit} tasks.
- Use only task ids present in the context below.
- Prefer in-progress, overdue, urgent, and high-priority work in that order.
- Keep each reason under 160 characters.
- Never invent tasks or include secrets, tokens, or credentials.

Branch: {branch}
Project filter: {project}

Eligible open tasks:
{context}
"#,
        limit = request.limit,
        branch = request.branch,
    )
}

fn triage_context_line(
    task: &Task,
    tasks: &[Task],
    include_description: bool,
    redact: bool,
) -> String {
    let title = if redact {
        redact_text(&task.title)
    } else {
        task.title.clone()
    };
    let due = task
        .due
        .map(|due| due.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| "none".into());
    let blockers = task
        .blocked_by
        .iter()
        .filter(|id| {
            tasks
                .iter()
                .any(|candidate| candidate.id == **id && candidate.status != Status::Done)
        })
        .count();
    let short_id = task.id.to_string();
    let mut line = format!(
        "- id={} | project={} | priority={} | status={} | due={} | blocked_by={} | title={}",
        &short_id[..8],
        task.project,
        task.priority,
        task.status,
        due,
        blockers,
        title
    );
    if include_description && !task.description.trim().is_empty() {
        let description = if redact {
            redact_text(&task.description)
        } else {
            task.description.clone()
        };
        line.push_str(&format!(
            " | description={}",
            description.replace(['\r', '\n'], " ")
        ));
    }
    line
}

pub fn parse_triage_response(raw: &str, limit: usize) -> Result<TriageResponse> {
    if limit == 0 {
        bail!("triage limit must be at least 1");
    }
    let mut response: TriageResponse = serde_json::from_str(json_object(raw)?)
        .context("AI returned an invalid triage response")?;
    response.summary = response.summary.trim().to_string();
    if response.summary.is_empty() {
        response.summary = "AI triage".into();
    }
    if response.summary.chars().count() > 1000 {
        response.summary = response.summary.chars().take(1000).collect();
    }
    if response.picks.is_empty() {
        bail!("AI returned no triage picks");
    }
    if response.picks.len() > limit {
        bail!(
            "AI returned {} triage picks, exceeding the limit of {limit}",
            response.picks.len()
        );
    }
    let mut seen = HashSet::new();
    let mut picks = Vec::new();
    for pick in response.picks {
        let id = pick.id.trim().to_string();
        if id.is_empty() {
            bail!("AI returned a triage pick with an empty task id");
        }
        if id.chars().count() > 64 {
            bail!("AI returned a triage task id longer than 64 characters");
        }
        if !seen.insert(id.clone()) {
            continue;
        }
        let mut reason = pick.reason.trim().to_string();
        if reason.is_empty() {
            reason = "selected by AI".into();
        }
        if reason.chars().count() > 400 {
            reason = reason.chars().take(400).collect();
        }
        picks.push(DraftTriagePick { id, reason });
    }
    if picks.is_empty() {
        bail!("AI returned no usable triage picks");
    }
    response.picks = picks;
    Ok(response)
}

pub fn resolve_triage(
    response: &TriageResponse,
    tasks: &[Task],
    branch: &str,
    project: Option<&str>,
    include_blocked: bool,
) -> Result<Vec<TriagePick>> {
    let mut seen = HashSet::new();
    let mut picks = Vec::new();
    for pick in &response.picks {
        let id = pick.id.trim().to_ascii_lowercase();
        let task = if let Ok(uuid) = uuid::Uuid::parse_str(&id) {
            tasks.iter().find(|task| task.id == uuid)
        } else {
            let matches: Vec<&Task> = tasks
                .iter()
                .filter(|task| task.id.to_string().starts_with(&id))
                .collect();
            match matches.as_slice() {
                [task] => Some(*task),
                [] => None,
                _ => bail!("AI triage id '{id}' is ambiguous"),
            }
        }
        .ok_or_else(|| anyhow!("AI triage referenced unknown task id '{id}'"))?;
        if task.status == Status::Done
            || task.branch != branch
            || project.is_some_and(|project| task.project != project)
            || (!include_blocked && task.is_blocked(tasks))
        {
            bail!("AI triage referenced task '{id}' that is not eligible");
        }
        if !seen.insert(task.id) {
            continue;
        }
        picks.push(TriagePick {
            task_id: task.id,
            reason: pick.reason.clone(),
        });
    }
    Ok(picks)
}

pub fn request_report(
    provider: &dyn Provider,
    config: &AiConfig,
    request: ReportRequest<'_>,
) -> Result<Report> {
    let prompt = report_prompt(config, &request);
    let raw = generate(provider, &prompt)?;
    parse_report(&raw, request.limit)
}

fn report_prompt(config: &AiConfig, request: &ReportRequest<'_>) -> String {
    let cutoff = Utc::now() - Duration::days(i64::from(request.days));
    let include_detail = config.include_descriptions && request.include_descriptions;
    let mut selected: Vec<&Task> = request
        .tasks
        .iter()
        .filter(|task| task.branch == request.branch)
        .filter(|task| {
            request
                .project
                .map_or(true, |project| task.project == project)
        })
        .filter(|task| match task.status {
            Status::Done => task
                .completed_at
                .is_some_and(|completed| completed >= cutoff),
            _ => true,
        })
        .collect();
    selected.sort_by_key(|task| {
        let status = match task.status {
            Status::Doing => 0,
            Status::Todo if task.is_blocked(request.tasks) => 1,
            Status::Todo => 2,
            Status::Done => 3,
        };
        (status, std::cmp::Reverse(task.updated_at))
    });
    selected.truncate(config.max_context_tasks);
    let context = if selected.is_empty() {
        "(none)".to_string()
    } else {
        selected
            .iter()
            .map(|task| report_context_line(task, request.tasks, include_detail, config.redact))
            .collect::<Vec<_>>()
            .join("\n")
    };
    let project = request.project.unwrap_or("all");
    format!(
        r#"You are Tasklynx, a concise developer {label} assistant.
Return exactly one JSON object with this shape:
{{"summary":"one or two sentences","sections":[{{"title":"Done|Doing|Blocked|Next","items":["short bullet"]}}]}}

Rules:
- Write a {label} from the context below.
- Use at most {limit} bullets across all sections.
- Only state facts present in the context; do not invent work.
- Prefer concrete outcomes over activity.
- Never include secrets, tokens, or credentials.

Branch: {branch}
Project filter: {project}
Window: last {days} day(s)

Tasks:
{context}
"#,
        label = request.kind.label(),
        limit = request.limit,
        branch = request.branch,
        days = request.days,
    )
}

fn report_context_line(
    task: &Task,
    tasks: &[Task],
    include_description: bool,
    redact: bool,
) -> String {
    let title = if redact {
        redact_text(&task.title)
    } else {
        task.title.clone()
    };
    let due = task
        .due
        .map(|due| due.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| "none".into());
    let completed = task
        .completed_at
        .map(|completed| completed.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| "none".into());
    let blocked = if task.is_blocked(tasks) { "yes" } else { "no" };
    let short_id = task.id.to_string();
    let mut line = format!(
        "- id={} | project={} | priority={} | status={} | due={} | completed={} | blocked={} | title={}",
        &short_id[..8],
        task.project,
        task.priority,
        task.status,
        due,
        completed,
        blocked,
        title
    );
    if include_description && !task.description.trim().is_empty() {
        let description = if redact {
            redact_text(&task.description)
        } else {
            task.description.clone()
        };
        line.push_str(&format!(
            " | description={}",
            description.replace(['\r', '\n'], " ")
        ));
    }
    line
}

pub fn parse_report(raw: &str, limit: usize) -> Result<Report> {
    if limit == 0 {
        bail!("report limit must be at least 1");
    }
    let mut report: Report =
        serde_json::from_str(json_object(raw)?).context("AI returned an invalid report")?;
    report.summary = report.summary.trim().to_string();
    if report.summary.is_empty() {
        bail!("AI returned an empty report summary");
    }
    if report.summary.chars().count() > 2000 {
        report.summary = report.summary.chars().take(2000).collect();
    }
    if report.sections.len() > 8 {
        bail!("AI returned too many report sections");
    }
    let mut total_items = 0;
    for section in &mut report.sections {
        section.title = section.title.trim().to_string();
        if section.title.is_empty() {
            bail!("AI returned a report section with an empty title");
        }
        if section.title.chars().count() > 80 {
            section.title = section.title.chars().take(80).collect();
        }
        section.items = std::mem::take(&mut section.items)
            .into_iter()
            .map(|item| item.trim().to_string())
            .filter(|item| !item.is_empty())
            .map(|item| item.chars().take(500).collect())
            .collect();
        total_items += section.items.len();
    }
    if total_items == 0 {
        bail!("AI returned a report without any bullet items");
    }
    if total_items > limit {
        bail!("AI returned {total_items} report items, exceeding the limit of {limit}");
    }
    Ok(report)
}

pub fn request_answer(
    provider: &dyn Provider,
    config: &AiConfig,
    request: AskRequest<'_>,
) -> Result<AiAnswer> {
    let prompt = ask_prompt(config, &request);
    let raw = generate(provider, &prompt)?;
    parse_answer(&raw)
}

fn ask_prompt(config: &AiConfig, request: &AskRequest<'_>) -> String {
    let context = context_text_limited(
        request.tasks,
        request.branch,
        request.project,
        config,
        request.include_descriptions,
        request.limit,
    );
    let project = request.project.unwrap_or("all");
    format!(
        r#"You are Tasklynx, a local developer task assistant.
Answer the user's question using only the task context below.
Return exactly one JSON object with this shape:
{{"answer":"concise answer"}}

Rules:
- If the context does not contain the answer, say so.
- Do not invent tasks, commits, deadlines, or blockers.
- Never include secrets, tokens, or credentials.

Branch: {branch}
Project filter: {project}
Context (up to {limit} open tasks):
{context}

Question: {question}
"#,
        branch = request.branch,
        limit = request.limit,
        question = request.question,
    )
}

pub fn parse_answer(raw: &str) -> Result<AiAnswer> {
    let mut answer: AiAnswer =
        serde_json::from_str(json_object(raw)?).context("AI returned an invalid answer")?;
    answer.answer = answer.answer.trim().to_string();
    if answer.answer.is_empty() {
        bail!("AI returned an empty answer");
    }
    if answer.answer.chars().count() > 4000 {
        answer.answer = answer.answer.chars().take(4000).collect();
    }
    Ok(answer)
}

fn json_object(raw: &str) -> Result<&str> {
    let start = raw
        .find('{')
        .ok_or_else(|| anyhow!("AI response did not contain a JSON object"))?;
    let end = raw
        .rfind('}')
        .ok_or_else(|| anyhow!("AI response did not contain a complete JSON object"))?;
    if end < start {
        bail!("AI response contained malformed JSON");
    }
    Ok(&raw[start..=end])
}

pub fn plan_prompt(
    config: &AiConfig,
    goal: &str,
    tasks: &[Task],
    branch: &str,
    project: Option<&str>,
    limit: usize,
    include_descriptions: bool,
) -> String {
    let context = context_text(tasks, branch, project, config, include_descriptions);
    let project_hint = project
        .map(|project| {
            format!("\nUse project \"{project}\" unless the goal clearly needs another.")
        })
        .unwrap_or_default();
    format!(
        r#"You are Tasklynx, a local developer task planner.
Return exactly one JSON object with this shape:
{{"project":"optional project name","tasks":[{{"title":"short imperative title","description":"optional detail","project":"optional project","priority":"low|normal|high|urgent","due":"YYYY-MM-DD or null","tags":["optional"],"depends_on":[0]}}]}}

Rules:
- Create at most {limit} tasks.
- Titles must be concrete and actionable.
- Use depends_on only for earlier tasks in the same response, using zero-based indexes.
- Prefer a small, focused plan over many speculative tasks.
- Never include secrets, tokens, or credentials in task text.{project_hint}

Existing open tasks on Tasklynx branch "{branch}":
{context}

User goal: {goal}
"#
    )
}

pub fn breakdown_prompt(
    config: &AiConfig,
    parent: &Task,
    children: &[Task],
    limit: usize,
    include_descriptions: bool,
) -> String {
    let include_detail = config.include_descriptions && include_descriptions;
    let parent_context = task_context_line(parent, include_detail, config.redact);
    let existing = if children.is_empty() {
        "(none)".to_string()
    } else {
        children
            .iter()
            .take(config.max_context_tasks)
            .map(|task| task_context_line(task, include_detail, config.redact))
            .collect::<Vec<_>>()
            .join("\n")
    };
    format!(
        r#"You are Tasklynx, a local developer task planner.
Break the parent task into concrete subtasks.
Return exactly one JSON object with this shape:
{{"tasks":[{{"title":"short imperative title","description":"optional detail","priority":"low|normal|high|urgent","due":"YYYY-MM-DD or null","tags":["optional"],"depends_on":[0]}}]}}

Rules:
- Create at most {limit} subtasks.
- Do not repeat existing subtasks.
- The parent's project, branch, and priority are inherited unless a subtask sets its own priority.
- Use depends_on only for earlier subtasks in this response, using zero-based indexes.
- Never include secrets, tokens, or credentials in task text.

Parent task:
{parent_context}

Existing subtasks:
{existing}
"#
    )
}

fn context_text(
    tasks: &[Task],
    branch: &str,
    project: Option<&str>,
    config: &AiConfig,
    include_descriptions: bool,
) -> String {
    context_text_limited(
        tasks,
        branch,
        project,
        config,
        include_descriptions,
        config.max_context_tasks,
    )
}

fn context_text_limited(
    tasks: &[Task],
    branch: &str,
    project: Option<&str>,
    config: &AiConfig,
    include_descriptions: bool,
    limit: usize,
) -> String {
    let include_detail = config.include_descriptions && include_descriptions;
    let mut selected: Vec<&Task> = tasks
        .iter()
        .filter(|task| task.status != Status::Done)
        .filter(|task| task.branch == branch)
        .filter(|task| project.map_or(true, |project| task.project == project))
        .collect();
    selected.sort_by_key(|task| (std::cmp::Reverse(task.priority.clone()), task.created_at));
    selected.truncate(limit.min(config.max_context_tasks));
    if selected.is_empty() {
        return "(none)".into();
    }
    selected
        .into_iter()
        .map(|task| task_context_line(task, include_detail, config.redact))
        .collect::<Vec<_>>()
        .join("\n")
}

fn task_context_line(task: &Task, include_description: bool, redact: bool) -> String {
    let title = if redact {
        redact_text(&task.title)
    } else {
        task.title.clone()
    };
    let due = task
        .due
        .map(|due| due.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| "none".into());
    let mut line = format!(
        "- project={} | priority={} | status={} | due={} | title={}",
        task.project, task.priority, task.status, due, title
    );
    if include_description && !task.description.trim().is_empty() {
        let description = if redact {
            redact_text(&task.description)
        } else {
            task.description.clone()
        };
        line.push_str(&format!(
            " | description={}",
            description.replace(['\r', '\n'], " ")
        ));
    }
    line
}

pub fn parse_response(raw: &str, limit: usize) -> Result<DraftPlan> {
    let start = raw
        .find('{')
        .ok_or_else(|| anyhow!("AI response did not contain a JSON object"))?;
    let end = raw
        .rfind('}')
        .ok_or_else(|| anyhow!("AI response did not contain a complete JSON object"))?;
    if end < start {
        bail!("AI response contained malformed JSON");
    }
    let plan: DraftPlan =
        serde_json::from_str(&raw[start..=end]).context("AI returned an invalid task plan")?;
    normalize_plan(plan, limit)
}

pub fn normalize_plan(mut plan: DraftPlan, limit: usize) -> Result<DraftPlan> {
    if limit == 0 {
        bail!("task limit must be at least 1");
    }
    if plan.tasks.is_empty() {
        bail!("AI returned no tasks");
    }
    if plan.tasks.len() > limit {
        bail!(
            "AI returned {} tasks, exceeding the limit of {limit}",
            plan.tasks.len()
        );
    }
    plan.project = clean_optional(plan.project);
    if let Some(project) = &plan.project {
        if project.chars().count() > MAX_PROJECT_CHARS {
            bail!("AI returned a project name longer than {MAX_PROJECT_CHARS} characters");
        }
    }
    for (index, task) in plan.tasks.iter_mut().enumerate() {
        task.title = task.title.trim().to_string();
        if task.title.is_empty() {
            bail!("AI returned a task with an empty title");
        }
        if task.title.chars().count() > MAX_TITLE_CHARS {
            bail!(
                "AI task {} has a title longer than {MAX_TITLE_CHARS} characters",
                index + 1
            );
        }
        task.description = task
            .description
            .trim()
            .chars()
            .take(MAX_DESCRIPTION_CHARS)
            .collect();
        task.project = clean_optional(task.project.take());
        if let Some(project) = &task.project {
            if project.chars().count() > MAX_PROJECT_CHARS {
                bail!(
                    "AI task {} has a project name longer than {MAX_PROJECT_CHARS} characters",
                    index + 1
                );
            }
        }
        if let Some(priority) = task.priority.take() {
            task.priority = Some(parse_priority(&priority)?.to_string());
        }
        if let Some(due) = task.due.take() {
            let due = due.trim();
            if due.is_empty() {
                task.due = None;
            } else {
                local_due_date(due)
                    .with_context(|| format!("AI task {} has an invalid due date", index + 1))?;
                task.due = Some(due.to_string());
            }
        }
        let mut seen = HashSet::new();
        task.tags = std::mem::take(&mut task.tags)
            .into_iter()
            .map(|tag| tag.trim().to_string())
            .filter(|tag| !tag.is_empty() && seen.insert(tag.clone()))
            .take(MAX_TAGS)
            .collect();
        if let Some(tag) = task
            .tags
            .iter()
            .find(|tag| tag.chars().count() > MAX_TAG_CHARS)
        {
            bail!(
                "AI task {} has a tag longer than {MAX_TAG_CHARS} characters: {tag}",
                index + 1
            );
        }
        task.depends_on.sort_unstable();
        task.depends_on.dedup();
        for dependency in &task.depends_on {
            if *dependency >= index {
                bail!(
                    "AI task {} has depends_on index {dependency}; dependencies must refer to earlier tasks",
                    index + 1
                );
            }
        }
    }
    Ok(plan)
}

pub fn plan_tasks(
    plan: &DraftPlan,
    data: &ProjectData,
    default_project: &str,
) -> Result<Vec<Task>> {
    let plan_project = plan.project.as_deref().unwrap_or(default_project).trim();
    let plan_project = if plan_project.is_empty() {
        "default"
    } else {
        plan_project
    };
    let mut created: Vec<Task> = Vec::new();
    for draft in &plan.tasks {
        let project = draft.project.as_deref().unwrap_or(plan_project);
        let mut task = Task::new(
            draft.title.clone(),
            project.to_string(),
            data.current_branch.clone(),
            draft_priority(draft)?,
            draft.tags.clone(),
            draft_due(draft)?,
        );
        task.description = draft.description.clone();
        for dependency in &draft.depends_on {
            if let Some(blocker) = created.get(*dependency) {
                task.blocked_by.push(blocker.id);
            }
        }
        created.push(task);
    }
    Ok(created)
}

pub fn breakdown_tasks(parent: &Task, plan: &DraftPlan) -> Result<Vec<Task>> {
    let mut created: Vec<Task> = Vec::new();
    for draft in &plan.tasks {
        let priority = match draft.priority.as_deref() {
            Some(priority) => parse_priority(priority)?,
            None => parent.priority.clone(),
        };
        let mut task = Task::new(
            draft.title.clone(),
            parent.project.clone(),
            parent.branch.clone(),
            priority,
            draft.tags.clone(),
            draft_due(draft)?,
        );
        task.description = draft.description.clone();
        task.parent = Some(parent.id);
        task.git_branch = parent.git_branch.clone();
        task.git_remote = parent.git_remote.clone();
        for dependency in &draft.depends_on {
            if let Some(blocker) = created.get(*dependency) {
                task.blocked_by.push(blocker.id);
            }
        }
        created.push(task);
    }
    Ok(created)
}

fn draft_priority(draft: &DraftTask) -> Result<Priority> {
    match draft.priority.as_deref() {
        Some(priority) => parse_priority(priority),
        None => Ok(Priority::Normal),
    }
}

fn draft_due(draft: &DraftTask) -> Result<Option<chrono::DateTime<chrono::Utc>>> {
    draft.due.as_deref().map(local_due_date).transpose()
}

fn parse_priority(value: &str) -> Result<Priority> {
    match value.trim().to_ascii_lowercase().as_str() {
        "low" => Ok(Priority::Low),
        "normal" => Ok(Priority::Normal),
        "high" => Ok(Priority::High),
        "urgent" => Ok(Priority::Urgent),
        other => {
            bail!("AI returned invalid priority '{other}'; expected low, normal, high, or urgent")
        }
    }
}

fn clean_optional(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

#[cfg(feature = "ai")]
fn short_text(value: &str, limit: usize) -> String {
    let mut text: String = value.chars().take(limit).collect();
    if value.chars().count() > limit {
        text.push_str("...");
    }
    text
}

pub fn redact_text(text: &str) -> String {
    let mut redacted = Vec::new();
    let mut in_private_key = false;
    for line in text.lines() {
        let lower = line.to_ascii_lowercase();
        if lower.contains("begin") && lower.contains("private key") {
            in_private_key = true;
            redacted.push("[REDACTED PRIVATE KEY]".to_string());
            continue;
        }
        if in_private_key {
            if lower.contains("end") && lower.contains("private key") {
                in_private_key = false;
            }
            continue;
        }
        redacted.push(redact_line(line));
    }
    redacted.join("\n")
}

fn redact_line(line: &str) -> String {
    const MARKERS: &[&str] = &[
        "api_key",
        "apikey",
        "database_url",
        "connection_string",
        "secret",
        "token",
        "password",
        "passwd",
        "authorization",
    ];
    const PREFIXES: &[&str] = &[
        "ghp_",
        "gho_",
        "ghu_",
        "ghs_",
        "ghr_",
        "github_pat_",
        "glpat-",
        "sk-",
        "xoxb-",
        "xoxp-",
        "AKIA",
        "AIza",
        "eyJ",
        "Bearer",
    ];

    let lower = line.to_ascii_lowercase();
    for marker in MARKERS {
        if let Some(marker_index) = lower.find(marker) {
            if let Some(separator_index) = line.find('=').or_else(|| line.find(':')) {
                if marker_index < separator_index {
                    let separator = &line[separator_index..=separator_index];
                    return format!("{}{} [REDACTED]", &line[..separator_index], separator);
                }
            }
        }
    }

    let mut changed = false;
    let redacted = line
        .split_whitespace()
        .map(|word| {
            let clean = word.trim_matches(|ch: char| {
                !ch.is_alphanumeric() && ch != '_' && ch != '-' && ch != '+'
            });
            if PREFIXES.iter().any(|prefix| clean.starts_with(prefix)) {
                changed = true;
                "[REDACTED]"
            } else {
                word
            }
        })
        .collect::<Vec<_>>();
    if changed {
        redacted.join(" ")
    } else {
        line.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FixtureProvider {
        response: String,
    }

    impl Provider for FixtureProvider {
        fn name(&self) -> &'static str {
            "fixture"
        }

        fn generate(&self, _prompt: &str) -> Result<String> {
            Ok(self.response.clone())
        }

        fn doctor(&self) -> Result<Vec<String>> {
            Ok(vec!["fixture is ready".into()])
        }
    }

    fn task(title: &str) -> Task {
        Task::new(
            title.into(),
            "default".into(),
            "main".into(),
            Priority::Normal,
            vec![],
            None,
        )
    }

    #[test]
    fn recognizes_provider_names() {
        let mut config = AiConfig::default();
        assert_eq!(config.provider_kind().unwrap(), ProviderKind::None);
        config.provider = "ollama".into();
        assert_eq!(config.provider_kind().unwrap(), ProviderKind::Ollama);
        config.provider = "openai-compatible".into();
        assert_eq!(
            config.provider_kind().unwrap(),
            ProviderKind::OpenAiCompatible
        );
        config.provider = "unknown".into();
        assert!(config.provider_kind().is_err());
    }

    #[cfg(feature = "ai")]
    #[test]
    fn builds_provider_urls() {
        let mut config = AiConfig {
            provider: "openai-compatible".into(),
            endpoint: "https://example.com/v1".into(),
            ..AiConfig::default()
        };
        let provider = HttpProvider::new(&config, ProviderKind::OpenAiCompatible).unwrap();
        assert_eq!(
            provider.url("/chat/completions"),
            "https://example.com/v1/chat/completions"
        );
        config.endpoint = "https://generativelanguage.googleapis.com/v1beta/openai".into();
        let provider = HttpProvider::new(&config, ProviderKind::OpenAiCompatible).unwrap();
        assert_eq!(
            provider.url("/models"),
            "https://generativelanguage.googleapis.com/v1beta/openai/models"
        );

        config.provider = "ollama".into();
        config.endpoint = "http://127.0.0.1:11434".into();
        let provider = HttpProvider::new(&config, ProviderKind::Ollama).unwrap();
        assert_eq!(provider.url("/api/tags"), "http://127.0.0.1:11434/api/tags");
    }

    #[cfg(feature = "ai")]
    #[test]
    fn accepts_prefixed_openai_models() {
        let names = vec!["models/gemini-2.5-flash".to_string()];
        assert!(openai_model_available(&names, "gemini-2.5-flash"));
        assert!(!openai_model_available(&names, "gemini-2.5-pro"));
    }

    #[test]
    fn parses_plan_inside_markdown_fence() {
        let raw = r#"```json
{"project":"api","tasks":[
  {"title":"Design schema","priority":"high","due":"2030-01-02","tags":["db","db"]},
  {"title":"Build endpoint","description":"Wire it up","depends_on":[0]}
]}
```"#;
        let plan = parse_response(raw, 5).unwrap();
        assert_eq!(plan.project.as_deref(), Some("api"));
        assert_eq!(plan.tasks[0].priority.as_deref(), Some("high"));
        assert_eq!(plan.tasks[0].tags, vec!["db"]);
        assert_eq!(plan.tasks[1].depends_on, vec![0]);
    }

    #[test]
    fn rejects_invalid_plan_fields() {
        let too_many = DraftPlan {
            project: None,
            tasks: vec![
                DraftTask {
                    title: "One".into(),
                    description: String::new(),
                    project: None,
                    priority: None,
                    due: None,
                    tags: vec![],
                    depends_on: vec![],
                },
                DraftTask {
                    title: "Two".into(),
                    description: String::new(),
                    project: None,
                    priority: None,
                    due: None,
                    tags: vec![],
                    depends_on: vec![],
                },
            ],
        };
        assert!(normalize_plan(too_many, 1).is_err());
        assert!(normalize_plan(
            DraftPlan {
                project: None,
                tasks: vec![DraftTask {
                    title: "  ".into(),
                    description: String::new(),
                    project: None,
                    priority: None,
                    due: None,
                    tags: vec![],
                    depends_on: vec![],
                }],
            },
            5
        )
        .is_err());
        assert!(normalize_plan(
            DraftPlan {
                project: None,
                tasks: vec![DraftTask {
                    title: "One".into(),
                    description: String::new(),
                    project: None,
                    priority: Some("impossible".into()),
                    due: None,
                    tags: vec![],
                    depends_on: vec![],
                }],
            },
            5
        )
        .is_err());
        assert!(normalize_plan(
            DraftPlan {
                project: None,
                tasks: vec![DraftTask {
                    title: "One".into(),
                    description: String::new(),
                    project: None,
                    priority: None,
                    due: None,
                    tags: vec![],
                    depends_on: vec![0],
                }],
            },
            5
        )
        .is_err());
    }

    #[test]
    fn creates_tasks_with_dependencies() {
        let plan = DraftPlan {
            project: Some("api".into()),
            tasks: vec![
                DraftTask {
                    title: "First".into(),
                    description: "Start here".into(),
                    project: None,
                    priority: Some("high".into()),
                    due: Some("2030-01-02".into()),
                    tags: vec!["one".into()],
                    depends_on: vec![],
                },
                DraftTask {
                    title: "Second".into(),
                    description: String::new(),
                    project: Some("backend".into()),
                    priority: None,
                    due: None,
                    tags: vec![],
                    depends_on: vec![0],
                },
            ],
        };
        let data = ProjectData::default();
        let tasks = plan_tasks(&plan, &data, "default").unwrap();
        assert_eq!(tasks.len(), 2);
        assert_eq!(tasks[0].project, "api");
        assert_eq!(tasks[1].project, "backend");
        assert_eq!(tasks[0].priority, Priority::High);
        assert_eq!(tasks[1].blocked_by, vec![tasks[0].id]);
        assert_eq!(tasks[0].description, "Start here");
    }

    #[test]
    fn creates_breakdown_tasks_inheriting_parent() {
        let mut parent = task("Parent");
        parent.project = "api".into();
        parent.branch = "feature".into();
        parent.priority = Priority::Urgent;
        parent.git_branch = "feature".into();
        let plan = DraftPlan {
            project: None,
            tasks: vec![DraftTask {
                title: "Child".into(),
                description: String::new(),
                project: None,
                priority: None,
                due: None,
                tags: vec![],
                depends_on: vec![],
            }],
        };
        let tasks = breakdown_tasks(&parent, &plan).unwrap();
        assert_eq!(tasks[0].parent, Some(parent.id));
        assert_eq!(tasks[0].project, "api");
        assert_eq!(tasks[0].branch, "feature");
        assert_eq!(tasks[0].priority, Priority::Urgent);
        assert_eq!(tasks[0].git_branch, "feature");
    }

    #[test]
    fn fixture_provider_feeds_plan() {
        let config = AiConfig::default();
        let provider = FixtureProvider {
            response: r#"{"tasks":[{"title":"Ship it"}]}"#.into(),
        };
        let plan = request_plan(
            &provider,
            &config,
            PlanRequest {
                goal: "ship the feature",
                tasks: &[],
                branch: "main",
                project: None,
                limit: 3,
                include_descriptions: true,
            },
        )
        .unwrap();
        assert_eq!(plan.tasks[0].title, "Ship it");
        assert_eq!(provider.doctor().unwrap(), vec!["fixture is ready"]);
    }

    #[test]
    fn redacts_common_secrets() {
        let text = "API_KEY=supersecret\nAuthorization: Bearer sk-123\nUse ghp_abcdef for access\nNormal task title";
        let redacted = redact_text(text);
        assert!(!redacted.contains("supersecret"), "{redacted}");
        assert!(!redacted.contains("sk-123"), "{redacted}");
        assert!(!redacted.contains("ghp_abcdef"), "{redacted}");
        assert!(redacted.contains("Normal task title"), "{redacted}");

        let key = "-----BEGIN RSA PRIVATE KEY-----\nabc123\n-----END RSA PRIVATE KEY-----";
        let redacted = redact_text(key);
        assert_eq!(redacted, "[REDACTED PRIVATE KEY]");
    }

    #[test]
    fn prompts_respect_description_setting() {
        let config = AiConfig::default();
        let mut open = task("Visible task");
        open.description = "Detailed context".into();
        let with_details = plan_prompt(&config, "plan", &[open.clone()], "main", None, 5, true);
        assert!(with_details.contains("Detailed context"));
        let without_details = plan_prompt(&config, "plan", &[open], "main", None, 5, false);
        assert!(!without_details.contains("Detailed context"));
    }

    #[test]
    fn offline_triage_orders_open_work_and_skips_blocked() {
        let mut blocker = task("Blocker");
        blocker.status = Status::Doing;
        let mut blocked = task("Blocked urgent");
        blocked.priority = Priority::Urgent;
        blocked.status = Status::Doing;
        blocked.blocked_by.push(blocker.id);
        let mut done = task("Done task");
        done.status = Status::Done;
        let mut other_branch = task("Other branch");
        other_branch.branch = "feature".into();
        let tasks = vec![
            blocker.clone(),
            blocked.clone(),
            done,
            other_branch,
            task("Normal task"),
        ];

        let picks = deterministic_triage(TriageRequest {
            tasks: &tasks,
            branch: "main",
            project: None,
            limit: 5,
            include_blocked: false,
            include_descriptions: true,
        });
        assert_eq!(picks.len(), 2);
        assert_eq!(picks[0].task_id, blocker.id);
        assert!(picks.iter().all(|pick| pick.task_id != blocked.id));

        let picks = deterministic_triage(TriageRequest {
            tasks: &tasks,
            branch: "main",
            project: None,
            limit: 1,
            include_blocked: true,
            include_descriptions: true,
        });
        assert_eq!(picks.len(), 1);
        assert_eq!(picks[0].task_id, blocked.id);
        assert!(picks[0].reason.contains("blocked by 1"));
    }

    #[test]
    fn triage_response_parsing_is_strict() {
        let response = parse_triage_response(
            r#"```json
            {"summary":"Focus on the blocker","picks":[
              {"id":"abc","reason":"overdue"},
              {"id":"abc","reason":"duplicate"}
            ]}
            ```"#,
            3,
        )
        .unwrap();
        assert_eq!(response.summary, "Focus on the blocker");
        assert_eq!(response.picks.len(), 1);
        assert_eq!(response.picks[0].id, "abc");
        assert!(parse_triage_response(r#"{"picks":[]}"#, 3).is_err());
        assert!(parse_triage_response(
            r#"{"picks":[{"id":"a"},{"id":"b"},{"id":"c"},{"id":"d"}]}"#,
            3
        )
        .is_err());
        assert!(parse_triage_response(r#"not json"#, 3).is_err());
    }

    #[test]
    fn resolves_triage_ids_by_prefix() {
        let first = task("First");
        let second = task("Second");
        let tasks = vec![first.clone(), second.clone()];
        let response = TriageResponse {
            summary: "pick".into(),
            picks: vec![
                DraftTriagePick {
                    id: first.id.to_string()[..8].into(),
                    reason: "start here".into(),
                },
                DraftTriagePick {
                    id: second.id.to_string(),
                    reason: "then this".into(),
                },
            ],
        };
        let picks = resolve_triage(&response, &tasks, "main", None, false).unwrap();
        assert_eq!(picks[0].task_id, first.id);
        assert_eq!(picks[1].task_id, second.id);

        let response = TriageResponse {
            summary: "bad".into(),
            picks: vec![DraftTriagePick {
                id: "does-not-exist".into(),
                reason: String::new(),
            }],
        };
        assert!(resolve_triage(&response, &tasks, "main", None, false).is_err());

        let mut done = task("Done");
        done.status = Status::Done;
        let tasks = vec![done.clone()];
        let response = TriageResponse {
            summary: "done".into(),
            picks: vec![DraftTriagePick {
                id: done.id.to_string(),
                reason: String::new(),
            }],
        };
        let error = resolve_triage(&response, &tasks, "main", None, false).unwrap_err();
        assert!(error.to_string().contains("not eligible"), "{error}");
    }

    #[test]
    fn parses_reports_and_answers() {
        let report = parse_report(
            r#"{"summary":"Shipped the API","sections":[
              {"title":"Done","items":["Added endpoint",""]},
              {"title":"Next","items":["Write tests"]}
            ]}"#,
            5,
        )
        .unwrap();
        assert_eq!(report.summary, "Shipped the API");
        assert_eq!(report.sections[0].items, vec!["Added endpoint"]);
        assert!(parse_report(r#"{"summary":"empty"}"#, 5).is_err());
        assert!(parse_report(
            r#"{"summary":"too many","sections":[{"title":"A","items":["1","2"]}]}"#,
            1
        )
        .is_err());

        let answer = parse_answer(r#"{"answer":"  Do the blocker first.  "}"#).unwrap();
        assert_eq!(answer.answer, "Do the blocker first.");
        assert!(parse_answer(r#"{"answer":"   "}"#).is_err());
    }

    #[test]
    fn report_prompt_includes_recent_completed_work() {
        let config = AiConfig::default();
        let mut completed = task("Finished today");
        completed.status = Status::Done;
        completed.completed_at = Some(Utc::now());
        let mut closed = task("Finished long ago");
        closed.status = Status::Done;
        closed.completed_at = Some(Utc::now() - Duration::days(60));
        let prompt = report_prompt(
            &config,
            &ReportRequest {
                kind: ReportKind::Standup,
                tasks: &[completed, closed],
                branch: "main",
                project: None,
                days: 1,
                limit: 5,
                include_descriptions: true,
            },
        );
        assert!(prompt.contains("Finished today"), "{prompt}");
        assert!(!prompt.contains("Finished long ago"), "{prompt}");
    }
}
