use crate::{
    model::{ProjectData, Task},
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
    d.tasks.push(t.clone());
    let _ = store::save(&d);
    Json(t)
}

async fn update(
    Path(id): Path<String>,
    State(s): State<WebState>,
    Json(t): Json<Task>,
) -> impl IntoResponse {
    let mut d = s.data.lock().unwrap();
    match store::find_mut(&mut d, &id) {
        Ok(x) => {
            *x = t.clone();
            let _ = store::save(&d);
            Json(t).into_response()
        }
        Err(e) => (StatusCode::NOT_FOUND, e.to_string()).into_response(),
    }
}

async fn remove(Path(id): Path<String>, State(s): State<WebState>) -> impl IntoResponse {
    let mut d = s.data.lock().unwrap();
    let before = d.tasks.len();
    d.tasks
        .retain(|t| t.id.to_string() != id && !t.id.to_string().starts_with(&id));
    if d.tasks.len() == before {
        (StatusCode::NOT_FOUND, format!("task not found: {id}")).into_response()
    } else {
        let _ = store::save(&d);
        (StatusCode::OK, "deleted").into_response()
    }
}
