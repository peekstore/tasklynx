use crate::model::{local_due_date, AiConfig, Priority, ProjectData, Status, Task};
use anyhow::{anyhow, bail, Context, Result};
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

pub trait Provider {
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
    let raw = provider.generate(&prompt)?;
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
    let raw = provider.generate(&prompt)?;
    parse_response(&raw, limit)
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
    let include_detail = config.include_descriptions && include_descriptions;
    let mut selected: Vec<&Task> = tasks
        .iter()
        .filter(|task| task.status != Status::Done)
        .filter(|task| task.branch == branch)
        .filter(|task| project.map_or(true, |project| task.project == project))
        .collect();
    selected.sort_by_key(|task| (std::cmp::Reverse(task.priority.clone()), task.created_at));
    selected.truncate(config.max_context_tasks);
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
}
