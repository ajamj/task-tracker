//! Output formatting and git suggestions.

use crate::models::Task;

/// Git suggestions for a command.
pub struct GitSuggestion {
    pub branch: Option<String>,
    pub commit_message: String,
    pub files_changed: Vec<String>,
}

impl GitSuggestion {
    /// Create git suggestion for task add.
    pub fn for_task_add(task: &Task, _project: &str) -> Self {
        Self {
            branch: Some(task.git_suggestions.branch.clone()),
            commit_message: task.git_suggestions.commit_add.clone(),
            files_changed: vec![task.file_path.clone()],
        }
    }

    /// Create git suggestion for task start.
    pub fn for_task_start(task: &Task, _project: &str) -> Self {
        Self {
            branch: Some(task.git_suggestions.branch.clone()),
            commit_message: task.git_suggestions.commit_start.clone(),
            files_changed: vec![task.file_path.clone()],
        }
    }

    /// Create git suggestion for task done.
    pub fn for_task_done(task: &Task, _project: &str) -> Self {
        Self {
            branch: Some(task.git_suggestions.branch.clone()),
            commit_message: task.git_suggestions.commit_done.clone(),
            files_changed: vec![task.file_path.clone()],
        }
    }

    /// Create git suggestion for log entry.
    pub fn for_log(date: &str, project: &str, log_path: &str) -> Self {
        Self {
            branch: None,
            commit_message: format!("log(add): {} ({})", date, project),
            files_changed: vec![log_path.to_string()],
        }
    }

    /// Create git suggestion for weekly report.
    pub fn for_report(week: &str, project: &str, report_path: &str) -> Self {
        Self {
            branch: None,
            commit_message: format!("report(week): {} ({})", week, project),
            files_changed: vec![report_path.to_string()],
        }
    }

    /// Create git suggestion for init.
    pub fn for_init() -> Self {
        Self {
            branch: None,
            commit_message: "chore(init): bootstrap worklog".to_string(),
            files_changed: vec![
                "tt.toml".to_string(),
                "projects/".to_string(),
            ],
        }
    }

    /// Display git suggestions.
    pub fn display(&self) {
        eprintln!();
        eprintln!("{}", style("Git suggestions (not executed):", "yellow", true));

        if let Some(ref branch) = self.branch {
            eprintln!("  Suggested branch: {}", style(branch, "cyan", false));
        }

        eprintln!("  Suggested commit: {}", style(&self.commit_message, "cyan", false));
        eprintln!("  Files changed:");

        for file in &self.files_changed {
            eprintln!("    - {}", file);
        }

        eprintln!();
        eprintln!("  Run these commands:");
        eprintln!("    git add -A");

        if let Some(ref branch) = self.branch {
            eprintln!("    git checkout -b {}", branch);
        }

        eprintln!("    git commit -m \"{}\"", self.commit_message);
    }
}

/// Simple style function for colored output.
fn style(text: &str, color: &str, bold: bool) -> String {
    // Check if we should use colors (not on Windows without ANSI support)
    let use_colors = std::env::var("NO_COLOR").is_err();

    if !use_colors {
        return text.to_string();
    }

    let (prefix, _suffix) = match color {
        "yellow" => ("\x1b[33m", "\x1b[0m"),
        "cyan" => ("\x1b[36m", "\x1b[0m"),
        "green" => ("\x1b[32m", "\x1b[0m"),
        "red" => ("\x1b[31m", "\x1b[0m"),
        _ => ("", ""),
    };

    let bold_prefix = if bold { "\x1b[1m" } else { "" };
    let bold_suffix = if bold { "\x1b[0m" } else { "" };

    format!("{}{}{}{}", prefix, bold_prefix, text, bold_suffix)
}

/// Format a task for display.
pub fn format_task(task: &Task) -> String {
    let mut lines = Vec::new();

    lines.push(format!("ID:        {}", task.id));
    lines.push(format!("Title:     {}", task.title));
    lines.push(format!("Status:    {}", task.status_display()));
    lines.push(format!("Created:   {}", task.created_at));
    lines.push(format!("Updated:   {}", task.updated_at));

    if let Some(ref due) = task.due {
        lines.push(format!("Due:       {}", due));
    }

    if let Some(ref started_at) = task.started_at {
        lines.push(format!("Started:   {}", started_at));
    }

    if let Some(ref done_at) = task.done_at {
        lines.push(format!("Done:      {}", done_at));
    }

    if let Some(priority) = task.priority {
        lines.push(format!("Priority:  {}", priority.display()));
    }

    if !task.tags.is_empty() {
        lines.push(format!("Tags:      {}", task.tags.join(", ")));
    }

    if !task.notes.is_empty() {
        lines.push(format!("Notes:     {}", task.notes.replace('\n', "\n             ")));
    }

    if !task.blocked_reason.is_empty() {
        lines.push(format!("Blocked:   {}", task.blocked_reason));
    }

    if let Some(ref estimate) = task.estimate {
        lines.push(format!("Estimate:  {}", estimate));
    }

    lines.join("\n")
}

/// Format tasks as a list grouped by status.
pub fn format_task_list(tasks: &[Task]) -> String {
    use crate::models::TaskStatus;

    let mut output = Vec::new();

    // Group by status
    let mut by_status = std::collections::HashMap::new();
    for task in tasks {
        by_status
            .entry(task.status)
            .or_insert_with(Vec::new)
            .push(task);
    }

    // Display in order: DOING, TODO, BLOCKED, DONE, CANCELED
    let status_order = [
        TaskStatus::Doing,
        TaskStatus::Todo,
        TaskStatus::Blocked,
        TaskStatus::Done,
        TaskStatus::Canceled,
    ];

    for status in status_order {
        if let Some(status_tasks) = by_status.get(&status) {
            if status_tasks.is_empty() {
                continue;
            }

            output.push(format!("{}:", style(status.display(), "green", true)));

            for task in status_tasks {
                let due_str = task
                    .due
                    .as_ref()
                    .map(|d| format!("  due {}", d))
                    .unwrap_or_default();

                let priority_str = task
                    .priority
                    .map(|p| format!("  {}", p.display()))
                    .unwrap_or_default();

                let tags_str = if !task.tags.is_empty() {
                    format!("  tags: {}", task.tags.join(","))
                } else {
                    String::new()
                };

                let mut line = format!(
                    "  {:12}  {:30}{}{}{}",
                    task.id,
                    truncate(&task.title, 30),
                    if !due_str.is_empty() { &due_str } else { "" },
                    if !priority_str.is_empty() { &priority_str } else { "" },
                    if !tags_str.is_empty() { &tags_str } else { "" },
                );

                // Trim trailing whitespace
                line = line.trim_end().to_string();

                output.push(line);
            }

            output.push(String::new());
        }
    }

    if output.is_empty() {
        output.push("No tasks found.".to_string());
    }

    output.join("\n")
}

/// Truncate text to max length with ellipsis.
fn truncate(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        text.to_string()
    } else {
        format!("{}...", &text[..max_len - 3])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("short", 10), "short");
        assert_eq!(truncate("exactly ten", 10), "exactly...");
        assert_eq!(truncate("this is a long text", 10), "this is...");
    }

    #[test]
    fn test_format_task() {
        use crate::models::{NewTask, Priority};

        let task = NewTask::builder("Test task")
            .project("work")
            .priority(Priority::P1)
            .tag("rust")
            .due("2026-04-03")
            .notes("Some notes")
            .build(1);

        let formatted = format_task(&task);

        assert!(formatted.contains("ID:        tt-000001"));
        assert!(formatted.contains("Title:     Test task"));
        assert!(formatted.contains("Priority:  P1"));
        assert!(formatted.contains("Due:       2026-04-03"));
        assert!(formatted.contains("Tags:      rust"));
    }
}
