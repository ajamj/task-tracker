//! Error types for the tt CLI with actionable suggestions.
//!
//! This module defines the error types used throughout the application,
//! providing clear, actionable error messages for users.

use thiserror::Error;

/// Main error type for the tt CLI.
#[derive(Error, Debug)]
pub enum TtError {
    #[error("Workspace not initialized. Run 'tt init' first.")]
    WorkspaceNotFound,

    #[error("Workspace not found at {0}")]
    WorkspaceNotFoundAtPath(String),

    #[error("Project '{0}' not found")]
    ProjectNotFound(String),

    #[error("Task '{0}' not found")]
    TaskNotFound(String),

    #[error("Invalid status transition: {from} → {to}")]
    InvalidStatusTransition { from: String, to: String },

    #[error("Failed to parse TOML: {0}")]
    TomlParseError(#[from] toml_edit::TomlError),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Template error: {0}")]
    TemplateError(#[from] minijinja::Error),

    #[error("ID generation failed: {0}")]
    IdGenerationError(String),

    #[error("File lock failed: {0}")]
    LockError(String),

    #[error("Date parsing error: {0}")]
    DateParseError(String),

    #[error("Invalid week format: {0}. Expected format: YYYY-Www (e.g., 2026-W13)")]
    InvalidWeekFormat(String),

    #[error("Editor not found. Set $EDITOR environment variable.")]
    EditorNotFound,

    #[error("Search index error: {0}")]
    SearchIndexError(String),
}

impl TtError {
    /// Get user-friendly suggestions for this error
    pub fn suggestions(&self) -> Vec<String> {
        match self {
            TtError::WorkspaceNotFound => vec![
                "Run 'tt init' to initialize a new workspace".to_string(),
                "Or navigate to an existing workspace directory".to_string(),
            ],
            TtError::WorkspaceNotFoundAtPath(path) => vec![
                format!("Check if the path exists: {}", path),
                "Run 'tt init' to initialize a workspace here".to_string(),
            ],
            TtError::ProjectNotFound(project) => vec![
                format!("Run 'tt project ls' to see available projects"),
                format!("Use --project flag with a valid project name"),
                format!("Project '{}' doesn't exist in this workspace", project),
            ],
            TtError::TaskNotFound(_id) => vec![
                "Check the task ID (format: tt-XXXXXX)".to_string(),
                "Run 'tt ls' to see all tasks".to_string(),
                "Task may be in a different project (use --project flag)".to_string(),
            ],
            TtError::InvalidStatusTransition { from, to } => vec![
                format!("Use 'tt start <id>' to transition to 'doing' first"),
                format!("Valid transitions: todo→doing, doing→done, todo→blocked"),
                format!("Cannot transition directly from '{}' to '{}'", from, to),
            ],
            TtError::TomlParseError(_) => vec![
                "Check the TOML file for syntax errors".to_string(),
                "TOML keys must be quoted strings".to_string(),
                "Ensure proper formatting (key = value)".to_string(),
            ],
            TtError::IoError(e) => vec![
                format!("IO error: {}", e),
                "Check file permissions".to_string(),
                "Ensure the file/directory exists".to_string(),
            ],
            TtError::TemplateError(_) => vec![
                "Check template syntax (Jinja2 format)".to_string(),
                "Ensure all template variables are defined".to_string(),
                "Try using the default template (remove custom template path)".to_string(),
            ],
            TtError::IdGenerationError(_) => vec![
                "Check file permissions in the tasks directory".to_string(),
                "Ensure no other process is locking the ID counter".to_string(),
                "Try removing the .tt/lock file if it exists".to_string(),
            ],
            TtError::LockError(_) => vec![
                "Another tt process may be running".to_string(),
                "Remove the .tt/lock file if no other process is running".to_string(),
            ],
            TtError::DateParseError(_) => vec![
                "Use YYYY-MM-DD format (e.g., 2026-04-03)".to_string(),
                "Ensure the date is valid (not Feb 30, etc.)".to_string(),
            ],
            TtError::InvalidWeekFormat(_) => vec![
                "Use ISO week format: YYYY-Www (e.g., 2026-W13)".to_string(),
                "Week number must be 01-53".to_string(),
            ],
            TtError::EditorNotFound => vec![
                "Set the $EDITOR environment variable".to_string(),
                "On Windows: setx EDITOR \"notepad.exe\"".to_string(),
                "On macOS/Linux: export EDITOR=\"vim\"".to_string(),
            ],
            TtError::SearchIndexError(_) => vec![
                "Try rebuilding the search index: rm -rf .tt/index".to_string(),
                "Ensure the workspace is initialized".to_string(),
            ],
        }
    }

    /// Display this error with suggestions
    pub fn display_with_suggestions(&self) -> String {
        let mut output = format!("{}\n", self);
        let suggestions = self.suggestions();
        
        if !suggestions.is_empty() {
            output.push_str("\nSuggestions:\n");
            for suggestion in suggestions {
                output.push_str(&format!("  - {}\n", suggestion));
            }
        }
        
        output
    }
}

/// Storage-specific errors.
#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Workspace not found at {0}")]
    WorkspaceNotFound(String),

    #[error("Project '{0}' not found")]
    ProjectNotFound(String),

    #[error("Task '{0}' not found")]
    TaskNotFound(String),

    #[error("Failed to parse TOML: {0}")]
    TomlParseError(#[from] toml_edit::TomlError),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("ID generation failed: {0}")]
    IdGenerationError(String),

    #[error("File lock failed: {0}")]
    LockError(String),
}

impl From<StorageError> for TtError {
    fn from(err: StorageError) -> Self {
        match err {
            StorageError::WorkspaceNotFound(path) => TtError::WorkspaceNotFoundAtPath(path),
            StorageError::ProjectNotFound(name) => TtError::ProjectNotFound(name),
            StorageError::TaskNotFound(id) => TtError::TaskNotFound(id),
            StorageError::TomlParseError(e) => TtError::TomlParseError(e),
            StorageError::IoError(e) => TtError::IoError(e),
            StorageError::IdGenerationError(msg) => TtError::IdGenerationError(msg),
            StorageError::LockError(msg) => TtError::LockError(msg),
        }
    }
}

/// Result type alias for tt operations.
pub type Result<T> = std::result::Result<T, TtError>;

/// Result type alias for storage operations.
pub type StorageResult<T> = std::result::Result<T, StorageError>;
