//! Data models for the tt CLI.
//!
//! This module defines the core data structures used throughout the application:
//! - Task: Represents a single task with all its metadata
//! - TaskStatus: The lifecycle state of a task
//! - Priority: Task priority levels (P0-P3)
//! - WeekRange: ISO-8601 week calculations with Monday start
//! - Config: Workspace and project configuration

pub mod config;
pub mod task;
pub mod week;

pub use config::{ProjectConfig, WorkspaceConfig};
pub use task::{NewTask, Priority, Task, TaskStatus};
pub use week::WeekRange;
