//! Dashboard views and routes.

use crate::api::AppState;
use crate::storage::TaskStorage;
use crate::models::TaskStatus;

/// Serve the main dashboard page.
pub async fn dashboard() -> impl axum::response::IntoResponse {
    let html = include_str!("templates/index.html");
    axum::response::Html(html)
}

/// Get task counts for stats.
pub async fn get_stats(axum::extract::State(state): axum::extract::State<AppState>) -> axum::Json<serde_json::Value> {
    let project = match state.workspace.get_default_project() {
        Ok(p) => p,
        Err(_) => return axum::Json(serde_json::json!({ "error": "No project found" })),
    };

    let task_storage = TaskStorage::new(project.tasks_dir.clone());
    let tasks = match task_storage.list() {
        Ok(t) => t,
        Err(_) => return axum::Json(serde_json::json!({ "error": "Failed to load tasks" })),
    };

    let todo_count = tasks.iter().filter(|t| matches!(t.status, TaskStatus::Todo)).count();
    let doing_count = tasks.iter().filter(|t| matches!(t.status, TaskStatus::Doing)).count();
    let done_count = tasks.iter().filter(|t| matches!(t.status, TaskStatus::Done)).count();
    let blocked_count = tasks.iter().filter(|t| matches!(t.status, TaskStatus::Blocked)).count();

    axum::Json(serde_json::json!({
        "todo": todo_count,
        "doing": doing_count,
        "done": done_count,
        "blocked": blocked_count
    }))
}
