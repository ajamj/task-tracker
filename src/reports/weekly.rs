//! Weekly report generation with smart task mention merging.

use std::collections::HashMap;

use crate::error::{Result, TtError};
use crate::models::{Task, TaskStatus, WeekRange};
use crate::reports::templates::TemplateType;
use crate::storage::{Log, LogStorage, Project, TaskStorage};

/// Weekly report data with smart merging.
pub struct WeeklyReport {
    pub week: WeekRange,
    pub project: String,
    pub done: Vec<Task>,
    pub in_progress: Vec<Task>,
    pub blocked: Vec<Task>,
    pub mentioned: HashMap<String, Vec<String>>, // task_id -> [dates]
    pub missing: HashMap<String, Vec<String>>,   // task_id -> [dates] (no TOML file)
    pub highlights: Vec<HighlightDay>,
    /// Merged tasks: tasks with their mention dates from logs
    /// This enables "smart merging" where mentioned tasks show log dates
    pub merged_done: Vec<TaskWithMentions>,
    pub merged_in_progress: Vec<TaskWithMentions>,
    pub merged_blocked: Vec<TaskWithMentions>,
}

/// A task with its mention dates from logs (for smart merging)
#[derive(Clone, serde::Serialize)]
pub struct TaskWithMentions {
    pub task: Task,
    pub mention_dates: Vec<String>,
}

/// Highlights for a single day.
pub struct HighlightDay {
    pub date: String,
    pub items: Vec<String>,
}

/// Context data for template rendering with smart merging support.
#[derive(serde::Serialize)]
pub struct WeeklyReportContext<'a> {
    pub week: &'a WeekRange,
    pub project: &'a str,
    pub done_tasks: &'a Vec<Task>,
    pub in_progress_tasks: &'a Vec<Task>,
    pub blocked_tasks: &'a Vec<Task>,
    pub mentioned_tasks: &'a HashMap<String, Vec<String>>,
    pub missing_tasks: &'a HashMap<String, Vec<String>>,
    pub logs: Vec<LogContext<'a>>,
    /// Merged tasks with mention dates (for smart merging in templates)
    pub merged_done_tasks: Vec<TaskWithMentionsContext>,
    pub merged_in_progress_tasks: Vec<TaskWithMentionsContext>,
    pub merged_blocked_tasks: Vec<TaskWithMentionsContext>,
}

/// Task with mentions context for template rendering
#[derive(serde::Serialize)]
pub struct TaskWithMentionsContext {
    pub id: String,
    pub title: String,
    pub status: String,
    pub mention_dates: Vec<String>,
}

/// Log context for template rendering.
#[derive(serde::Serialize)]
pub struct LogContext<'a> {
    pub date: &'a str,
    pub highlights: Vec<String>,
}

impl WeeklyReport {
    /// Generate a weekly report.
    pub fn generate(
        task_storage: &TaskStorage,
        log_storage: &LogStorage,
        project: &Project,
        week: &WeekRange,
    ) -> Result<Self> {
        // Get all tasks
        let all_tasks = task_storage.list()
            .map_err(|e| TtError::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to list tasks: {}", e),
            )))?;

        // Categorize tasks
        let mut done = Vec::new();
        let mut in_progress = Vec::new();
        let mut blocked = Vec::new();

        for task in &all_tasks {
            match task.status {
                TaskStatus::Done => {
                    // Check if done_at is in this week
                    if let Some(ref done_at) = task.done_at {
                        if done_at >= &week.start.to_string() && done_at <= &week.end.to_string() {
                            done.push(task.clone());
                        }
                    }
                }
                TaskStatus::Doing => {
                    in_progress.push(task.clone());
                }
                TaskStatus::Blocked => {
                    blocked.push(task.clone());
                }
                _ => {}
            }
        }

        // Get logs for the week
        let logs = log_storage.get_for_date_range(
            &week.start.to_string(),
            &week.end.to_string(),
            &project.slug,
        )
        .map_err(|e| TtError::IoError(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to get logs: {}", e),
        )))?;

        // Build task ID map for existence checking
        let task_ids: HashMap<String, &Task> = all_tasks
            .iter()
            .map(|t| (t.id.clone(), t))
            .collect();

        // Scan logs for task IDs and build mentioned/missing maps
        let mut mentioned: HashMap<String, Vec<String>> = HashMap::new();
        let mut missing: HashMap<String, Vec<String>> = HashMap::new();

        for log in &logs {
            for task_id in &log.task_ids {
                mentioned
                    .entry(task_id.clone())
                    .or_insert_with(Vec::new)
                    .push(log.date.clone());

                // Check if task exists
                if !task_ids.contains_key(task_id) {
                    missing
                        .entry(task_id.clone())
                        .or_insert_with(Vec::new)
                        .push(log.date.clone());
                }
            }
        }

        // Sort dates in mentioned/missing
        for dates in mentioned.values_mut() {
            dates.sort();
        }
        for dates in missing.values_mut() {
            dates.sort();
        }

        // Extract highlights
        let highlights = extract_highlights(&logs);

        // Smart merging: merge mentioned tasks into Done/In Progress/Blocked sections
        let merged_done = merge_tasks_with_mentions(&done, &mentioned);
        let merged_in_progress = merge_tasks_with_mentions(&in_progress, &mentioned);
        let merged_blocked = merge_tasks_with_mentions(&blocked, &mentioned);

        Ok(Self {
            week: week.clone(),
            project: project.slug.clone(),
            done,
            in_progress,
            blocked,
            mentioned,
            missing,
            highlights,
            merged_done,
            merged_in_progress,
            merged_blocked,
        })
    }

    /// Render the report as Markdown.
    pub fn render(&self) -> String {
        let mut lines = Vec::new();

        // Header
        lines.push(format!("# Weekly Report — {} ({})", self.week.iso_week, self.project));
        lines.push(format!("Range: {} to {}", self.week.start, self.week.end));
        lines.push(String::new());

        // Summary
        lines.push("## Summary".to_string());
        lines.push(format!("- Done (by done_at): {}", self.done.len()));
        lines.push(format!("- In progress (current status): {}", self.in_progress.len()));
        lines.push(format!("- Blocked (current status): {}", self.blocked.len()));
        lines.push(format!("- Mentioned in logs: {}", self.mentioned.len()));
        lines.push(format!("- Missing tasks referenced in logs: {}", self.missing.len()));
        lines.push(String::new());

        // Done section
        lines.push("## Done".to_string());
        if self.done.is_empty() {
            lines.push("- (none)".to_string());
        } else {
            for task in &self.done {
                lines.push(format!("- {} — {}", task.id, task.title));
            }
        }
        lines.push(String::new());

        // In Progress section
        lines.push("## In Progress".to_string());
        if self.in_progress.is_empty() {
            lines.push("- (none)".to_string());
        } else {
            for task in &self.in_progress {
                lines.push(format!("- {} — {}", task.id, task.title));
            }
        }
        lines.push(String::new());

        // Blocked section
        lines.push("## Blocked".to_string());
        if self.blocked.is_empty() {
            lines.push("- (none)".to_string());
        } else {
            for task in &self.blocked {
                if task.blocked_reason.is_empty() {
                    lines.push(format!("- {} — {}", task.id, task.title));
                } else {
                    lines.push(format!("- {} — {} — ({})", task.id, task.title, task.blocked_reason));
                }
            }
        }
        lines.push(String::new());

        // Mentioned in Logs section
        lines.push("## Mentioned in Logs".to_string());
        if self.mentioned.is_empty() {
            lines.push("- (none)".to_string());
        } else {
            let mut sorted: Vec<_> = self.mentioned.iter().collect();
            sorted.sort_by(|a, b| a.0.cmp(b.0));
            for (task_id, dates) in sorted {
                lines.push(format!("- {} — Mentioned on: {}", task_id, dates.join(", ")));
            }
        }
        lines.push(String::new());

        // Missing tasks section
        lines.push("## Missing tasks referenced in logs".to_string());
        if self.missing.is_empty() {
            lines.push("- (none)".to_string());
        } else {
            let mut sorted: Vec<_> = self.missing.iter().collect();
            sorted.sort_by(|a, b| a.0.cmp(b.0));
            for (task_id, dates) in sorted {
                lines.push(format!("- {} — Mentioned on: {}", task_id, dates.join(", ")));
            }
        }
        lines.push(String::new());

        // Worklog Highlights section
        lines.push("## Worklog Highlights".to_string());
        if self.highlights.is_empty() {
            lines.push("- (no logs found in this range)".to_string());
        } else {
            for day in &self.highlights {
                lines.push(format!("### {}", day.date));
                if day.items.is_empty() {
                    lines.push("- (no highlights)".to_string());
                } else {
                    for item in &day.items {
                        lines.push(format!("- {}", item));
                    }
                }
            }
        }

        lines.join("\n")
    }

    /// Build the template context for this report with smart merging support.
    fn build_context(&self) -> WeeklyReportContext<'_> {
        let logs = self.highlights
            .iter()
            .map(|day| LogContext {
                date: &day.date,
                highlights: day.items.clone(),
            })
            .collect();

        // Build merged task contexts
        let merged_done_tasks = self.merged_done
            .iter()
            .map(|t| {
                let status_str = t.task.status.to_string();
                TaskWithMentionsContext {
                    id: t.task.id.clone(),
                    title: t.task.title.clone(),
                    status: status_str,
                    mention_dates: t.mention_dates.clone(),
                }
            })
            .collect();

        let merged_in_progress_tasks = self.merged_in_progress
            .iter()
            .map(|t| {
                let status_str = t.task.status.to_string();
                TaskWithMentionsContext {
                    id: t.task.id.clone(),
                    title: t.task.title.clone(),
                    status: status_str,
                    mention_dates: t.mention_dates.clone(),
                }
            })
            .collect();

        let merged_blocked_tasks = self.merged_blocked
            .iter()
            .map(|t| {
                let status_str = t.task.status.to_string();
                TaskWithMentionsContext {
                    id: t.task.id.clone(),
                    title: t.task.title.clone(),
                    status: status_str,
                    mention_dates: t.mention_dates.clone(),
                }
            })
            .collect();

        WeeklyReportContext {
            week: &self.week,
            project: &self.project,
            done_tasks: &self.done,
            in_progress_tasks: &self.in_progress,
            blocked_tasks: &self.blocked,
            mentioned_tasks: &self.mentioned,
            missing_tasks: &self.missing,
            logs,
            merged_done_tasks,
            merged_in_progress_tasks,
            merged_blocked_tasks,
        }
    }

    /// Render the report using a template.
    ///
    /// # Arguments
    /// * `template_content` - Template string to use
    ///
    /// # Returns
    /// * `Ok(String)` - Rendered report
    /// * `Err(TtError)` - Template rendering error
    pub fn render_with_template(&self, template_content: &str) -> Result<String> {
        let ctx = self.build_context();
        crate::reports::templates::render_template(template_content, &ctx)
    }

    /// Render the report using templates from workspace.
    ///
    /// # Arguments
    /// * `workspace_root` - Root directory of the workspace
    /// * `config` - Workspace configuration
    ///
    /// # Returns
    /// * `Ok(String)` - Rendered report
    /// * `Err(TtError)` - Template loading/rendering error
    pub fn render_from_workspace(
        &self,
        workspace_root: &std::path::Path,
        config: &crate::models::config::WorkspaceConfig,
    ) -> Result<String> {
        let ctx = self.build_context();
        crate::reports::templates::load_and_render_template(
            workspace_root,
            config,
            TemplateType::WeeklyReport,
            &ctx,
        )
    }
}

/// Merge tasks with their mention dates from logs (smart merging)
fn merge_tasks_with_mentions(
    tasks: &[Task],
    mentioned: &HashMap<String, Vec<String>>,
) -> Vec<TaskWithMentions> {
    tasks
        .iter()
        .map(|task| {
            let mention_dates = mentioned
                .get(&task.id)
                .cloned()
                .unwrap_or_default();

            TaskWithMentions {
                task: task.clone(),
                mention_dates,
            }
        })
        .collect()
}

/// Extract highlights from logs.
fn extract_highlights(logs: &[Log]) -> Vec<HighlightDay> {
    let mut highlights = Vec::new();

    for log in logs {
        let items = extract_highlights_from_log(&log.content);
        highlights.push(HighlightDay {
            date: log.date.clone(),
            items,
        });
    }

    highlights
}

/// Extract highlights from a single log file.
fn extract_highlights_from_log(content: &str) -> Vec<String> {
    let mut items = Vec::new();
    let mut in_section = false;
    let mut current_section = String::new();

    // Eligible sections for highlights
    let eligible_sections = ["highlights", "done", "doing", "blocked", "notes"];

    for line in content.lines() {
        let trimmed = line.trim();

        // Check for section headers (## Section Name)
        if trimmed.starts_with("##") {
            let section_name = trimmed.trim_start_matches('#').trim().to_lowercase();
            current_section = section_name;
            in_section = eligible_sections.iter().any(|s| current_section.contains(*s));
            continue;
        }

        // If in eligible section, collect bullet items
        if in_section {
            // Check for bullet points
            if trimmed.starts_with('-') || trimmed.starts_with('*') {
                let item = trimmed.trim_start_matches('-').trim_start_matches('*').trim();

                // Skip empty bullets
                if !item.is_empty() {
                    items.push(item.to_string());
                }
            }
        }
    }

    // Limit items per day (configurable in future)
    items.truncate(10);

    items
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_highlights_from_log() {
        let content = r#"
# 2026-03-28 (work)

## Highlights
- tt-000001: Initial implementation
- Completed the first pass

## Done
- Some completed task

## Notes
- Random note
"#;

        let items = extract_highlights_from_log(content);

        assert_eq!(items.len(), 4);
        assert!(items.contains(&"tt-000001: Initial implementation".to_string()));
        assert!(items.contains(&"Completed the first pass".to_string()));
    }

    #[test]
    fn test_extract_highlights_limits() {
        // Create content with many highlights
        let mut content = String::from("# 2026-03-28 (work)\n\n## Highlights\n");
        for i in 0..15 {
            content.push_str(&format!("- Item {}\n", i));
        }

        let items = extract_highlights_from_log(&content);

        assert_eq!(items.len(), 10); // Limited to 10
    }
}
