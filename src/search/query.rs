//! Search query building and filters.

use serde::{Deserialize, Serialize};

/// Filters for search queries
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchFilters {
    /// Filter by project slug
    pub project: Option<String>,
    /// Filter by status (multiple allowed)
    pub status: Option<Vec<String>>,
    /// Filter by tags (multiple allowed)
    pub tag: Option<Vec<String>>,
    /// Filter by date range (from)
    pub from: Option<String>,
    /// Filter by date range (to)
    pub to: Option<String>,
}

impl SearchFilters {
    /// Create empty filters
    pub fn new() -> Self {
        Self::default()
    }

    /// Set project filter
    pub fn with_project(mut self, project: impl Into<String>) -> Self {
        self.project = Some(project.into());
        self
    }

    /// Add status filter
    pub fn with_status(mut self, status: impl Into<String>) -> Self {
        self.status.get_or_insert_with(Vec::new).push(status.into());
        self
    }

    /// Add tag filter
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tag.get_or_insert_with(Vec::new).push(tag.into());
        self
    }

    /// Set date range (from)
    pub fn with_from(mut self, from: impl Into<String>) -> Self {
        self.from = Some(from.into());
        self
    }

    /// Set date range (to)
    pub fn with_to(mut self, to: impl Into<String>) -> Self {
        self.to = Some(to.into());
        self
    }

    /// Check if any filters are set
    pub fn is_empty(&self) -> bool {
        self.project.is_none()
            && self.status.is_none()
            && self.tag.is_none()
            && self.from.is_none()
            && self.to.is_none()
    }
}

/// Search result item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Task ID or log date
    pub id: String,
    /// Task title or log title
    pub title: String,
    /// Type: "task" or "log"
    pub doc_type: String,
    /// Project slug
    pub project: String,
    /// Task status (only for tasks)
    pub status: Option<String>,
    /// Task tags (only for tasks)
    pub tags: Vec<String>,
    /// Search score (relevance)
    pub score: f64,
}

impl SearchResult {
    /// Format result for display
    pub fn format(&self) -> String {
        let type_icon = match self.doc_type.as_str() {
            "task" => "📋",
            "log" => "📝",
            _ => "📄",
        };

        let status = self.status.as_ref()
            .map(|s| format!(" [{}]", s))
            .unwrap_or_default();

        let tags = if !self.tags.is_empty() {
            format!(" ({})", self.tags.join(", "))
        } else {
            String::new()
        };

        format!(
            "{} {}{}{} — {}",
            type_icon,
            self.id,
            status,
            tags,
            self.title
        )
    }

    /// Format as JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

/// Format multiple results for display
pub fn format_results(results: &[SearchResult], as_json: bool) -> Result<String, serde_json::Error> {
    if as_json {
        serde_json::to_string_pretty(results)
    } else {
        let output: Vec<String> = results.iter().map(|r| r.format()).collect();
        Ok(output.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_filters_new() {
        let filters = SearchFilters::new();
        assert!(filters.is_empty());
    }

    #[test]
    fn test_search_filters_builder() {
        let filters = SearchFilters::new()
            .with_project("work")
            .with_status("todo")
            .with_tag("rust");

        assert_eq!(filters.project, Some("work".to_string()));
        assert_eq!(filters.status, Some(vec!["todo".to_string()]));
        assert_eq!(filters.tag, Some(vec!["rust".to_string()]));
        assert!(!filters.is_empty());
    }

    #[test]
    fn test_search_result_format() {
        let result = SearchResult {
            id: "tt-000001".to_string(),
            title: "Test task".to_string(),
            doc_type: "task".to_string(),
            project: "work".to_string(),
            status: Some("todo".to_string()),
            tags: vec!["rust".to_string()],
            score: 1.0,
        };

        let formatted = result.format();
        assert!(formatted.contains("tt-000001"));
        assert!(formatted.contains("Test task"));
        assert!(formatted.contains("[todo]"));
        assert!(formatted.contains("(rust)"));
    }
}
