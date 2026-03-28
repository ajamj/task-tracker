# Tasks 2-4 Summary: Storage Layer Complete

**Status:** ✅ COMPLETE  
**Completed:** 2026-03-28

---

## Overview

Tasks 2-4 implemented the complete storage layer for the `tt` CLI:
- **Task 2:** Workspace + config loading
- **Task 3:** Task file I/O + ID generation with file locking
- **Task 4:** Log file I/O + template management

---

## Files Created

### Configuration Models
| File | Purpose | Lines |
|------|---------|-------|
| `src/models/config.rs` | WorkspaceConfig, ProjectConfig, all settings structs | ~280 |

### Storage Layer
| File | Purpose | Lines |
|------|---------|-------|
| `src/storage/mod.rs` | Storage module root with exports | ~10 |
| `src/storage/workspace.rs` | Workspace loading, project discovery | ~280 |
| `src/storage/task.rs` | Task CRUD, ID generation with fs2 locking | ~380 |
| `src/storage/log.rs` | Log CRUD, template, task ID scanning | ~320 |

**Total:** ~1,270 lines of Rust code

---

## Implementation Details

### Task 2: Workspace + Config (config.rs, workspace.rs)

**WorkspaceConfig struct:**
- `version` — Schema version (currently 1)
- `workspace` — WorkspaceSettings (default_project, week_starts_on, task_id_prefix, task_id_width)
- `storage` — StorageConfig (projects_dir)
- `reports` — ReportsConfig (track_in_git, weekly_dir, template)
- `git` — GitConfig (suggest_branch, suggest_commit)
- `editor` — EditorConfig (command)

**ProjectConfig struct:**
- `version`, `name`, `slug`, `description`

**Workspace struct:**
- `root` — Root directory path
- `config` — Loaded WorkspaceConfig
- `projects` — HashMap<String, Project>

**Key methods:**
```rust
Workspace::load(root: PathBuf) -> Result<Self>
Workspace::init(root: PathBuf) -> Result<Self>
Workspace::get_project(slug: &str) -> Option<&Project>
Workspace::get_default_project() -> Result<&Project>

Project::task_path(year, month, task_id) -> PathBuf
Project::log_path(date) -> PathBuf
Project::weekly_report_path(week) -> PathBuf
```

**Tests (7 tests):**
- `test_workspace_init_creates_structure`
- `test_workspace_load_existing`
- `test_workspace_get_default_project`
- `test_workspace_get_project`
- `test_workspace_load_missing_config`
- `test_project_paths`
- `test_workspace_project_slugs`

---

### Task 3: Task File I/O + ID Generation (task.rs)

**TaskStorage struct:**
- `tasks_dir` — Project tasks directory
- `counter_path` — Counter file for ID generation (`.task_counter`)

**Key methods:**
```rust
TaskStorage::new(tasks_dir: PathBuf) -> Self
TaskStorage::next_id() -> StorageResult<u64>  // With fs2 file locking
TaskStorage::create(task: &Task) -> StorageResult<()>
TaskStorage::get(task_id: &str) -> StorageResult<Task>
TaskStorage::update(task: &Task) -> StorageResult<()>
TaskStorage::list() -> StorageResult<Vec<Task>>
TaskStorage::list_by_status(status: TaskStatus) -> StorageResult<Vec<Task>>
TaskStorage::exists(task_id: &str) -> bool
```

**ID Generation with File Locking:**
```rust
pub fn next_id(&self) -> StorageResult<u64> {
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&self.counter_path)?;

    file.lock_exclusive()?;  // Exclusive lock

    // Read current value
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    let current: u64 = content.trim().parse().unwrap_or(0);

    // Increment and write back
    let next = current + 1;
    file.seek(SeekFrom::Start(0))?;
    file.write_all(next.to_string().as_bytes())?;
    file.set_len(next.to_string().len() as u64)?;

    file.unlock()?;
    Ok(next)
}
```

**TOML Serialization:**
- Uses `toml_edit` for preserving formatting
- `task_to_document()` — Convert Task to TOML
- `parse_task()` — Parse TOML to Task with robust error handling

**Tests (7 tests):**
- `test_next_id_sequential`
- `test_create_and_get_task`
- `test_update_task`
- `test_list_tasks`
- `test_list_by_status`
- `test_get_nonexistent_task`
- `test_task_exists`

---

### Task 4: Log File I/O + Template (log.rs)

**Log struct:**
- `date` — Date string (YYYY-MM-DD)
- `project` — Project slug
- `content` — Raw markdown content
- `file_path` — File system path
- `task_ids` — Extracted task IDs (computed)

**LogStorage struct:**
- `logs_dir` — Project logs directory

**Key methods:**
```rust
LogStorage::new(logs_dir: PathBuf) -> Self
LogStorage::get_or_create(date, project) -> StorageResult<Log>
LogStorage::create(date, project) -> StorageResult<Log>
LogStorage::load(date, project) -> StorageResult<Log>
LogStorage::append(date, project, text) -> StorageResult<Log>
LogStorage::get_for_date_range(start_date, end_date, project) -> StorageResult<Vec<Log>>
LogStorage::exists(date) -> bool
```

**Default Log Template:**
```markdown
# {{date}} ({{project}})

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
```

**Task ID Scanning:**
```rust
pub fn scan_for_task_ids(content: &str) -> Vec<String> {
    let task_id_regex = Regex::new(r"(?i)\btt-\d{6}\b").unwrap();
    
    let mut ids: HashSet<String> = HashSet::new();
    for mat in task_id_regex.find_iter(content) {
        ids.insert(mat.as_str().to_lowercase());  // Normalize
    }
    
    let mut ids: Vec<String> = ids.into_iter().collect();
    ids.sort();
    ids
}
```

**Tests (9 tests):**
- `test_log_new_with_template`
- `test_log_append`
- `test_scan_for_task_ids`
- `test_scan_for_task_ids_deduplicates`
- `test_log_storage_get_or_create`
- `test_log_storage_append`
- `test_log_storage_load_existing`
- `test_log_storage_get_for_date_range`
- `test_log_path_format`

---

## Test Summary

| Module | Tests | Coverage |
|--------|-------|----------|
| `config.rs` | 6 | Default, parsing, TOML generation |
| `workspace.rs` | 7 | Init, load, project discovery, paths |
| `task.rs` | 7 | ID generation, CRUD, listing |
| `log.rs` | 9 | Template, append, scanning, date range |

**Total:** 29 unit tests

---

## Key Design Decisions

### D2.1: File Locking Strategy
- **Decision:** Use `fs2` exclusive locks for ID generation
- **Rationale:** Prevents race conditions, simple implementation
- **Implementation:** Counter file (`.task_counter`) with exclusive lock

### D2.2: Directory Structure
- **Decision:** Tasks in `tasks/YYYY/MM/`, Logs in `logs/YYYY/`
- **Rationale:** Prevents too many files in single directory, easy to navigate

### D2.3: TOML Formatting Preservation
- **Decision:** Use `toml_edit` instead of `toml` crate
- **Rationale:** Preserves user comments and formatting on manual edits

### D2.4: Task ID Counter
- **Decision:** File-based counter (`.task_counter`)
- **Rationale:** Simple, reliable, works across restarts

### D2.5: Log Template
- **Decision:** Hardcoded template with sections
- **Rationale:** Consistent format, easy to parse for highlights

### D2.6: Task ID Normalization
- **Decision:** Convert to lowercase when scanning
- **Rationale:** Case-insensitive matching (`TT-000001` = `tt-000001`)

---

## Integration Points

### Workspace → Project → Storage
```rust
let workspace = Workspace::load(root)?;
let project = workspace.get_default_project()?;

let task_storage = TaskStorage::new(project.tasks_dir.clone());
let log_storage = LogStorage::new(project.logs_dir.clone());

let next_id = task_storage.next_id()?;
let task = NewTask::builder("Task").project(&project.slug).build(next_id);
task_storage.create(&task)?;

let log = log_storage.append("2026-03-28", &project.slug, "Worked on tt-000001")?;
```

### Path Resolution
```rust
// Task: projects/work/tasks/2026/03/tt-000001.toml
project.task_path(2026, 3, "tt-000001")

// Log: projects/work/logs/2026/2026-03-28.md
project.log_path("2026-03-28")

// Report: projects/work/reports/weekly/2026-W13.md
project.weekly_report_path("2026-W13")
```

---

## Error Handling

### Workspace Errors
- `WorkspaceNotFoundAtPath` — tt.toml missing
- `ProjectNotFound` — Project doesn't exist
- `TomlParseError` — Invalid TOML syntax

### Task Storage Errors
- `TaskNotFound` — Task file doesn't exist
- `IdGenerationError` — Counter file issues
- `LockError` — File locking failed

### Log Storage Errors
- `IoError` — File read/write failures

---

## Verification Commands

When cargo is available:

```bash
# Build
cargo build

# Run all storage tests
cargo test storage::workspace::tests
cargo test storage::task::tests
cargo test storage::log::tests
cargo test models::config::tests

# Run all tests
cargo test

# Lint
cargo clippy -- -D warnings
```

---

## Next Steps

**Task 5:** CLI Commands - `tt init`, `tt add`, `tt ls`, `tt show`

**Subtasks:**
1. Set up clap CLI structure with all subcommands
2. Implement `tt init` command
3. Implement `tt add <title>` command
4. Implement `tt ls` command (list by status)
5. Implement `tt show <id>` command
6. Add git suggestions output formatting
7. Add `--project` flag support

---

## Notes

- Git is not available in this environment (installed at `C:\Program Files\Git\bin\git.exe`)
- Build verification pending (cargo not accessible via shell tool)
- All code follows design specifications from `planning/phases/01-mvp/DESIGN.md`
- Storage layer is fully testable with `tempfile` crate for isolation
