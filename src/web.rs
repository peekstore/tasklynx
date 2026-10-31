use crate::{
    ai, config,
    model::{ProjectData, Repeat, Task},
    store,
};
use axum::{
    extract::{DefaultBodyLimit, Path, Request, State},
    http::{header, HeaderValue, StatusCode},
    middleware::{self, Next},
    response::{Html, IntoResponse, Response},
    routing::{get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

const INDEX: &str = include_str!("../assets/index.html");
const FAVICON: &str = include_str!("../assets/favicon.svg");

#[derive(Clone)]
pub struct WebState {
    pub data: Arc<Mutex<ProjectData>>,
}

pub async fn serve(host: String, port: u16, data: ProjectData) -> anyhow::Result<()> {
    let state = WebState {
        data: Arc::new(Mutex::new(data)),
    };
    let app = Router::new()
        .route("/", get(index))
        .route("/favicon.svg", get(favicon))
        .route("/api/tasks", get(tasks).post(create))
        .route("/api/tasks/{id}", put(update).delete(remove))
        .route("/api/ai/triage", post(ai_triage))
        .route("/api/ai/plan", post(ai_plan))
        .route("/api/ai/apply-plan", post(ai_apply_plan))
        .route("/api/ai/breakdown", post(ai_breakdown))
        .route("/api/ai/apply-breakdown", post(ai_apply_breakdown))
        .route("/api/ai/standup", post(ai_standup))
        .route("/api/ai/review", post(ai_review))
        .route("/api/ai/ask", post(ai_ask))
        .layer(middleware::from_fn(security_headers))
        .layer(DefaultBodyLimit::max(2 * 1024 * 1024))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!("{host}:{port}")).await?;
    println!("Tasklynx web → http://{host}:{port}");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn security_headers(req: Request, next: Next) -> Response {
    let mut resp = next.run(req).await;
    let headers = resp.headers_mut();
    headers.insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static(
            "default-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self';",
        ),
    );
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(header::X_FRAME_OPTIONS, HeaderValue::from_static("DENY"));
    headers.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("no-referrer"),
    );
    resp
}

async fn index() -> Html<&'static str> {
    Html(INDEX)
}

async fn favicon() -> impl IntoResponse {
    ([(header::CONTENT_TYPE, "image/svg+xml")], FAVICON)
}

async fn tasks(State(s): State<WebState>) -> Json<ProjectData> {
    Json(s.data.lock().unwrap().clone())
}

async fn create(State(s): State<WebState>, Json(t): Json<Task>) -> Json<Task> {
    let mut d = s.data.lock().unwrap();
    let previous = d.clone();
    d.tasks.push(t.clone());
    let _ = store::save_with_undo(&d, &previous);
    Json(t)
}

fn merge_task(existing: &mut Task, incoming: Task) -> (Task, Option<Task>) {
    let next = if existing.status != incoming.status {
        existing.set_status(incoming.status)
    } else {
        None
    };
    existing.title = incoming.title;
    existing.description = incoming.description;
    existing.project = incoming.project;
    existing.tags = incoming.tags;
    existing.priority = incoming.priority;
    existing.due = incoming.due;
    existing.branch = incoming.branch;
    existing.commit = incoming.commit;
    existing.git_branch = incoming.git_branch;
    existing.git_remote = incoming.git_remote;
    existing.git_pr_url = incoming.git_pr_url;
    existing.source_path = incoming.source_path;
    existing.source_line = incoming.source_line;
    existing.parent = incoming.parent;
    existing.blocked_by = incoming.blocked_by;
    existing.estimate_minutes = incoming.estimate_minutes;
    existing.repeat = incoming.repeat;
    existing.repeat_until = if existing.repeat == Repeat::None {
        None
    } else {
        incoming.repeat_until
    };
    existing.updated_at = chrono::Utc::now();
    (existing.clone(), next)
}

async fn update(
    Path(id): Path<String>,
    State(s): State<WebState>,
    Json(t): Json<Task>,
) -> impl IntoResponse {
    let mut d = s.data.lock().unwrap();
    let previous = d.clone();
    match store::find_mut(&mut d, &id) {
        Ok(x) => {
            let (updated, next) = merge_task(x, t);
            if let Some(next) = next {
                d.tasks.push(next);
            }
            let _ = store::save_with_undo(&d, &previous);
            Json(updated).into_response()
        }
        Err(e) => (StatusCode::NOT_FOUND, e.to_string()).into_response(),
    }
}

async fn remove(Path(id): Path<String>, State(s): State<WebState>) -> impl IntoResponse {
    let mut d = s.data.lock().unwrap();
    let previous = d.clone();
    match store::delete_to_trash(&mut d, &id) {
        Ok(_) => {
            let _ = store::save_with_undo(&d, &previous);
            (StatusCode::OK, "deleted").into_response()
        }
        Err(e) => (StatusCode::NOT_FOUND, e.to_string()).into_response(),
    }
}

fn default_triage_limit() -> usize {
    5
}

fn default_plan_limit() -> usize {
    10
}

fn default_report_limit() -> usize {
    10
}

fn default_standup_days() -> u32 {
    1
}

fn default_review_days() -> u32 {
    30
}

fn clean_project(project: Option<String>) -> Option<String> {
    project
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

async fn run_ai<T, F>(work: F) -> Result<T, (StatusCode, String)>
where
    F: FnOnce() -> anyhow::Result<T> + Send + 'static,
    T: Send + 'static,
{
    match tokio::task::spawn_blocking(work).await {
        Ok(Ok(value)) => Ok(value),
        Ok(Err(error)) => Err((StatusCode::BAD_GATEWAY, format!("AI error: {error}"))),
        Err(error) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("AI task failed: {error}"),
        )),
    }
}

#[derive(Deserialize)]
struct AiTriageBody {
    #[serde(default)]
    project: Option<String>,
    #[serde(default = "default_triage_limit")]
    limit: usize,
    #[serde(default)]
    include_blocked: bool,
    #[serde(default)]
    no_description: bool,
    #[serde(default)]
    offline: bool,
}

#[derive(Serialize)]
struct AiTriagePickDto {
    id: String,
    title: String,
    project: String,
    priority: String,
    status: String,
    due: Option<String>,
    reason: String,
}

#[derive(Serialize)]
struct AiTriageResponseDto {
    provider: String,
    model: String,
    offline: bool,
    summary: String,
    picks: Vec<AiTriagePickDto>,
}

async fn ai_triage(State(s): State<WebState>, Json(body): Json<AiTriageBody>) -> Response {
    if body.limit == 0 || body.limit > 50 {
        return (StatusCode::BAD_REQUEST, "--limit must be between 1 and 50").into_response();
    }
    let data = s.data.lock().unwrap().clone();
    let project = clean_project(body.project);
    let result = run_ai(move || {
        let config = config::load()?;
        let include_descriptions = config.ai.include_descriptions && !body.no_description;
        let request = ai::TriageRequest {
            tasks: &data.tasks,
            branch: &data.current_branch,
            project: project.as_deref(),
            limit: body.limit,
            include_blocked: body.include_blocked,
            include_descriptions,
        };
        let offline = body.offline || matches!(config.ai.provider_kind()?, ai::ProviderKind::None);
        let no_candidates = ai::triage_candidate_count(
            &data.tasks,
            &data.current_branch,
            project.as_deref(),
            body.include_blocked,
        ) == 0;
        let (provider, summary, picks) = if offline || no_candidates {
            (
                "offline".to_string(),
                if no_candidates {
                    "no eligible tasks to triage".to_string()
                } else {
                    "offline triage".to_string()
                },
                ai::deterministic_triage(request),
            )
        } else {
            let provider = ai::provider(&config.ai)?;
            let response = ai::request_triage(provider.as_ref(), &config.ai, request)?;
            let picks = ai::resolve_triage(
                &response,
                &data.tasks,
                &data.current_branch,
                project.as_deref(),
                body.include_blocked,
            )?;
            (provider.name().to_string(), response.summary, picks)
        };
        let picks = picks
            .into_iter()
            .filter_map(|pick| {
                let task = store::find(&data, &pick.task_id.to_string()).ok()?;
                Some(AiTriagePickDto {
                    id: task.id.to_string(),
                    title: task.title.clone(),
                    project: task.project.clone(),
                    priority: task.priority.to_string(),
                    status: task.status.to_string(),
                    due: task.due.map(|due| {
                        due.with_timezone(&chrono::Local)
                            .format("%Y-%m-%d")
                            .to_string()
                    }),
                    reason: pick.reason,
                })
            })
            .collect();
        Ok(AiTriageResponseDto {
            provider,
            model: config.ai.model.clone(),
            offline,
            summary,
            picks,
        })
    })
    .await;
    match result {
        Ok(response) => Json(response).into_response(),
        Err((status, message)) => (status, message).into_response(),
    }
}

#[derive(Deserialize)]
struct AiPlanBody {
    goal: String,
    #[serde(default)]
    project: Option<String>,
    #[serde(default = "default_plan_limit")]
    limit: usize,
    #[serde(default)]
    no_description: bool,
}

#[derive(Serialize)]
struct AiPlanResponseDto {
    provider: String,
    model: String,
    plan: ai::DraftPlan,
}

async fn ai_plan(State(s): State<WebState>, Json(body): Json<AiPlanBody>) -> Response {
    if body.goal.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, "goal must not be empty").into_response();
    }
    if body.limit == 0 || body.limit > 100 {
        return (StatusCode::BAD_REQUEST, "--limit must be between 1 and 100").into_response();
    }
    let data = s.data.lock().unwrap().clone();
    let project = clean_project(body.project);
    let result = run_ai(move || {
        let config = config::load()?;
        let provider = ai::provider(&config.ai)?;
        let plan = ai::request_plan(
            provider.as_ref(),
            &config.ai,
            ai::PlanRequest {
                goal: body.goal.trim(),
                tasks: &data.tasks,
                branch: &data.current_branch,
                project: project.as_deref(),
                limit: body.limit,
                include_descriptions: config.ai.include_descriptions && !body.no_description,
            },
        )?;
        Ok(AiPlanResponseDto {
            provider: provider.name().to_string(),
            model: config.ai.model.clone(),
            plan,
        })
    })
    .await;
    match result {
        Ok(response) => Json(response).into_response(),
        Err((status, message)) => (status, message).into_response(),
    }
}

#[derive(Deserialize)]
struct AiApplyPlanBody {
    plan: ai::DraftPlan,
    #[serde(default)]
    project: Option<String>,
}

async fn ai_apply_plan(State(s): State<WebState>, Json(body): Json<AiApplyPlanBody>) -> Response {
    let config = match config::load() {
        Ok(config) => config,
        Err(error) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()).into_response()
        }
    };
    let plan = match ai::normalize_plan(body.plan, 100) {
        Ok(plan) => plan,
        Err(error) => return (StatusCode::BAD_REQUEST, error.to_string()).into_response(),
    };
    let mut data = s.data.lock().unwrap();
    let default_project = body
        .project
        .as_deref()
        .map(str::trim)
        .filter(|project| !project.is_empty())
        .unwrap_or(&config.default_project);
    let tasks = match ai::plan_tasks(&plan, &data, default_project) {
        Ok(tasks) => tasks,
        Err(error) => return (StatusCode::BAD_REQUEST, error.to_string()).into_response(),
    };
    let previous = data.clone();
    data.tasks.extend(tasks.clone());
    if let Err(error) = store::save_with_undo(&data, &previous) {
        return (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()).into_response();
    }
    Json(tasks).into_response()
}

#[derive(Deserialize)]
struct AiBreakdownBody {
    id: String,
    #[serde(default = "default_plan_limit")]
    limit: usize,
    #[serde(default)]
    no_description: bool,
}

#[derive(Serialize)]
struct AiBreakdownResponseDto {
    provider: String,
    model: String,
    parent_id: String,
    parent_title: String,
    plan: ai::DraftPlan,
}

async fn ai_breakdown(State(s): State<WebState>, Json(body): Json<AiBreakdownBody>) -> Response {
    if body.limit == 0 || body.limit > 100 {
        return (StatusCode::BAD_REQUEST, "--limit must be between 1 and 100").into_response();
    }
    let data = s.data.lock().unwrap().clone();
    let parent = match store::find(&data, &body.id) {
        Ok(parent) => parent.clone(),
        Err(error) => return (StatusCode::NOT_FOUND, error.to_string()).into_response(),
    };
    let children: Vec<Task> = data
        .tasks
        .iter()
        .filter(|task| task.parent == Some(parent.id))
        .cloned()
        .collect();
    let result = run_ai(move || {
        let config = config::load()?;
        let provider = ai::provider(&config.ai)?;
        let plan = ai::request_breakdown(
            provider.as_ref(),
            &config.ai,
            &parent,
            &children,
            body.limit,
            config.ai.include_descriptions && !body.no_description,
        )?;
        Ok(AiBreakdownResponseDto {
            provider: provider.name().to_string(),
            model: config.ai.model.clone(),
            parent_id: parent.id.to_string(),
            parent_title: parent.title.clone(),
            plan,
        })
    })
    .await;
    match result {
        Ok(response) => Json(response).into_response(),
        Err((status, message)) => (status, message).into_response(),
    }
}

#[derive(Deserialize)]
struct AiApplyBreakdownBody {
    parent_id: String,
    plan: ai::DraftPlan,
}

async fn ai_apply_breakdown(
    State(s): State<WebState>,
    Json(body): Json<AiApplyBreakdownBody>,
) -> Response {
    let plan = match ai::normalize_plan(body.plan, 100) {
        Ok(plan) => plan,
        Err(error) => return (StatusCode::BAD_REQUEST, error.to_string()).into_response(),
    };
    let mut data = s.data.lock().unwrap();
    let parent = match store::find(&data, &body.parent_id) {
        Ok(parent) => parent.clone(),
        Err(error) => return (StatusCode::NOT_FOUND, error.to_string()).into_response(),
    };
    let tasks = match ai::breakdown_tasks(&parent, &plan) {
        Ok(tasks) => tasks,
        Err(error) => return (StatusCode::BAD_REQUEST, error.to_string()).into_response(),
    };
    let previous = data.clone();
    data.tasks.extend(tasks.clone());
    if let Err(error) = store::save_with_undo(&data, &previous) {
        return (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()).into_response();
    }
    Json(tasks).into_response()
}

#[derive(Deserialize)]
struct AiReportBody {
    #[serde(default)]
    project: Option<String>,
    #[serde(default)]
    branch: Option<String>,
    #[serde(default)]
    days: Option<u32>,
    #[serde(default)]
    limit: Option<usize>,
    #[serde(default)]
    no_description: bool,
}

#[derive(Serialize)]
struct AiReportResponseDto {
    provider: String,
    model: String,
    report: ai::Report,
}

async fn run_report(
    kind: ai::ReportKind,
    data: ProjectData,
    body: AiReportBody,
    default_days: u32,
) -> Response {
    let days = body.days.unwrap_or(default_days);
    let limit = body.limit.unwrap_or_else(default_report_limit);
    if !(1..=3650).contains(&days) {
        return (StatusCode::BAD_REQUEST, "--days must be between 1 and 3650").into_response();
    }
    if limit == 0 || limit > 50 {
        return (StatusCode::BAD_REQUEST, "--limit must be between 1 and 50").into_response();
    }
    let project = clean_project(body.project);
    let branch = body.branch.unwrap_or_else(|| data.current_branch.clone());
    let result = run_ai(move || {
        let config = config::load()?;
        let provider = ai::provider(&config.ai)?;
        let report = ai::request_report(
            provider.as_ref(),
            &config.ai,
            ai::ReportRequest {
                kind,
                tasks: &data.tasks,
                branch: &branch,
                project: project.as_deref(),
                days,
                limit,
                include_descriptions: config.ai.include_descriptions && !body.no_description,
            },
        )?;
        Ok(AiReportResponseDto {
            provider: provider.name().to_string(),
            model: config.ai.model.clone(),
            report,
        })
    })
    .await;
    match result {
        Ok(response) => Json(response).into_response(),
        Err((status, message)) => (status, message).into_response(),
    }
}

async fn ai_standup(State(s): State<WebState>, Json(body): Json<AiReportBody>) -> Response {
    let data = s.data.lock().unwrap().clone();
    run_report(ai::ReportKind::Standup, data, body, default_standup_days()).await
}

async fn ai_review(State(s): State<WebState>, Json(body): Json<AiReportBody>) -> Response {
    let data = s.data.lock().unwrap().clone();
    run_report(ai::ReportKind::Review, data, body, default_review_days()).await
}

#[derive(Deserialize)]
struct AiAskBody {
    question: String,
    #[serde(default)]
    project: Option<String>,
    #[serde(default)]
    branch: Option<String>,
    #[serde(default)]
    limit: Option<usize>,
    #[serde(default)]
    no_description: bool,
}

#[derive(Serialize)]
struct AiAnswerResponseDto {
    provider: String,
    model: String,
    answer: String,
}

async fn ai_ask(State(s): State<WebState>, Json(body): Json<AiAskBody>) -> Response {
    if body.question.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, "question must not be empty").into_response();
    }
    let limit = body.limit.unwrap_or(50);
    if limit == 0 || limit > 100 {
        return (StatusCode::BAD_REQUEST, "--limit must be between 1 and 100").into_response();
    }
    let data = s.data.lock().unwrap().clone();
    let project = clean_project(body.project);
    let branch = body.branch.unwrap_or_else(|| data.current_branch.clone());
    let result = run_ai(move || {
        let config = config::load()?;
        let provider = ai::provider(&config.ai)?;
        let answer = ai::request_answer(
            provider.as_ref(),
            &config.ai,
            ai::AskRequest {
                question: body.question.trim(),
                tasks: &data.tasks,
                branch: &branch,
                project: project.as_deref(),
                limit,
                include_descriptions: config.ai.include_descriptions && !body.no_description,
            },
        )?;
        Ok(AiAnswerResponseDto {
            provider: provider.name().to_string(),
            model: config.ai.model.clone(),
            answer: answer.answer,
        })
    })
    .await;
    match result {
        Ok(response) => Json(response).into_response(),
        Err((status, message)) => (status, message).into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{local_due_date, Priority, Status};

    fn recurring_task() -> Task {
        let mut t = Task::new(
            "Daily task".into(),
            "default".into(),
            "main".into(),
            Priority::Normal,
            vec![],
            Some(local_due_date("2026-01-01").unwrap()),
        );
        t.repeat = Repeat::Daily;
        t
    }

    #[test]
    fn completing_through_update_creates_next_occurrence() {
        let mut existing = recurring_task();
        let id = existing.id;
        let created_at = existing.created_at;
        let mut incoming = existing.clone();
        incoming.status = Status::Done;
        incoming.title = "Renamed".into();

        let (updated, next) = merge_task(&mut existing, incoming);

        assert_eq!(updated.id, id);
        assert_eq!(updated.created_at, created_at);
        assert_eq!(updated.title, "Renamed");
        assert_eq!(updated.status, Status::Done);
        assert!(updated.completed_at.is_some());
        let next = next.unwrap();
        assert_eq!(next.series_id, Some(id));
        assert_eq!(next.status, Status::Todo);
    }

    #[test]
    fn updating_a_completed_task_does_not_duplicate_occurrence() {
        let mut existing = recurring_task();
        existing.set_status(Status::Done);
        let mut incoming = existing.clone();
        incoming.status = Status::Done;

        let (_, next) = merge_task(&mut existing, incoming);
        assert!(next.is_none());
    }

    #[test]
    fn embedded_ui_exposes_ai_panel() {
        assert!(INDEX.contains("id=\"aiModal\""), "missing AI modal");
        assert!(INDEX.contains("function runAi("), "missing AI runner");
        assert!(INDEX.contains("/api/ai/"), "missing AI endpoint calls");
        assert!(
            INDEX.contains("/api/ai/apply-plan"),
            "missing plan apply endpoint"
        );
    }

    #[test]
    fn project_filter_ignores_blank_values() {
        assert_eq!(
            clean_project(Some("  api  ".into())).as_deref(),
            Some("api")
        );
        assert_eq!(clean_project(Some("   ".into())), None);
        assert_eq!(clean_project(None), None);
    }

    #[tokio::test]
    async fn offline_ai_triage_endpoint_ranks_tasks() {
        let mut data = ProjectData::default();
        let mut urgent = Task::new(
            "Urgent task".into(),
            "default".into(),
            "main".into(),
            Priority::Urgent,
            vec![],
            None,
        );
        urgent.description = "should stay local".into();
        let normal = Task::new(
            "Normal task".into(),
            "default".into(),
            "main".into(),
            Priority::Normal,
            vec![],
            None,
        );
        data.tasks = vec![normal, urgent.clone()];
        let state = WebState {
            data: Arc::new(Mutex::new(data)),
        };
        let response = ai_triage(
            State(state),
            Json(AiTriageBody {
                project: None,
                limit: 5,
                include_blocked: false,
                no_description: false,
                offline: true,
            }),
        )
        .await;
        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(value["provider"], "offline");
        assert_eq!(value["picks"][0]["title"], "Urgent task");
        assert_eq!(value["picks"][0]["id"], urgent.id.to_string());
    }
}
