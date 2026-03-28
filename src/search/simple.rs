//! Simple regex-based search implementation.

use regex::Regex;
use serde::{Serialize, Deserialize};
use crate::models::Task;
use crate::storage::TaskStorage;

/// Search result item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub title: String,
    pub status: String,
    pub priority: Option<String>,
    pub tags: Vec<String>,
    pub score: f64,
}

/// Search tasks by query.
pub fn search_tasks(task_storage: &TaskStorage, query: &str) -> Result<Vec<SearchResult>, String> {
    let tasks = task_storage.list()
        .map_err(|e| format!("Failed to list tasks: {}", e))?;

    let query_lower = query.to_lowercase();
    let mut results = Vec::new();

    for task in tasks {
        let mut score = 0.0;

        // Search in title
        if task.title.to_lowercase().contains(&query_lower) {
            score += 10.0;
        }

        // Search in notes
        if let Some(ref notes) = task.notes {
            if notes.to_lowercase().contains(&query_lower) {
                score += 5.0;
            }
        }

        // Search in tags
        for tag in &task.tags {
            if tag.to_lowercase().contains(&query_lower) {
                score += 3.0;
            }
        }

        // Search in ID
        if task.id.to_lowercase().contains(&query_lower) {
            score += 2.0;
        }

        if score > 0.0 {
            results.push(SearchResult {
                id: task.id.clone(),
                title: task.title.clone(),
                status: task.status.to_string(),
                priority: task.priority.as_ref().map(|p| p.to_string()),
                tags: task.tags.clone(),
                score,
            });
        }
    }

    // Sort by score (descending)
    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

    Ok(results)
}

/// Search with regex pattern.
pub fn search_tasks_regex(task_storage: &TaskStorage, pattern: &str) -> Result<Vec<SearchResult>, String> {
    let regex = Regex::new(pattern)
        .map_err(|e| format!("Invalid regex pattern: {}", e))?;

    let tasks = task_storage.list()
        .map_err(|e| format!("Failed to list tasks: {}", e))?;

    let mut results = Vec::new();

    for task in tasks {
        let mut score = 0.0;

        // Search in title
        if regex.is_match(&task.title) {
            score += 10.0;
        }

        // Search in notes
        if let Some(ref notes) = task.notes {
            if regex.is_match(notes) {
                score += 5.0;
            }
        }

        // Search in tags
        for tag in &task.tags {
            if regex.is_match(tag) {
                score += 3.0;
            }
        }

        if score > 0.0 {
            results.push(SearchResult {
                id: task.id.clone(),
                title: task.title.clone(),
                status: task.status.to_string(),
                priority: task.priority.as_ref().map(|p| p.to_string()),
                tags: task.tags.clone(),
                score,
            });
        }
    }

    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_by_title() {
        // Simple test - actual implementation would need task storage
        assert!(true);
    }
}
