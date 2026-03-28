# Implementation Context: Phase 1 (MVP)

**Status:** Draft  
**Created:** 2026-03-28

---

## Coding Conventions

### General Rust Style

- **Edition:** Rust 2021
- **Formatter:** `rustfmt` (default settings)
- **Linter:** `clippy` (all warnings enabled)
- **MSRV:** 1.75 (check with `cargo msrv`)

### File Organization

```
src/
├── main.rs           # Entry point, CLI dispatch
├── lib.rs            # Library exports, common types
├── error.rs          # Error types (thiserror)
├── cli/
│   ├── mod.rs        # Module root
│   ├── args.rs       # Clap definitions
│   ├── commands.rs   # Command implementations
│   └── format.rs     # Output formatting
├── storage/
│   ├── mod.rs
│   ├── workspace.rs  # Workspace loading
│   ├── task.rs       # Task file operations
│   ├── log.rs        # Log file operations
│   └── id.rs         # ID generation with locking
├── linking/
│   ├── mod.rs
│   ├── scanner.rs    # Task ID regex scanning
│   └── regex.rs      # Compiled regex cache
├── reports/
│   ├── mod.rs
│   ├── weekly.rs     # Weekly report generation
│   ├── highlights.rs # Highlight extraction
│   └── template.rs   # Embedded templates
└── models/
    ├── mod.rs
    ├── task.rs       # Task struct
    ├── week.rs       # WeekRange struct
    └── config.rs     # Config structs
```

### Naming Conventions

| Type | Convention | Example |
|------|------------|---------|
| Modules | `snake_case` | `task_storage`, `id_generator` |
| Structs | `PascalCase` | `Task`, `WeekRange`, `GitSuggestion` |
| Enums | `PascalCase` | `TaskStatus`, `Priority`, `TtError` |
| Functions | `snake_case` | `generate_report`, `scan_for_ids` |
| Constants | `UPPER_SNAKE_CASE` | `TASK_ID_WIDTH`, `DEFAULT_PRIORITY` |
| Traits | `PascalCase` | `TaskStorage`, `LogStorage` |

### Error Handling

**Library code:** Use `thiserror` for custom error types

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TtError {
    #[error("Workspace not found")]
    WorkspaceNotFound,
    // ...
}

pub type Result<T> = std::result::Result<T, TtError>;
```

**Application code (main.rs):** Use `anyhow` for error propagation

```rust
use anyhow::Result;

fn main() -> Result<()> {
    // Use ? for propagation
    // Use anyhow::Context for error messages
}
```

### Logging

Use `tracing` crate for structured logging:

```rust
use tracing::{info, debug, warn, error};

fn create_task(task: &NewTask) -> Result<Task> {
    debug!(title = %task.title, project = %task.project, "Creating task");
    // ...
    info!(id = %new_task.id, "Task created");
    Ok(new_task)
}
```

**Log levels:**
- `trace`: Very detailed debugging
- `debug`: Module-level debugging
- `info`: High-level progress (user-facing actions)
- `warn`: Recoverable issues
- `error`: Unrecoverable errors

---

## Library Choices

### Confirmed Dependencies (v0.1)

```toml
[dependencies]
# CLI
clap = { version = "4.5", features = ["derive"] }
clap_complete = "4.5"

# TOML
toml_edit = "0.22"
toml_datetime = "0.6"

# Date/time
chrono = { version = "0.4", features = ["serde"] }

# Markdown parsing
pulldown-cmark = "0.10"

# Templates
minijinja = { version = "2.0", features = ["loader"] }

# Regex
regex = "1.10"
once_cell = "1.19"

# File operations
fs2 = "0.12"
dirs = "5.0"

# Error handling
thiserror = "1.0"
anyhow = "1.0"

# Serialization
serde = { version = "1.0", features = ["derive"] }

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Utilities
slug = "0.1"  # For generating URL-friendly slugs
```

### Dev Dependencies

```toml
[dev-dependencies]
# Snapshot testing
insta = { version = "1.38", features = ["yaml", "json"] }

# Test utilities
tempfile = "3.10"
assert_cmd = "2.0"
predicates = "3.1"
```

### Deferred Dependencies (v0.2+)

| Crate | Purpose | Phase |
|-------|---------|-------|
| `comrak` | Markdown rendering | v0.2 (if needed) |
| `comfy-table` / `tabled` | Pretty tables | v0.2 (CLI UX) |
| `rusqlite` | SQLite cache | v0.3 (performance) |
| `notify` | File watching | v0.3 (optional) |
| `axum` / `actix-web` | Dashboard server | v0.2 (dashboard) |

---

## Testing Strategy

### Test Categories

#### 1. Unit Tests (In-place)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_week_range_from_date() {
        let date = NaiveDate::from_ymd_opt(2026, 3, 28).unwrap(); // Saturday
        let week = WeekRange::from_date(date);

        assert_eq!(week.start, NaiveDate::from_ymd_opt(2026, 3, 23).unwrap()); // Monday
        assert_eq!(week.end, NaiveDate::from_ymd_opt(2026, 3, 29).unwrap()); // Sunday
        assert_eq!(week.iso_week, "2026-W13");
    }

    #[test]
    fn test_status_transition() {
        assert!(TaskStatus::Todo.can_transition_to(TaskStatus::Doing));
        assert!(!TaskStatus::Done.can_transition_to(TaskStatus::Todo));
    }
}
```

#### 2. Integration Tests (`tests/`)

```rust
// tests/cli_tests.rs
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_init_creates_workspace() {
    let temp_dir = tempfile::tempdir().unwrap();
    let mut cmd = Command::cargo_bin("tt").unwrap();

    cmd.arg("init").current_dir(&temp_dir);
    let assert = cmd.assert();

    assert.success();
    assert!(temp_dir.path().join("tt.toml").exists());
}

#[test]
fn test_add_creates_task() {
    let temp_dir = tempfile::tempdir().unwrap();
    init_workspace(&temp_dir);

    let mut cmd = Command::cargo_bin("tt").unwrap();
    cmd.arg("add")
        .arg("Test task")
        .current_dir(&temp_dir);

    cmd.assert().success();

    // Verify task file created
    let tasks_dir = temp_dir.path().join("projects/work/tasks");
    // ... assertion logic
}
```

#### 3. Snapshot Tests (insta)

```rust
#[test]
fn test_cli_help_snapshot() {
    let mut cmd = Command::cargo_bin("tt").unwrap();
    let assert = cmd.arg("--help").assert();

    let output = String::from_utf8_lossy(&assert.get_output().stdout);
    insta::assert_snapshot!(output);
}

#[test]
fn test_weekly_report_snapshot() {
    let workspace = create_test_workspace_with_data();
    let report = generate_weekly_report(&workspace, "2026-W13").unwrap();

    insta::assert_snapshot!(report);
}
```

**Review snapshots:**
```bash
cargo insta test          # Run tests
cargo insta review        # Review pending snapshots
cargo insta accept        # Accept all changes
```

### Test Coverage Goals

| Module | Target Coverage |
|--------|-----------------|
| `models/` | 90%+ (pure data, easy to test) |
| `storage/` | 80%+ (critical path) |
| `linking/` | 90%+ (regex logic) |
| `reports/` | 85%+ (template rendering) |
| `cli/` | 60%+ (integration tests cover rest) |

---

## Performance Considerations

### Benchmarks to Target

| Operation | Target | Notes |
|-----------|--------|-------|
| `tt init` | < 100ms | One-time operation |
| `tt add` | < 50ms | With file locking |
| `tt ls` | < 100ms | Up to 100 tasks |
| `tt log` | < 50ms | Append + scan |
| `tt report week` | < 500ms | 100 tasks, 7 logs |
| Log scanning | < 10ms/MB | Regex performance |

### Optimization Strategies

1. **Regex compilation:** Use `once_cell::Lazy` for static regexes
2. **File I/O:** Batch reads where possible
3. **ID generation:** File locking with `fs2`, minimal contention
4. **Template rendering:** Pre-compile templates (minijinja does this)

### Future Optimization (v0.3)

- SQLite read cache for large workspaces (1000+ tasks)
- Incremental report generation (cache intermediate results)
- Parallel log scanning (`rayon` crate)

---

## Cross-Platform Considerations

### Path Handling

**Always use `PathBuf`:**
```rust
// ✅ Good
let task_path = tasks_dir.join(format!("tt-{:06}.toml", id));

// ❌ Bad
let task_path = format!("{}/tt-{:06}.toml", tasks_dir.display(), id);
```

### Line Endings

TOML and Markdown files should use Unix line endings (`\n`) for consistency:

```rust
use std::io::Write;

fn write_task_file(path: &Path, content: &str) -> std::io::Result<()> {
    let mut file = File::create(path)?;
    // Normalize line endings
    let normalized = content.replace("\r\n", "\n");
    file.write_all(normalized.as_bytes())?;
    Ok(())
}
```

### Editor Detection

```rust
fn get_editor() -> String {
    std::env::var("EDITOR")
        .ok()
        .or_else(|| {
            // Fallback to platform defaults
            if cfg!(windows) {
                Some("notepad".to_string())
            } else {
                Some("vi".to_string())
            }
        })
        .unwrap_or_else(|| "vi".to_string())
}
```

---

## Git Integration Design

### Suggestion Format

All git integration is **suggestions only** - never execute git commands.

```rust
pub fn print_git_suggestion(suggestion: &GitSuggestion) {
    eprintln!("\n{}", style("Git suggestions (not executed):").bold().yellow());
    eprintln!("  Suggested branch: {}", style(&suggestion.branch).cyan());
    eprintln!("  Suggested commit: {}", style(&suggestion.commit_message).cyan());
    eprintln!("  Files changed:");
    for file in &suggestion.files_changed {
        eprintln!("    - {}", file);
    }
    eprintln!("\n  Run these commands:");
    eprintln!("    git add -A");
    eprintln!("    git checkout -b {}", suggestion.branch);
    eprintln!("    git commit -m \"{}\"", suggestion.commit_message);
}
```

### Commit Message Conventions

Follow Conventional Commits:

| Action | Format | Example |
|--------|--------|---------|
| Task add | `task(add): tt-000001 Title` | `task(add): tt-000001 Refactor config loader` |
| Task start | `task(start): tt-000001 Title` | `task(start): tt-000001 Refactor config loader` |
| Task done | `task(done): tt-000001 Title` | `task(done): tt-000001 Refactor config loader` |
| Log entry | `log(add): YYYY-MM-DD (project)` | `log(add): 2026-03-28 (work)` |
| Weekly report | `report(week): YYYY-Www (project)` | `report(week): 2026-W13 (work)` |
| Init | `chore(init): bootstrap worklog` | `chore(init): bootstrap worklog` |

### Branch Name Format

```
<project>/tt-<id>-<slug>
```

Example: `work/tt-000001-refactor-config-loader`

Slug generation:
```rust
fn slugify(text: &str) -> String {
    slug::slugify(text)
        .chars()
        .take(50)  // Limit length
        .collect()
}
```

---

## Configuration Design

### tt.toml (Root Config)

```toml
version = 1

[workspace]
default_project = "work"
week_starts_on = "monday"  # Fixed for v0.1
task_id_prefix = "tt-"
task_id_width = 6

[storage]
projects_dir = "projects"

[reports]
track_in_git = true
weekly_dir = "reports/weekly"
template = "default"  # Reserved for v0.3

[git]
suggest_branch = true
suggest_commit = true

[editor]
command = ""  # Empty = use $EDITOR env var
```

### project.toml (Project Config)

```toml
version = 1
name = "Work"
slug = "work"
description = "Work-related tasks and reports"
```

### Config Loading Order

1. Check for `tt.toml` in current directory
2. If not found, search parent directories (like git)
3. If still not found, error: "Workspace not initialized. Run `tt init`."

---

## Documentation Requirements

### README Structure

1. Title + one-liner
2. What it is (3 bullets)
3. Quickstart (< 5 minutes)
4. Core workflow (daily, weekly)
5. Workspace layout
6. Task format example
7. Logs & auto-linking
8. Reports
9. Git philosophy
10. Installation
11. Configuration
12. Roadmap
13. Contributing
14. License

### Inline Documentation

**Functions:** Doc comments for public API

```rust
/// Generate a weekly report for the given week.
///
/// The report includes:
/// - Tasks completed this week (by `done_at` date)
/// - Tasks currently in progress
/// - Tasks currently blocked
/// - Task IDs mentioned in daily logs
/// - Worklog highlights extracted from log sections
///
/// # Arguments
///
/// * `workspace` - The workspace to generate the report from
/// * `week` - ISO week string (e.g., "2026-W13")
///
/// # Returns
///
/// The generated report content as a Markdown string.
///
/// # Example
///
/// ```
/// let report = generate_weekly_report(&workspace, "2026-W13")?;
/// ```
pub fn generate_weekly_report(workspace: &Workspace, week: &str) -> Result<String> {
    // ...
}
```

**Modules:** Module-level doc comments

```rust
//! Task storage management.
//!
//! This module provides abstractions for reading and writing task files.
//! Key features:
//! - Per-file TOML storage
//! - Atomic ID generation with file locking
//! - Robust parsing with helpful error messages

pub mod task_storage;
```

---

## Security Considerations

### Local-First Design

- All data stored locally (no network calls)
- No telemetry, analytics, or external services
- Dashboard (v0.2) binds to `localhost` only

### File Permissions

- Task files: Default OS permissions (no special handling)
- Counter file: Read/write for current user only

### Input Validation

- Sanitize task titles for file paths (prevent directory traversal)
- Validate date formats before parsing
- Limit tag length (max 50 chars) to prevent abuse

---

## Release Checklist (v0.1)

- [ ] All R1.x requirements implemented
- [ ] Tests pass (`cargo test`)
- [ ] Clippy clean (`cargo clippy -- -D warnings`)
- [ ] README complete with quickstart
- [ ] Sample workspace in repo
- [ ] `cargo install --path .` works
- [ ] Windows testing (user's machine)
- [ ] Snapshot tests reviewed and accepted
- [ ] CHANGELOG.md created
- [ ] Git tag: `v0.1.0`

---

## Open Decisions (To Be Confirmed)

| Decision | Options | Recommendation | Status |
|----------|---------|----------------|--------|
| Editor detection | `$EDITOR` vs `editors` crate | `$EDITOR` + fallback | Pending |
| Task ID width | Fixed 6 vs configurable | Fixed 6 for v0.1 | Pending |
| Report template | Hardcoded vs external | Hardcoded for v0.1 | Pending |
| Table formatting | Plain text vs `comfy-table` | Plain text for v0.1 | Pending |

---

## Next Steps

1. **Review this document** — Confirm implementation preferences
2. **Approve or iterate** — Provide feedback on open decisions
3. **Run `/gsd:plan-phase 1`** — Generate atomic implementation tasks
