//! Task data model and related types.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Unique task identifier (e.g., "tt-000001").
pub type TaskId = String;

/// Task status representing the lifecycle state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Todo,
    Doing,
    Done,
    Blocked,
    Canceled,
}

impl fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display())
    }
}

impl TaskStatus {
    /// Check if a status transition is valid.
    ///
    /// Valid transitions:
    /// - todo → doing, blocked, canceled
    /// - doing → done, blocked, canceled
    /// - blocked → doing, canceled
    pub fn can_transition_to(self, target: TaskStatus) -> bool {
        matches!(
            (self, target),
            (TaskStatus::Todo, TaskStatus::Doing)
                | (TaskStatus::Todo, TaskStatus::Blocked)
                | (TaskStatus::Todo, TaskStatus::Canceled)
                | (TaskStatus::Doing, TaskStatus::Done)
                | (TaskStatus::Doing, TaskStatus::Blocked)
                | (TaskStatus::Doing, TaskStatus::Canceled)
                | (TaskStatus::Blocked, TaskStatus::Doing)
                | (TaskStatus::Blocked, TaskStatus::Canceled)
        )
    }

    /// Human-readable display name.
    pub fn display(self) -> &'static str {
        match self {
            TaskStatus::Todo => "TODO",
            TaskStatus::Doing => "DOING",
            TaskStatus::Done => "DONE",
            TaskStatus::Blocked => "BLOCKED",
            TaskStatus::Canceled => "CANCELED",
        }
    }
}

impl Default for TaskStatus {
    fn default() -> Self {
        TaskStatus::Todo
    }
}

/// Task priority levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Priority {
    #[serde(rename = "P0")]
    P0, // Critical
    #[serde(rename = "P1")]
    P1, // High
    #[serde(rename = "P2")]
    P2, // Medium (default)
    #[serde(rename = "P3")]
    P3, // Low
}

impl Default for Priority {
    fn default() -> Self {
        Priority::P2
    }
}

impl Priority {
    /// Display priority as string.
    pub fn display(self) -> &'static str {
        match self {
            Priority::P0 => "P0",
            Priority::P1 => "P1",
            Priority::P2 => "P2",
            Priority::P3 => "P3",
        }
    }
}

/// Git suggestions for a task.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GitSuggestions {
    #[serde(default)]
    pub branch: String,
    #[serde(default)]
    pub commit_add: String,
    #[serde(default)]
    pub commit_start: String,
    #[serde(default)]
    pub commit_done: String,
}

impl GitSuggestions {
    /// Check if all suggestions are empty.
    pub fn is_empty(&self) -> bool {
        self.branch.is_empty()
            && self.commit_add.is_empty()
            && self.commit_start.is_empty()
            && self.commit_done.is_empty()
    }
}

/// A reference to another entity (log, report, commit).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskReference {
    pub kind: ReferenceKind,
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Type of reference.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReferenceKind {
    Log,
    Report,
    Commit,
}

/// A task in the tt system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Internal version for schema evolution.
    pub version: u32,

    /// Unique identifier: "tt-000001".
    pub id: String,

    /// Task title (required).
    pub title: String,

    /// Current status.
    pub status: TaskStatus,

    /// Timestamps (ISO-8601 date format: "2026-03-28").
    pub created_at: String,
    pub updated_at: String,

    /// Optional due date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due: Option<String>,

    /// When the task was started.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<String>,

    /// When the task was completed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub done_at: Option<String>,

    /// Task priority.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<Priority>,

    /// Tags for categorization.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<String>,

    /// Additional notes.
    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub notes: String,

    /// Reason if blocked.
    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub blocked_reason: String,

    /// Time estimate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimate: Option<String>,

    /// Git suggestions (auto-generated).
    #[serde(skip_serializing_if = "GitSuggestions::is_empty", default)]
    pub git_suggestions: GitSuggestions,

    /// References to logs/reports.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub refs: Vec<TaskReference>,

    /// File system path (not serialized).
    #[serde(skip)]
    pub file_path: String,
}

impl Task {
    /// Get the display status.
    pub fn status_display(&self) -> &'static str {
        self.status.display()
    }

    /// Get the display priority.
    pub fn priority_display(&self) -> Option<&'static str> {
        self.priority.map(|p| p.display())
    }
}

/// Builder for creating new tasks.
#[derive(Debug, Clone, Default)]
pub struct NewTask {
    pub title: String,
    pub project: String,
    pub due: Option<String>,
    pub priority: Option<Priority>,
    pub tags: Vec<String>,
    pub notes: Option<String>,
    pub estimate: Option<String>,
}

impl NewTask {
    /// Create a new task builder.
    pub fn builder(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            ..Default::default()
        }
    }

    /// Set the project.
    pub fn project(mut self, project: impl Into<String>) -> Self {
        self.project = project.into();
        self
    }

    /// Set the due date.
    pub fn due(mut self, due: impl Into<String>) -> Self {
        self.due = Some(due.into());
        self
    }

    /// Set the priority.
    pub fn priority(mut self, priority: Priority) -> Self {
        self.priority = Some(priority);
        self
    }

    /// Add a tag.
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Set the notes.
    pub fn notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }

    /// Set the estimate.
    pub fn estimate(mut self, estimate: impl Into<String>) -> Self {
        self.estimate = Some(estimate.into());
        self
    }

    /// Build the task with an auto-generated ID.
    pub fn build(self, id: u64) -> Task {
        let now = chrono::Local::now().format("%Y-%m-%d").to_string();
        let slug = slug::slugify(&self.title);

        Task {
            version: 1,
            id: format!("tt-{:06}", id),
            title: self.title.clone(),
            status: TaskStatus::Todo,
            created_at: now.clone(),
            updated_at: now,
            due: self.due,
            started_at: None,
            done_at: None,
            priority: self.priority,
            tags: self.tags,
            notes: self.notes.unwrap_or_default(),
            blocked_reason: String::new(),
            estimate: self.estimate,
            git_suggestions: GitSuggestions {
                branch: format!("{}/tt-{:06}-{}", self.project, id, slug),
                commit_add: format!("task(add): tt-{:06} {}", id, self.title),
                commit_start: format!("task(start): tt-{:06} {}", id, self.title),
                commit_done: format!("task(done): tt-{:06} {}", id, self.title),
            },
            refs: Vec::new(),
            file_path: String::new(), // Set by storage
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_transition_valid() {
        // todo → doing, blocked, canceled
        assert!(TaskStatus::Todo.can_transition_to(TaskStatus::Doing));
        assert!(TaskStatus::Todo.can_transition_to(TaskStatus::Blocked));
        assert!(TaskStatus::Todo.can_transition_to(TaskStatus::Canceled));

        // doing → done, blocked, canceled
        assert!(TaskStatus::Doing.can_transition_to(TaskStatus::Done));
        assert!(TaskStatus::Doing.can_transition_to(TaskStatus::Blocked));
        assert!(TaskStatus::Doing.can_transition_to(TaskStatus::Canceled));

        // blocked → doing, canceled
        assert!(TaskStatus::Blocked.can_transition_to(TaskStatus::Doing));
        assert!(TaskStatus::Blocked.can_transition_to(TaskStatus::Canceled));
    }

    #[test]
    fn test_status_transition_invalid() {
        // done is terminal
        assert!(!TaskStatus::Done.can_transition_to(TaskStatus::Todo));
        assert!(!TaskStatus::Done.can_transition_to(TaskStatus::Doing));
        assert!(!TaskStatus::Done.can_transition_to(TaskStatus::Blocked));

        // canceled is terminal
        assert!(!TaskStatus::Canceled.can_transition_to(TaskStatus::Todo));
        assert!(!TaskStatus::Canceled.can_transition_to(TaskStatus::Doing));

        // invalid transitions
        assert!(!TaskStatus::Doing.can_transition_to(TaskStatus::Todo));
        assert!(!TaskStatus::Blocked.can_transition_to(TaskStatus::Todo));
        assert!(!TaskStatus::Blocked.can_transition_to(TaskStatus::Done));
    }

    #[test]
    fn test_new_task_builder() {
        let task = NewTask::builder("Test task")
            .project("work")
            .priority(Priority::P1)
            .tag("rust")
            .tag("cli")
            .due("2026-04-03")
            .estimate("2h")
            .build(1);

        assert_eq!(task.id, "tt-000001");
        assert_eq!(task.title, "Test task");
        assert_eq!(task.status, TaskStatus::Todo);
        assert_eq!(task.priority, Some(Priority::P1));
        assert_eq!(task.tags, vec!["rust", "cli"]);
        assert_eq!(task.due, Some("2026-04-03".to_string()));
        assert_eq!(task.estimate, Some("2h".to_string()));
        assert_eq!(task.git_suggestions.branch, "work/tt-000001-test-task");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(TaskStatus::Todo.display(), "TODO");
        assert_eq!(TaskStatus::Doing.display(), "DOING");
        assert_eq!(TaskStatus::Done.display(), "DONE");
        assert_eq!(TaskStatus::Blocked.display(), "BLOCKED");
        assert_eq!(TaskStatus::Canceled.display(), "CANCELED");
    }
}
