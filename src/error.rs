//! Error types for the tt CLI.
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
