use crate::{
    model::{ProjectData, Task},
    store,
};
use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{Html, IntoResponse},
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
        .route("/api/tasks/{id}", put(update))
        .with_state(state);
    let listener = tokio::net::TcpListener::bind(format!("{host}:{port}")).await?;
    println!("Tasklynx web → http://{host}:{port}");
    axum::serve(listener, app).await?;
    Ok(())
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
