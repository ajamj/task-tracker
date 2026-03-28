//! Task API handlers.

use axum::{Json, extract::State};
use crate::api::AppState;
use crate::storage::TaskStorage;

/// List all tasks.
pub async fn list_tasks(State(state): State<AppState>) -> Json<serde_json::Value> {
    let project = match state.workspace.get_default_project() {
        Ok(p) => p,
        Err(e) => return Json(serde_json::json!({ "error": e.to_string() })),
    };

    let task_storage = TaskStorage::new(project.tasks_dir.clone());
    
    match task_storage.list() {
        Ok(tasks) => {
            let task_list: Vec<serde_json::Value> = tasks.iter().map(|task| {
                serde_json::json!({
                    "id": task.id,
                    "title": task.title,
                    "status": task.status.to_string(),
                    "priority": task.priority.as_ref().map(|p| p.to_string()),
                    "tags": task.tags.clone(),
                    "due": task.due.clone(),
                    "created_at": task.created_at.clone(),
                })
            }).collect();

            Json(serde_json::json!({
                "tasks": task_list,
                "total": task_list.len()
            }))
        }
        Err(e) => Json(serde_json::json!({ "error": e.to_string() }))
    }
}
