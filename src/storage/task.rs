//! Task file I/O and ID generation with file locking.

use std::fs::{self, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use fs2::FileExt;
use toml_edit::{DocumentMut, value};

use crate::error::{StorageError, StorageResult};
use crate::models::{Task, TaskStatus};
use chrono::{Datelike, Local};

/// Task storage manager.
pub struct TaskStorage {
    /// Project tasks directory.
    tasks_dir: PathBuf,
    /// Counter file path for ID generation.
    counter_path: PathBuf,
}

impl TaskStorage {
    /// Create a new task storage instance.
    pub fn new(tasks_dir: PathBuf) -> Self {
        let counter_path = tasks_dir.join(".task_counter");
        Self {
            tasks_dir,
            counter_path,
        }
    }

    /// Get the next task ID with file locking.
    pub fn next_id(&self) -> StorageResult<u64> {
        // Open with read+write, create if not exists
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&self.counter_path)
            .map_err(|e| StorageError::IoError(e))?;

        // Acquire exclusive lock
        file.lock_exclusive()
            .map_err(|e| StorageError::LockError(e.to_string()))?;

        // Read current value
        let mut content = String::new();
        file.read_to_string(&mut content)
            .map_err(|e| StorageError::IoError(e))?;

        let current: u64 = content.trim().parse().unwrap_or(0);

        // Increment and write back
        let next = current + 1;
        file.seek(SeekFrom::Start(0))
            .map_err(|e| StorageError::IoError(e))?;
        file.write_all(next.to_string().as_bytes())
            .map_err(|e| StorageError::IoError(e))?;
        file.set_len(next.to_string().len() as u64)
            .map_err(|e| StorageError::IoError(e))?;

        // Release lock (automatic on drop, but explicit for clarity)
        file.unlock()
            .map_err(|e| StorageError::LockError(e.to_string()))?;

        Ok(next)
    }

    /// Create a new task file.
    pub fn create(&self, task: &Task) -> StorageResult<()> {
        let doc = task_to_document(task);
        fs::write(&task.file_path, doc.to_string())
            .map_err(|e| StorageError::IoError(e))?;
        Ok(())
    }

    /// Get a task by ID.
    pub fn get(&self, task_id: &str) -> StorageResult<Task> {
        let task_path = self.task_path(task_id);

        if !task_path.exists() {
            return Err(StorageError::TaskNotFound(task_id.to_string()));
        }

        let content = fs::read_to_string(&task_path)
            .map_err(|e| StorageError::IoError(e))?;

        parse_task(&content, task_path)
    }

    /// Update an existing task.
    pub fn update(&self, task: &Task) -> StorageResult<()> {
        let doc = task_to_document(task);
        fs::write(&task.file_path, doc.to_string())
            .map_err(|e| StorageError::IoError(e))?;
        Ok(())
    }

    /// List all tasks.
    pub fn list(&self) -> StorageResult<Vec<Task>> {
        let mut tasks = Vec::new();

        if !self.tasks_dir.exists() {
            return Ok(tasks);
        }

        // Recursively find all .toml files
        for entry in walk_dir(&self.tasks_dir) {
            let entry = entry.map_err(|e| StorageError::IoError(e))?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) != Some("toml") {
                continue;
            }

            // Skip counter file
            if path.file_name().and_then(|s| s.to_str()) == Some(".task_counter") {
                continue;
            }

            let content = fs::read_to_string(&path)
                .map_err(|e| StorageError::IoError(e))?;

            match parse_task(&content, path.clone()) {
                Ok(task) => tasks.push(task),
                Err(e) => {
                    // Log warning but continue
                    eprintln!("Warning: Failed to parse {:?}: {}", path, e);
                }
            }
        }

        // Sort by ID
        tasks.sort_by(|a, b| a.id.cmp(&b.id));

        Ok(tasks)
    }

    /// List tasks by status.
    pub fn list_by_status(&self, status: TaskStatus) -> StorageResult<Vec<Task>> {
        let tasks = self.list()?;
        Ok(tasks.into_iter().filter(|t| t.status == status).collect())
    }

    /// Get the file path for a task ID.
    pub fn task_path(&self, task_id: &str) -> PathBuf {
        // Extract year/month from existing tasks or use current date
        // For simplicity, we'll use current date for new tasks
        let now = Local::now();
        let year = now.year();
        let month = now.month();

        self.tasks_dir
            .join(format!("{:04}", year))
            .join(format!("{:02}", month))
            .join(format!("{}.toml", task_id))
    }

    /// Check if a task exists.
    pub fn exists(&self, task_id: &str) -> bool {
        self.task_path(task_id).exists()
    }
}

/// Convert a Task to a TOML document.
fn task_to_document(task: &Task) -> DocumentMut {
    let mut doc = DocumentMut::new();

    doc["version"] = value(task.version as i64);
    doc["id"] = value(&task.id);
    doc["title"] = value(&task.title);
    doc["status"] = value(task.status.display().to_lowercase());
    doc["created_at"] = value(&task.created_at);
    doc["updated_at"] = value(&task.updated_at);

    if let Some(ref due) = task.due {
        doc["due"] = value(due);
    }

    if let Some(ref started_at) = task.started_at {
        doc["started_at"] = value(started_at);
    }

    if let Some(ref done_at) = task.done_at {
        doc["done_at"] = value(done_at);
    }

    if let Some(ref priority) = task.priority {
        doc["priority"] = value(priority.display());
    }

    if !task.tags.is_empty() {
        let tags_array = toml_edit::Array::from_iter(task.tags.iter().cloned());
        doc["tags"] = value(toml_edit::Value::Array(tags_array));
    }

    if !task.notes.is_empty() {
        doc["notes"] = value(&task.notes);
    }

    if !task.blocked_reason.is_empty() {
        doc["blocked_reason"] = value(&task.blocked_reason);
    }

    if let Some(ref estimate) = task.estimate {
        doc["estimate"] = value(estimate);
    }

    // Git suggestions
    if !task.git_suggestions.is_empty() {
        let mut git_doc = DocumentMut::new();
        if !task.git_suggestions.branch.is_empty() {
            git_doc["branch"] = value(&task.git_suggestions.branch);
        }
        if !task.git_suggestions.commit_add.is_empty() {
            git_doc["commit_add"] = value(&task.git_suggestions.commit_add);
        }
        if !task.git_suggestions.commit_start.is_empty() {
            git_doc["commit_start"] = value(&task.git_suggestions.commit_start);
        }
        if !task.git_suggestions.commit_done.is_empty() {
            git_doc["commit_done"] = value(&task.git_suggestions.commit_done);
        }
        let git_table = git_doc.into_table();
        let mut git_item = toml_edit::Item::Table(git_table);
        git_item.as_table_mut().unwrap().set_implicit(false);
        doc["git_suggestions"] = git_item;
    }

    doc
}

/// Parse a Task from TOML content.
fn parse_task(content: &str, path: PathBuf) -> StorageResult<Task> {
    let doc = content.parse::<DocumentMut>()
        .map_err(|e| StorageError::TomlParseError(e))?;

    let id = doc.get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| StorageError::IdGenerationError("Missing 'id' field".to_string()))?
        .to_string();

    let title = doc.get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("Untitled")
        .to_string();

    let status_str = doc.get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("todo");

    let status = match status_str.to_lowercase().as_str() {
        "doing" => TaskStatus::Doing,
        "done" => TaskStatus::Done,
        "blocked" => TaskStatus::Blocked,
        "canceled" => TaskStatus::Canceled,
        _ => TaskStatus::Todo,
    };

    let created_at = doc.get("created_at")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let updated_at = doc.get("updated_at")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let due = doc.get("due").and_then(|v| v.as_str()).map(String::from);
    let started_at = doc.get("started_at").and_then(|v| v.as_str()).map(String::from);
    let done_at = doc.get("done_at").and_then(|v| v.as_str()).map(String::from);

    let priority = doc.get("priority")
        .and_then(|v| v.as_str())
        .and_then(|p| match p {
            "P0" => Some(crate::models::Priority::P0),
            "P1" => Some(crate::models::Priority::P1),
            "P2" => Some(crate::models::Priority::P2),
            "P3" => Some(crate::models::Priority::P3),
            _ => None,
        });

    let tags = doc.get("tags")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let notes = doc.get("notes")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let blocked_reason = doc.get("blocked_reason")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let estimate = doc.get("estimate")
        .and_then(|v| v.as_str())
        .map(String::from);

    // Parse git suggestions
    let mut git_suggestions = crate::models::task::GitSuggestions::default();
    if let Some(git_table) = doc.get("git_suggestions").and_then(|v| v.as_table()) {
        if let Some(branch) = git_table.get("branch").and_then(|v| v.as_str()) {
            git_suggestions.branch = branch.to_string();
        }
        if let Some(commit_add) = git_table.get("commit_add").and_then(|v| v.as_str()) {
            git_suggestions.commit_add = commit_add.to_string();
        }
        if let Some(commit_start) = git_table.get("commit_start").and_then(|v| v.as_str()) {
            git_suggestions.commit_start = commit_start.to_string();
        }
        if let Some(commit_done) = git_table.get("commit_done").and_then(|v| v.as_str()) {
            git_suggestions.commit_done = commit_done.to_string();
        }
    }

    Ok(Task {
        version: 1,
        id,
        title,
        status,
        created_at,
        updated_at,
        due,
        started_at,
        done_at,
        priority,
        tags,
        notes,
        blocked_reason,
        estimate,
        git_suggestions,
        refs: Vec::new(),
        file_path: path.to_string_lossy().to_string(),
    })
}

/// Walk directory recursively to find files.
fn walk_dir(dir: &Path) -> Vec<std::io::Result<std::fs::DirEntry>> {
    let mut entries = Vec::new();

    if let Ok(read_dir) = fs::read_dir(dir) {
        for entry in read_dir {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_dir() {
                    entries.extend(walk_dir(&path));
                } else {
                    entries.push(Ok(entry));
                }
            }
        }
    }

    entries
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::NewTask;
    use tempfile::TempDir;

    fn create_test_storage() -> (TempDir, TaskStorage) {
        let temp_dir = TempDir::new().unwrap();
        let tasks_dir = temp_dir.path().join("tasks");
        fs::create_dir_all(&tasks_dir).unwrap();
        let storage = TaskStorage::new(tasks_dir);
        (temp_dir, storage)
    }

    #[test]
    fn test_next_id_sequential() {
        let (_temp_dir, storage) = create_test_storage();

        let id1 = storage.next_id().unwrap();
        let id2 = storage.next_id().unwrap();
        let id3 = storage.next_id().unwrap();

        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert_eq!(id3, 3);
    }

    #[test]
    fn test_create_and_get_task() {
        let (_temp_dir, storage) = create_test_storage();

        let id = storage.next_id().unwrap();
        let mut task = NewTask::builder("Test task")
            .project("work")
            .priority(crate::models::Priority::P1)
            .build(id);

        // Set file path
        task.file_path = storage.task_path(&task.id).to_string_lossy().to_string();

        storage.create(&task).unwrap();

        let retrieved = storage.get(&task.id).unwrap();

        assert_eq!(retrieved.id, task.id);
        assert_eq!(retrieved.title, "Test task");
        assert_eq!(retrieved.status, TaskStatus::Todo);
        assert_eq!(retrieved.priority, Some(crate::models::Priority::P1));
    }

    #[test]
    fn test_update_task() {
        let (_temp_dir, storage) = create_test_storage();

        let id = storage.next_id().unwrap();
        let mut task = NewTask::builder("Test task")
            .project("work")
            .build(id);

        task.file_path = storage.task_path(&task.id).to_string_lossy().to_string();
        storage.create(&task).unwrap();

        // Update task
        let mut updated = storage.get(&task.id).unwrap();
        updated.status = TaskStatus::Doing;
        updated.started_at = Some("2026-03-28".to_string());
        updated.updated_at = "2026-03-28".to_string();

        storage.update(&updated).unwrap();

        let retrieved = storage.get(&task.id).unwrap();
        assert_eq!(retrieved.status, TaskStatus::Doing);
        assert_eq!(retrieved.started_at, Some("2026-03-28".to_string()));
    }

    #[test]
    fn test_list_tasks() {
        let (_temp_dir, storage) = create_test_storage();

        // Create multiple tasks
        for i in 1..=3 {
            let id = storage.next_id().unwrap();
            let mut task = NewTask::builder(format!("Task {}", i))
                .project("work")
                .build(id);
            task.file_path = storage.task_path(&task.id).to_string_lossy().to_string();
            storage.create(&task).unwrap();
        }

        let tasks = storage.list().unwrap();
        assert_eq!(tasks.len(), 3);
        assert_eq!(tasks[0].title, "Task 1");
        assert_eq!(tasks[1].title, "Task 2");
        assert_eq!(tasks[2].title, "Task 3");
    }

    #[test]
    fn test_list_by_status() {
        let (_temp_dir, storage) = create_test_storage();

        // Create tasks with different statuses
        let id1 = storage.next_id().unwrap();
        let mut task1 = NewTask::builder("Todo task")
            .project("work")
            .build(id1);
        task1.file_path = storage.task_path(&task1.id).to_string_lossy().to_string();
        storage.create(&task1).unwrap();

        let id2 = storage.next_id().unwrap();
        let mut task2 = NewTask::builder("Doing task")
            .project("work")
            .build(id2);
        task2.file_path = storage.task_path(&task2.id).to_string_lossy().to_string();
        storage.create(&task2).unwrap();

        // Update task2 to doing
        let mut updated = storage.get(&task2.id).unwrap();
        updated.status = TaskStatus::Doing;
        updated.updated_at = "2026-03-28".to_string();
        storage.update(&updated).unwrap();

        let todo_tasks = storage.list_by_status(TaskStatus::Todo).unwrap();
        let doing_tasks = storage.list_by_status(TaskStatus::Doing).unwrap();

        assert_eq!(todo_tasks.len(), 1);
        assert_eq!(doing_tasks.len(), 1);
        assert_eq!(todo_tasks[0].title, "Todo task");
        assert_eq!(doing_tasks[0].title, "Doing task");
    }

    #[test]
    fn test_get_nonexistent_task() {
        let (_temp_dir, storage) = create_test_storage();

        let result = storage.get("tt-999999");
        assert!(result.is_err());
        match result {
            Err(StorageError::TaskNotFound(id)) => assert_eq!(id, "tt-999999"),
            _ => panic!("Expected TaskNotFound"),
        }
    }

    #[test]
    fn test_task_exists() {
        let (_temp_dir, storage) = create_test_storage();

        let id = storage.next_id().unwrap();
        let mut task = NewTask::builder("Test")
            .project("work")
            .build(id);
        task.file_path = storage.task_path(&task.id).to_string_lossy().to_string();
        storage.create(&task).unwrap();

        assert!(storage.exists(&task.id));
        assert!(!storage.exists("tt-999999"));
    }
}
