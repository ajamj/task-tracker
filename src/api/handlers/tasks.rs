//! Task API handlers.

use axum::{Json, extract::State};
use crate::api::AppState;
use crate::storage::TaskStorage;
use serde_json::json;

/// List all tasks.
pub async fn list_tasks(State(state): State<AppState>) -> Json<serde_json::Value> {
    let project = match state.workspace.default_project() {
        Ok(p) => p,
        Err(e) => return Json(json!({ "error": e.to_string() })),
    };

    let task_storage = TaskStorage::new(project.tasks_dir.clone());
    
    match task_storage.list() {
        Ok(tasks) => {
            let task_list: Vec<serde_json::Value> = tasks.iter().map(|task| {
                json!({
                    "id": task.id,
                    "title": task.title,
                    "status": task.status.to_string(),
                    "priority": task.priority.as_ref().map(|p| p.to_string()),
                    "tags": task.tags,
                    "due": task.due,
                    "created_at": task.created_at,
                })
            }).collect();

            Json(json!({
                "tasks": task_list,
                "total": task_list.len()
            }))
        }
        Err(e) => Json(json!({ "error": e.to_string() }))
    }
}
