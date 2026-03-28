//! tt - Git-friendly personal task tracking CLI
//!
//! A local-first task tracker that stores tasks in TOML files and daily logs in Markdown,
//! all within a dedicated Git repository. Generates weekly reports and provides git
//! commit/branch suggestions (but never executes git commands automatically).
//!
//! # Example
//!
//! ```bash
//! tt init                    # Initialize workspace
//! tt add "Refactor config"   # Create task
//! tt ls                      # List tasks
//! tt start tt-000001         # Start working
//! tt log "Made progress"     # Log work
//! tt done tt-000001          # Complete task
//! tt report week             # Generate weekly report
//! ```

pub mod error;
pub mod models;
pub mod storage;
pub mod reports;
pub mod cli;
// pub mod api;  // Dashboard API - requires proper axum setup
// pub mod dashboard;  // Dashboard UI - requires proper axum setup
// pub mod search;  // Temporarily disabled due to tantivy dependency conflict

// Re-export commonly used types
pub use error::{Result, StorageError, StorageResult, TtError};
pub use models::{NewTask, Priority, Task, TaskStatus, WeekRange, WorkspaceConfig};
pub use storage::{Project, Workspace, TaskStorage, Log, LogStorage, scan_for_task_ids};
pub use reports::WeeklyReport;
pub use cli::{Cli, Commands, execute};
