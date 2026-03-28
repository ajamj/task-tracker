//! Application state for API.

use crate::storage::Workspace;
use std::path::PathBuf;

/// Shared application state.
#[derive(Clone)]
pub struct AppState {
    pub workspace_root: PathBuf,
    pub workspace: Workspace,
}

impl AppState {
    /// Create new AppState.
    pub fn new(workspace_root: PathBuf, workspace: Workspace) -> Self {
        Self {
            workspace_root,
            workspace,
        }
    }
}
