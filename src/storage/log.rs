//! Log file I/O and template management.

use std::fs;
use std::path::PathBuf;

use crate::error::{StorageError, StorageResult};

/// Default daily log template.
const DEFAULT_LOG_TEMPLATE: &str = r#"# {{date}} ({{project}})

## Highlights
- 

## Done
- 

## Doing
- 

## Blocked
- 

## Notes
- 
"#;

/// Log entry for a specific date.
#[derive(Debug, Clone)]
pub struct Log {
    /// Date (YYYY-MM-DD).
    pub date: String,

    /// Project slug.
    pub project: String,

    /// Raw markdown content.
    pub content: String,

    /// File system path.
    pub file_path: PathBuf,

    /// Extracted task IDs (computed).
    pub task_ids: Vec<String>,
}

impl Log {
    /// Create a new log with default template.
    pub fn new(date: &str, project: &str) -> Self {
        let content = DEFAULT_LOG_TEMPLATE
            .replace("{{date}}", date)
            .replace("{{project}}", project);

        Self {
            date: date.to_string(),
            project: project.to_string(),
            content,
            file_path: PathBuf::new(),
            task_ids: Vec::new(),
        }
    }

    /// Append text to the log content.
    pub fn append(&mut self, text: &str) {
        self.content.push('\n');
        self.content.push_str(text);
    }

    /// Scan for task IDs in the content.
    pub fn scan_task_ids(&mut self) -> Vec<String> {
        self.task_ids = scan_for_task_ids(&self.content);
        self.task_ids.clone()
    }
}

/// Log storage manager.
pub struct LogStorage {
    /// Project logs directory.
    logs_dir: PathBuf,
}

impl LogStorage {
    /// Create a new log storage instance.
    pub fn new(logs_dir: PathBuf) -> Self {
        Self { logs_dir }
    }

    /// Get or create a log for a specific date.
    pub fn get_or_create(&self, date: &str, project: &str) -> StorageResult<Log> {
        let log_path = self.log_path(date);

        // Ensure parent directory exists (YYYY/)
        if let Some(parent) = log_path.parent() {
            fs::create_dir_all(parent)
                .map_err(StorageError::IoError)?;
        }

        if log_path.exists() {
            self.load(date, project)
        } else {
            self.create(date, project)
        }
    }

    /// Create a new log file.
    pub fn create(&self, date: &str, project: &str) -> StorageResult<Log> {
        let mut log = Log::new(date, project);
        log.file_path = self.log_path(date);

        fs::write(&log.file_path, &log.content)
            .map_err(StorageError::IoError)?;

        Ok(log)
    }

    /// Load an existing log file.
    pub fn load(&self, date: &str, project: &str) -> StorageResult<Log> {
        let log_path = self.log_path(date);

        if !log_path.exists() {
            return Err(StorageError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Log file not found: {}", log_path.display()),
            )));
        }

        let content = fs::read_to_string(&log_path)
            .map_err(StorageError::IoError)?;

        let mut log = Log {
            date: date.to_string(),
            project: project.to_string(),
            content,
            file_path: log_path,
            task_ids: Vec::new(),
        };

        // Scan for task IDs on load
        log.scan_task_ids();

        Ok(log)
    }

    /// Append text to a log file.
    pub fn append(&self, date: &str, project: &str, text: &str) -> StorageResult<Log> {
        let mut log = self.get_or_create(date, project)?;
        log.append(text);
        log.scan_task_ids();

        fs::write(&log.file_path, &log.content)
            .map_err(StorageError::IoError)?;

        Ok(log)
    }

    /// Get logs for a date range.
    pub fn get_for_date_range(
        &self,
        start_date: &str,
        end_date: &str,
        project: &str,
    ) -> StorageResult<Vec<Log>> {
        let mut logs = Vec::new();

        if !self.logs_dir.exists() {
            return Ok(logs);
        }

        // Walk through year directories
        for year_entry in fs::read_dir(&self.logs_dir)
            .map_err(StorageError::IoError)?
        {
            let year_entry = year_entry.map_err(StorageError::IoError)?;
            let year_path = year_entry.path();

            if !year_path.is_dir() {
                continue;
            }

            // Walk through log files in year directory
            for log_entry in fs::read_dir(&year_path)
                .map_err(StorageError::IoError)?
            {
                let log_entry = log_entry.map_err(StorageError::IoError)?;
                let log_path = log_entry.path();

                if log_path.extension().and_then(|s| s.to_str()) != Some("md") {
                    continue;
                }

                // Extract date from filename
                if let Some(file_stem) = log_path.file_stem().and_then(|s| s.to_str()) {
                    // Check if date is in range
                    if file_stem >= start_date && file_stem <= end_date {
                        if let Ok(log) = self.load(file_stem, project) {
                            logs.push(log);
                        }
                    }
                }
            }
        }

        // Sort by date
        logs.sort_by(|a, b| a.date.cmp(&b.date));

        Ok(logs)
    }

    /// Get the file path for a log date.
    pub fn log_path(&self, date: &str) -> PathBuf {
        // date format: YYYY-MM-DD
        if let Some(year) = date.get(0..4) {
            self.logs_dir.join(year).join(format!("{}.md", date))
        } else {
            self.logs_dir.join(format!("{}.md", date))
        }
    }

    /// Check if a log exists for a date.
    pub fn exists(&self, date: &str) -> bool {
        self.log_path(date).exists()
    }
}

/// Scan content for task IDs matching the pattern tt-XXXXXX.
pub fn scan_for_task_ids(content: &str) -> Vec<String> {
    use regex::Regex;
    use std::collections::HashSet;

    // Compile regex once per call (could be optimized with once_cell)
    let task_id_regex = Regex::new(r"(?i)\btt-\d{6}\b").unwrap();

    let mut ids: HashSet<String> = HashSet::new();

    for mat in task_id_regex.find_iter(content) {
        // Normalize to lowercase
        ids.insert(mat.as_str().to_lowercase());
    }

    let mut ids: Vec<String> = ids.into_iter().collect();
    ids.sort();
    ids
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_storage() -> (TempDir, LogStorage) {
        let temp_dir = TempDir::new().unwrap();
        let logs_dir = temp_dir.path().join("logs");
        fs::create_dir_all(&logs_dir).unwrap();
        let storage = LogStorage::new(logs_dir);
        (temp_dir, storage)
    }

    #[test]
    fn test_log_new_with_template() {
        let log = Log::new("2026-03-28", "work");

        assert!(log.content.contains("# 2026-03-28 (work)"));
        assert!(log.content.contains("## Highlights"));
        assert!(log.content.contains("## Done"));
        assert!(log.content.contains("## Doing"));
        assert!(log.content.contains("## Blocked"));
        assert!(log.content.contains("## Notes"));
    }

    #[test]
    fn test_log_append() {
        let mut log = Log::new("2026-03-28", "work");
        log.append("Worked on tt-000001");

        assert!(log.content.contains("Worked on tt-000001"));
    }

    #[test]
    fn test_scan_for_task_ids() {
        let content = r#"
# 2026-03-28

## Done
- tt-000001: Completed task
- tt-000002: Another task

## Notes
Worked on tt-000001 and tt-000003.
Also mentioned TT-000001 (uppercase).
"#;

        let ids = scan_for_task_ids(content);

        assert_eq!(ids.len(), 3);
        assert!(ids.contains(&"tt-000001".to_string()));
        assert!(ids.contains(&"tt-000002".to_string()));
        assert!(ids.contains(&"tt-000003".to_string()));
    }

    #[test]
    fn test_scan_for_task_ids_deduplicates() {
        let content = "tt-000001 tt-000001 tt-000001";

        let ids = scan_for_task_ids(content);

        assert_eq!(ids.len(), 1);
        assert_eq!(ids[0], "tt-000001");
    }

    #[test]
    fn test_log_storage_get_or_create() {
        let (_temp_dir, storage) = create_test_storage();

        let log = storage.get_or_create("2026-03-28", "work").unwrap();

        assert!(storage.exists("2026-03-28"));
        assert!(log.content.contains("# 2026-03-28 (work)"));
    }

    #[test]
    fn test_log_storage_append() {
        let (_temp_dir, storage) = create_test_storage();

        let mut log = storage.append("2026-03-28", "work", "Worked on tt-000001").unwrap();
        log.scan_task_ids();

        assert!(log.content.contains("Worked on tt-000001"));
        assert!(log.task_ids.contains(&"tt-000001".to_string()));
    }

    #[test]
    fn test_log_storage_load_existing() {
        let (_temp_dir, storage) = create_test_storage();

        // Create log
        storage.append("2026-03-28", "work", "Initial entry").unwrap();

        // Load it back
        let log = storage.load("2026-03-28", "work").unwrap();

        assert!(log.content.contains("Initial entry"));
        assert_eq!(log.date, "2026-03-28");
        assert_eq!(log.project, "work");
    }

    #[test]
    fn test_log_storage_get_for_date_range() {
        let (_temp_dir, storage) = create_test_storage();

        // Create logs for multiple dates
        storage.append("2026-03-26", "work", "Day 1").unwrap();
        storage.append("2026-03-27", "work", "Day 2").unwrap();
        storage.append("2026-03-28", "work", "Day 3").unwrap();
        storage.append("2026-03-29", "work", "Day 4").unwrap();

        // Get logs for range
        let logs = storage.get_for_date_range("2026-03-27", "2026-03-28", "work").unwrap();

        assert_eq!(logs.len(), 2);
        assert_eq!(logs[0].date, "2026-03-27");
        assert_eq!(logs[1].date, "2026-03-28");
    }

    #[test]
    fn test_log_path_format() {
        let (_temp_dir, storage) = create_test_storage();

        let path = storage.log_path("2026-03-28");

        assert_eq!(
            path,
            storage.logs_dir.join("2026").join("2026-03-28.md")
        );
    }
}
