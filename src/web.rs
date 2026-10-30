use crate::{
    model::{ProjectData, Repeat, Task},
    store,
};
use axum::{
    extract::{DefaultBodyLimit, Path, Request, State},
    http::{header, HeaderValue, StatusCode},
    middleware::{self, Next},
    response::{Html, IntoResponse, Response},
    routing::{get, put},
    Json, Router,
};
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
}
