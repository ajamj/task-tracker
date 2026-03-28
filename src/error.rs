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
    pub fn suggestions(&self) -> Vec<&str> {
        match self {
            TtError::WorkspaceNotFound => vec![
                "Run 'tt init' to initialize a new workspace",
                "Or navigate to an existing workspace directory",
            ],
            TtError::WorkspaceNotFoundAtPath(path) => vec![
                &format!("Check if the path exists: {}", path),
                "Run 'tt init' to initialize a workspace here",
            ],
            TtError::ProjectNotFound(project) => vec![
                &format!("Run 'tt project ls' to see available projects"),
                &format!("Use --project flag with a valid project name"),
                &format!("Project '{}' doesn't exist in this workspace", project),
            ],
            TtError::TaskNotFound(id) => vec![
                "Check the task ID (format: tt-XXXXXX)",
                "Run 'tt ls' to see all tasks",
                "Task may be in a different project (use --project flag)",
            ],
            TtError::InvalidStatusTransition { from, to } => vec![
                &format!("Use 'tt start <id>' to transition to 'doing' first"),
                &format!("Valid transitions: todo→doing, doing→done, todo→blocked"),
                &format!("Cannot transition directly from '{}' to '{}'", from, to),
            ],
            TtError::TomlParseError(_) => vec![
                "Check the TOML file for syntax errors",
                "TOML keys must be quoted strings",
                "Ensure proper formatting (key = value)",
            ],
            TtError::IoError(e) => vec![
                &format!("IO error: {}", e),
                "Check file permissions",
                "Ensure the file/directory exists",
            ],
            TtError::TemplateError(_) => vec![
                "Check template syntax (Jinja2 format)",
                "Ensure all template variables are defined",
                "Try using the default template (remove custom template path)",
            ],
            TtError::IdGenerationError(_) => vec![
                "Check file permissions in the tasks directory",
                "Ensure no other process is locking the ID counter",
                "Try removing the .tt/lock file if it exists",
            ],
            TtError::LockError(_) => vec![
                "Another tt process may be running",
                "Remove the .tt/lock file if no other process is running",
            ],
            TtError::DateParseError(_) => vec![
                "Use YYYY-MM-DD format (e.g., 2026-04-03)",
                "Ensure the date is valid (not Feb 30, etc.)",
            ],
            TtError::InvalidWeekFormat(_) => vec![
                "Use ISO week format: YYYY-Www (e.g., 2026-W13)",
                "Week number must be 01-53",
            ],
            TtError::EditorNotFound => vec![
                "Set the $EDITOR environment variable",
                "On Windows: setx EDITOR \"notepad.exe\"",
                "On macOS/Linux: export EDITOR=\"vim\"",
            ],
            TtError::SearchIndexError(_) => vec![
                "Try rebuilding the search index: rm -rf .tt/index",
                "Ensure the workspace is initialized",
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

/// Result type alias for tt operations.
pub type Result<T> = std::result::Result<T, TtError>;

/// Result type alias for storage operations.
pub type StorageResult<T> = std::result::Result<T, StorageError>;
