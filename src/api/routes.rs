//! API routes.

use axum::{Router, routing::get};
use crate::api::AppState;
use crate::api::handlers::tasks;
use crate::dashboard::views;

/// Create API router.
pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Dashboard pages
        .route("/", get(views::dashboard))
        // API endpoints
        .route("/api/tasks", get(tasks::list_tasks))
        .route("/api/stats", get(views::get_stats))
        .with_state(state)
}
