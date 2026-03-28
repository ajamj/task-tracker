# Technical Research: `tt` CLI Task Tracking Tool

A comprehensive technical analysis for building a Git-friendly personal task tracking CLI tool in Rust.

---

## Table of Contents

1. [Rust Crate Recommendations](#1-rust-crate-recommendations)
2. [Project Structure](#2-project-structure)
3. [Potential Pitfalls & Solutions](#3-potential-pitfalls--solutions)
4. [Distribution Strategy](#4-distribution-strategy)
5. [Alternative Approaches](#5-alternative-approaches)
6. [Recommended Dependency Versions](#6-recommended-dependency-versions)

---

## 1. Rust Crate Recommendations

### 1.1 CLI Framework

**Recommendation: `clap` v4.x with derive macros**

```toml
[dependencies]
clap = { version = "4.5", features = ["derive"] }
```

**Why clap:**
- Industry standard with excellent documentation
- Derive macros reduce boilerplate significantly
- Built-in help generation and shell completions
- Strong ecosystem support (clap_complete, clap_mangen)
- Actively maintained with regular updates

**Alternative considered:**
- `structopt` - Deprecated, merged into clap
- `argh` - Simpler but less feature-rich
- `commander` - Less mature ecosystem

**Example usage:**
```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "tt")]
#[command(about = "Git-friendly task tracking CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new task
    Add {
        #[arg(short, long)]
        title: String,
        #[arg(short, long, default_value = "medium")]
        priority: String,
    },
    /// List tasks for current week
    List,
    /// Generate weekly report
    Report {
        #[arg(short, long)]
        week: Option<String>,
    },
}
```

---

### 1.2 TOML Parsing

**Recommendation: `toml_edit` v0.22.x**

```toml
[dependencies]
toml_edit = "0.22"
```

**Why toml_edit over `toml`:**
- **Preserves comments and formatting** - Critical for user-edited files
- Round-trip editing without losing user modifications
- Supports incremental updates
- Better error messages for malformed TOML

**The standard `toml` crate:**
- Parses to a lossless DOM but doesn't preserve formatting
- Better for read-only scenarios
- Use `toml_datetime` for date handling within TOML

**Example usage:**
```rust
use toml_edit::{DocumentMut, value, Item};
use std::fs;

fn create_task_file(id: u64, title: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut doc = DocumentMut::new();
    
    doc["id"] = value(id);
    doc["title"] = value(title);
    doc["status"] = value("open");
    doc["created_at"] = value(chrono::Utc::now().to_rfc3339());
    
    // Preserves any comments users add later
    fs::write(format!("tt-{id:06}.toml"), doc.to_string())?;
    Ok(())
}
```

---

### 1.3 Markdown Parsing

**Recommendation: `pulldown-cmark` v0.10.x for parsing, `comrak` v0.24.x for rendering**

```toml
[dependencies]
pulldown-cmark = "0.10"
comrak = "0.24"
```

**Why this combination:**

| Feature | pulldown-cmark | comrak |
|---------|---------------|--------|
| Parsing speed | Excellent | Good |
| CommonMark compliant | Yes | Yes |
| GFM extensions | Via pulldown-cmark-gfm | Built-in |
| Rendering to HTML | Manual | Built-in |
| Syntax highlighting | No | Yes (with syntect) |
| Task lists | Via extensions | Built-in |

**For `tt` specifically:**
- Use `pulldown-cmark` for **scanning** daily logs (faster, lighter)
- Use `comrak` for **report generation** (better HTML output, task list support)

**Example - Scanning for task IDs:**
```rust
use pulldown_cmark::{Parser, Event, Tag};
use regex::Regex;

fn extract_task_ids_from_log(content: &str) -> Vec<String> {
    let task_id_pattern = Regex::new(r"tt-\d{6}").unwrap();
    let parser = Parser::new(content);
    
    let mut ids = Vec::new();
    for event in parser {
        if let Event::Text(text) = event {
            for mat in task_id_pattern.find_iter(&text) {
                ids.push(mat.as_str().to_string());
            }
        }
    }
    ids
}
```

---

### 1.4 Date/Time Handling

**Recommendation: `chrono` v0.4.x with `time` v0.3.x for serialization**

```toml
[dependencies]
chrono = { version = "0.4", features = ["serde"] }
time = { version = "0.3", features = ["serde", "macros"] }
```

**Why chrono (despite maintenance concerns):**
- ISO-8601 week support built-in (`iso_week()` method)
- Mature ecosystem with extensive examples
- Monday-first week configuration native
- Better timezone handling for cross-platform consistency

**The `time` crate:**
- More modern API, actively maintained
- Better for serialization/deserialization
- Use together: chrono for logic, time for storage format

**Example - ISO-8601 week handling:**
```rust
use chrono::{Datelike, NaiveDate, Weekday};

fn get_week_start(date: NaiveDate) -> NaiveDate {
    // ISO-8601: Week starts on Monday
    let weekday = date.weekday();
    let days_since_monday = weekday.num_days_from_monday();
    date - chrono::Duration::days(days_since_monday as i64)
}

fn get_iso_week_string(date: NaiveDate) -> String {
    let iso_week = date.iso_week();
    format!("{}-W{:02}", iso_week.year(), iso_week.week())
}

fn get_week_range(date: NaiveDate) -> (NaiveDate, NaiveDate) {
    let start = get_week_start(date);
    let end = start + chrono::Duration::days(6);
    (start, end)
}
```

---

### 1.5 Template Rendering

**Recommendation: `minijinja` v2.x**

```toml
[dependencies]
minijinja = { version = "2.0", features = ["loader"] }
```

**Why minijinja over alternatives:**

| Template Engine | Pros | Cons for `tt` |
|-----------------|------|---------------|
| **minijinja** | Fast, Jinja2-compatible, no runtime deps, excellent error messages | Newer ecosystem |
| askama | Compile-time checked, type-safe | Requires build scripts, slower compilation |
| tera | Django-compatible, feature-rich | Heavier, more dependencies |
| handlebars | Simple, popular | Less expressive than Jinja2 |

**Example - Weekly report template:**
```rust
use minijinja::{Environment, context};

fn generate_weekly_report(tasks: Vec<Task>, week: &str) -> Result<String, minijinja::Error> {
    let mut env = Environment::new();
    
    let template_str = r#"
# Weekly Report: {{ week }}

## Summary
- Total tasks: {{ tasks|length }}
- Completed: {{ tasks|selectattr("status", "equalto", "done")|list|length }}
- In Progress: {{ tasks|selectattr("status", "equalto", "in_progress")|list|length }}

## Tasks
{% for task in tasks %}
### {{ task.id }}
- **Title:** {{ task.title }}
- **Status:** {{ task.status }}
- **Priority:** {{ task.priority }}
{% endfor %}
    "#;
    
    let template = env.from_str(template_str)?;
    template.render(context!(week => week, tasks => tasks))
}
```

---

### 1.6 Testing

**Recommendation: `insta` v1.x for snapshot testing + standard test framework**

```toml
[dev-dependencies]
insta = { version = "1.38", features = ["yaml", "json"] }
tempfile = "3.10"
assert_cmd = "2.0"
predicates = "3.1"
```

**Why insta:**
- Perfect for testing report output, CLI help text, TOML generation
- Catch unintended output changes
- Easy review workflow with `cargo insta review`

**Example - Snapshot test for report generation:**
```rust
#[cfg(test)]
mod tests {
    use insta::assert_yaml_snapshot;
    
    #[test]
    fn test_weekly_report_format() {
        let tasks = vec![
            Task { id: 1, title: "Test task".to_string(), status: "done".to_string() },
        ];
        let report = generate_weekly_report(tasks, "2024-W01").unwrap();
        
        assert_yaml_snapshot!(report);
    }
    
    #[test]
    fn test_cli_help_output() {
        let mut cmd = Command::cargo_bin("tt").unwrap();
        let assert = cmd.arg("--help").assert();
        assert.success();
        
        insta::assert_snapshot!(String::from_utf8_lossy(&assert.get_output().stdout));
    }
}
```

**Additional testing crates:**
- `tempfile` - Isolated test directories
- `assert_cmd` - CLI integration testing
- `predicates` - Output assertion helpers

---

## 2. Project Structure

### 2.1 Recommended Layout

```
tt/
├── Cargo.toml              # Workspace root (if using workspace)
├── Cargo.lock
├── README.md
├── LICENSE
├── .gitignore
│
├── planning/               # Design docs, research (like this file)
│   ├── RESEARCH.md
│   └── SPEC.md
│
├── src/
│   ├── main.rs             # CLI entry point
│   ├── lib.rs              # Library exports
│   │
│   ├── cli/                # CLI layer
│   │   ├── mod.rs
│   │   ├── commands.rs     # Command implementations
│   │   └── args.rs         # Clap argument definitions
│   │
│   ├── storage/            # File I/O and data layer
│   │   ├── mod.rs
│   │   ├── task_file.rs    # Individual task TOML handling
│   │   ├── workspace.rs    # Multi-project workspace management
│   │   └── id_generator.rs # Task ID generation
│   │
│   ├── linking/            # Task ID detection in logs
│   │   ├── mod.rs
│   │   ├── scanner.rs      # Markdown scanning logic
│   │   └── regex_cache.rs  # Compiled regex management
│   │
│   ├── reports/            # Report generation
│   │   ├── mod.rs
│   │   ├── weekly.rs       # Weekly report logic
│   │   └── templates/      # Embedded templates
│   │
│   └── models/             # Data structures
│       ├── mod.rs
│       ├── task.rs         # Task struct
│       └── week.rs         # Week representation
│
├── tests/                  # Integration tests
│   ├── cli_tests.rs
│   └── fixtures/           # Test data
│
├── templates/              # External templates (if not embedded)
│   └── weekly_report.j2
│
└── examples/               # Usage examples
    └── basic_usage.rs
```

### 2.2 Module Organization Rationale

```
src/
├── cli/          # Pure CLI concerns - parsing args, formatting output
├── storage/      # File system operations - reading/writing TOML
├── linking/      # Cross-reference logic - finding task IDs in logs
├── reports/      # Aggregation and templating - weekly summaries
└── models/       # Shared data types - used across all modules
```

**Key design principles:**
1. **Separation of concerns** - CLI doesn't know about file formats
2. **Testability** - Each module can be tested in isolation
3. **Future extensibility** - Easy to add SQLite layer in storage/

### 2.3 Workspace Consideration (Future)

For v0.3+ with multiple binaries or libraries:

```toml
# Cargo.toml (workspace root)
[workspace]
members = ["crates/tt-core", "crates/tt-cli", "crates/tt-reports"]
resolver = "2"
```

**Not recommended for v0.1** - Keep it simple initially.

---

## 3. Potential Pitfalls & Solutions

### 3.1 Handling Malformed TOML Gracefully

**Problem:** Users manually edit task files, introducing syntax errors.

**Solution: Multi-layer validation with helpful error messages**

```rust
use toml_edit::TomlError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TaskFileError {
    #[error("Failed to parse TOML: {0}")]
    ParseError(#[from] TomlError),
    
    #[error("Missing required field: {0}")]
    MissingField(String),
    
    #[error("Invalid field type for '{field}': expected {expected}, got {actual}")]
    InvalidFieldType { field: String, expected: String, actual: String },
    
    #[error("Task file corrupted: {0}")]
    Corruption(String),
}

pub fn load_task_file(path: &Path) -> Result<Task, TaskFileError> {
    let content = fs::read_to_string(path)
        .map_err(|e| TaskFileError::Corruption(format!("Cannot read file: {e}")))?;
    
    let doc = content.parse::<DocumentMut>()?;
    
    // Validate required fields with helpful messages
    let id = doc.get("id")
        .ok_or_else(|| TaskFileError::MissingField("id".to_string()))?
        .as_integer()
        .ok_or_else(|| TaskFileError::InvalidFieldType {
            field: "id".to_string(),
            expected: "integer".to_string(),
            actual: "unknown".to_string(),
        })? as u64;
    
    // ... continue validation
    
    Ok(Task { id, /* ... */ })
}
```

**Additional strategies:**
- Create `.tt-backup/` directory for auto-backups before edits
- Provide `tt validate` command to check all task files
- Use `toml_edit`'s span information for precise error locations

---

### 3.2 Regex Performance for Log Scanning

**Problem:** Scanning large daily logs with regex can be slow.

**Solution: Compiled regex cache + incremental scanning**

```rust
use once_cell::sync::Lazy;
use regex::Regex;

// Compile once at startup
static TASK_ID_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"tt-\d{6}").expect("Invalid regex pattern")
});

// For very large files, use memmap2 for zero-copy reading
use memmap2::Mmap;

pub fn scan_log_file(path: &Path) -> Result<Vec<String>, io::Error> {
    let file = File::open(path)?;
    let mmap = unsafe { Mmap::map(&file)? };
    
    // Search in memory without loading entire file into heap
    let mut ids = Vec::new();
    for mat in TASK_ID_PATTERN.find_iter(&mmap) {
        ids.push(String::from_utf8_lossy(mat.as_bytes()).to_string());
    }
    
    Ok(ids)
}
```

**Performance benchmarks to target:**
- < 10ms for 1MB log file
- < 100ms for 10MB log file

**Alternative: Use `aho-corasick` for multiple patterns**
```toml
[dependencies]
aho-corasick = "1.1"
```

If scanning for multiple patterns (task IDs, dates, priorities), `aho-corasick` provides O(n) matching.

---

### 3.3 Cross-Platform Path Handling

**Problem:** Windows uses `\`, Unix uses `/`, and users may have different expectations.

**Solution: Use `PathBuf` consistently + `dirs` crate for standard locations**

```rust
use std::path::{Path, PathBuf};
use dirs::home_dir;

#[derive(Debug, Clone)]
pub struct Workspace {
    root: PathBuf,
    tasks_dir: PathBuf,
    logs_dir: PathBuf,
}

impl Workspace {
    pub fn new(root: PathBuf) -> Self {
        Self {
            tasks_dir: root.join("tasks"),
            logs_dir: root.join("logs"),
            root,
        }
    }
    
    pub fn task_path(&self, id: u64) -> PathBuf {
        // Always use forward slashes in code, OS handles conversion
        self.tasks_dir.join(format!("tt-{id:06}.toml"))
    }
    
    pub fn log_path(&self, date: NaiveDate) -> PathBuf {
        self.logs_dir.join(format!("{}.md", date.format("%Y-%m-%d")))
    }
    
    /// Get default workspace location
    pub fn default_location() -> Option<PathBuf> {
        home_dir().map(|home| home.join(".tt-workspace"))
    }
}
```

**Key crates:**
- `dirs` v5.x - Platform-specific directories
- `path-absolutize` v3.x - Convert relative to absolute paths
- `dunce` v1.x - Normalize Windows paths (UNC handling)

---

### 3.4 ID Generation Concurrency

**Problem:** Multiple `tt add` commands running simultaneously could generate duplicate IDs.

**Solution: Atomic file-based ID counter with file locking**

```rust
use fs2::FileExt;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write, Seek, SeekFrom};

pub struct IdGenerator {
    counter_path: PathBuf,
}

impl IdGenerator {
    pub fn new(counter_path: PathBuf) -> Self {
        // Initialize counter file if it doesn't exist
        if !counter_path.exists() {
            fs::write(&counter_path, "0").ok();
        }
        Self { counter_path }
    }
    
    pub fn next_id(&self) -> Result<u64, Box<dyn std::error::Error>> {
        // Open with read+write, create if not exists
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&self.counter_path)?;
        
        // Acquire exclusive lock
        file.lock_exclusive()?;
        
        // Read current value
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        let current: u64 = content.trim().parse().unwrap_or(0);
        
        // Increment and write back
        let next = current + 1;
        file.seek(SeekFrom::Start(0))?;
        file.write_all(next.to_string().as_bytes())?;
        file.set_len(next.to_string().len() as u64)?;
        
        // Release lock (automatic on drop, but explicit for clarity)
        file.unlock()?;
        
        Ok(next)
    }
}
```

**Key crate:** `fs2` v0.12.x for cross-platform file locking

**Alternative approach for v0.3:**
- Use timestamp-based IDs: `tt-YYYYMMDDHHMMSS`
- Eliminates counter file entirely
- Trade-off: Longer IDs, but guaranteed uniqueness

---

### 3.5 Robustness Against Manual Edits

**Problem:** Users may edit TOML/Markdown files directly, potentially breaking assumptions.

**Solutions:**

1. **Defensive parsing with defaults:**
```rust
impl Task {
    pub fn from_document(doc: &DocumentMut) -> Result<Self, TaskError> {
        Ok(Self {
            id: doc.get("id").and_then(|v| v.as_integer()).unwrap_or(0) as u64,
            title: doc.get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("Untitled")
                .to_string(),
            status: doc.get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("open")
                .to_string(),
            // Always provide sensible defaults
        })
    }
}
```

2. **Schema validation command:**
```rust
// tt validate --all
pub fn validate_workspace(workspace: &Workspace) -> ValidationResult {
    let mut errors = Vec::new();
    
    for task_file in workspace.list_task_files() {
        match load_task_file(&task_file) {
            Ok(_) => {}
            Err(e) => errors.push(ValidationIssue {
                file: task_file,
                error: e.to_string(),
                severity: Severity::Error,
            }),
        }
    }
    
    ValidationResult { errors, warnings: vec![] }
}
```

3. **Auto-repair for common issues:**
```rust
pub fn repair_task_file(path: &Path) -> Result<RepairReport, RepairError> {
    let content = fs::read_to_string(path)?;
    let mut report = RepairReport::default();
    
    // Fix missing newlines at EOF
    if !content.ends_with('\n') {
        fs::write(path, format!("{content}\n"))?;
        report.fixes.push("Added trailing newline".to_string());
    }
    
    // Fix common typos in field names
    let fixed = content
        .replace("tite:", "title:")
        .replace("statu:", "status:");
    
    if fixed != content {
        fs::write(path, fixed)?;
        report.fixes.push("Corrected field name typos".to_string());
    }
    
    Ok(report)
}
```

---

## 4. Distribution Strategy

### 4.1 Primary: `cargo install`

**Cargo.toml configuration:**
```toml
[package]
name = "tt"
version = "0.1.0"
edition = "2021"
license = "MIT OR Apache-2.0"
description = "Git-friendly personal task tracking CLI"
repository = "https://github.com/yourusername/tt"
keywords = ["cli", "task", "productivity", "rust"]
categories = ["command-line-utilities"]

[[bin]]
name = "tt"
path = "src/main.rs"

[profile.release]
lto = true
codegen-units = 1
strip = true
```

**Installation command:**
```bash
cargo install tt
```

**Pros:**
- Simple for Rust developers
- Always gets latest version
- No maintenance overhead

**Cons:**
- Requires Rust toolchain
- Slow compilation for end users

---

### 4.2 GitHub Releases with Pre-built Binaries

**Tool: `cross` for cross-compilation**

```toml
# .github/workflows/release.yml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  build:
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
          - os: ubuntu-latest
            target: x86_64-unknown-linux-musl
          - os: macos-latest
            target: x86_64-apple-darwin
          - os: macos-latest
            target: aarch64-apple-darwin
          - os: windows-latest
            target: x86_64-pc-windows-msvc
    
    runs-on: ${{ matrix.os }}
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-action@stable
        with:
          targets: ${{ matrix.target }}
      
      - name: Build
        run: cargo build --release --target ${{ matrix.target }}
      
      - name: Package
        run: |
          mkdir release
          cp target/${{ matrix.target }}/release/tt release/
          tar -czf tt-${{ matrix.target }}.tar.gz -C release tt
      
      - name: Upload
        uses: softprops/action-gh-release@v1
        with:
          files: tt-*.tar.gz
```

**Alternative: Use `cargo-binstall` friendly releases**

Add to Cargo.toml:
```toml
[package.metadata.binstall]
pkg-url = "{ repo }/releases/download/v{ version }/{ name }-{ target }.tar.gz"
bin-dir = "{ bin }{ binary-ext }"
```

---

### 4.3 Platform-Specific Considerations

| Platform | Considerations |
|----------|----------------|
| **Windows** | - Use `.exe` extension<br>- Handle UNC paths with `dunce` crate<br>- Consider Windows Terminal color support |
| **macOS** | - Sign binaries for Gatekeeper (optional for CLI)<br>- Support both x86_64 and Apple Silicon<br>- Consider Homebrew formula |
| **Linux** | - Prefer static linking (musl) for portability<br>- Consider .deb and .rpm packages<br>- AUR package for Arch users |

**Homebrew formula example:**
```ruby
class Tt < Formula
  desc "Git-friendly personal task tracking CLI"
  homepage "https://github.com/yourusername/tt"
  url "https://github.com/yourusername/tt/archive/v0.1.0.tar.gz"
  
  depends_on "rust" => :build
  
  def install
    system "cargo", "install", *std_cargo_args
  end
  
  test do
    system "#{bin}/tt", "--version"
  end
end
```

---

## 5. Alternative Approaches

### 5.1 SQLite Cache for Large Workspaces (v0.3)

**When to consider:**
- Workspace has 1000+ task files
- Report generation takes > 1 second
- Need complex queries (filtering, sorting)

**Implementation approach:**
```rust
// Keep TOML as source of truth, SQLite as read cache
pub struct CachedWorkspace {
    fs_backend: FileSystemBackend,
    cache: SqliteConnection,
}

impl CachedWorkspace {
    pub fn sync(&mut self) -> Result<(), CacheError> {
        // Watch for file changes, update cache
        // Use mtime or content hash for change detection
    }
    
    pub fn query_tasks(&self, filter: TaskFilter) -> Vec<Task> {
        // Query SQLite for fast filtering
        // Return hydrated Task objects
    }
}
```

**Key crates:**
- `rusqlite` v0.31.x - SQLite bindings
- `notify` v6.x - File system watching

---

### 5.2 File Locking Strategies

**Comparison of approaches:**

| Strategy | Pros | Cons |
|----------|------|------|
| **Exclusive locks (fs2)** | Simple, prevents corruption | Blocks concurrent reads |
| **Read-write locks** | Allows concurrent reads | More complex implementation |
| **Optimistic locking** | No blocking | May fail on write, requires retry |
| **Lock-free (atomic writes)** | Maximum concurrency | Complex, platform-specific |

**Recommended for v0.1:** Exclusive locks with `fs2`

**Upgrade path for v0.2:**
```rust
use parking_lot::RwLock;

pub struct TaskStore {
    tasks: RwLock<HashMap<u64, Task>>,
}

impl TaskStore {
    pub fn read_task(&self, id: u64) -> Option<Task> {
        // Multiple readers allowed
        self.tasks.read().get(&id).cloned()
    }
    
    pub fn write_task(&self, task: Task) {
        // Exclusive write access
        self.tasks.write().insert(task.id, task);
    }
}
```

---

### 5.3 Editor Integration Approaches

**Option 1: LSP Server (Advanced)**
```rust
// tt-lsp binary
// Provides completion, hover info for task IDs in editors
```

**Option 2: Vim/Neovim Plugin**
```lua
-- Lua plugin that calls `tt` CLI
vim.keymap.set('n', '<leader>ta', ':silent !tt add<CR>')
```

**Option 3: VS Code Extension**
```typescript
// Extension that wraps tt CLI
// Provides UI for task management
```

**Recommended for v0.1:** Focus on CLI only, design output for easy parsing by editor plugins later.

**Design for extensibility:**
```rust
// Always support machine-readable output
// tt list --format json
// tt report --format markdown
```

---

## 6. Recommended Dependency Versions

### Complete `Cargo.toml` for v0.1

```toml
[package]
name = "tt"
version = "0.1.0"
edition = "2021"
rust-version = "1.75"
license = "MIT OR Apache-2.0"
description = "Git-friendly personal task tracking CLI"
repository = "https://github.com/yourusername/tt"
keywords = ["cli", "task", "productivity"]
categories = ["command-line-utilities"]

[dependencies]
# CLI
clap = { version = "4.5", features = ["derive", "env"] }
clap_complete = "4.5"

# TOML handling
toml_edit = "0.22"
toml_datetime = "0.6"

# Markdown
pulldown-cmark = "0.10"
comrak = "0.24"

# Date/time
chrono = { version = "0.4", features = ["serde"] }

# Templates
minijinja = { version = "2.0", features = ["loader"] }

# Regex
regex = "1.10"
once_cell = "1.19"

# File operations
fs2 = "0.12"
dirs = "5.0"
memmap2 = "0.9"

# Error handling
thiserror = "1.0"
anyhow = "1.0"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

[dev-dependencies]
# Testing
insta = { version = "1.38", features = ["yaml", "json"] }
tempfile = "3.10"
assert_cmd = "2.0"
predicates = "3.1"

# Benchmarking (optional)
criterion = "0.5"

[[bench]]
name = "benchmarks"
harness = false

[profile.release]
lto = true
codegen-units = 1
strip = true
panic = "abort"

[profile.dev]
debug = 1
```

---

## Summary & Next Steps

### Recommended Stack Summary

| Category | Choice | Rationale |
|----------|--------|-----------|
| CLI Framework | clap 4.5 | Industry standard, derive macros |
| TOML | toml_edit 0.22 | Preserves formatting for user edits |
| Markdown | pulldown-cmark 0.10 | Fast scanning |
| Date/Time | chrono 0.4 | ISO-8601 week support |
| Templates | minijinja 2.0 | Fast, Jinja2-compatible |
| Testing | insta 1.38 | Snapshot testing for outputs |
| File Locking | fs2 0.12 | Cross-platform exclusive locks |

### Implementation Order

1. **Phase 1:** Core data model + TOML storage
2. **Phase 2:** CLI skeleton + `tt add` command
3. **Phase 3:** ID generation + file locking
4. **Phase 4:** Markdown scanning + linking
5. **Phase 5:** Weekly report generation
6. **Phase 6:** Testing + documentation
7. **Phase 7:** Distribution setup

### Risks to Monitor

- **toml_edit API changes** - Pin exact version, monitor changelog
- **chrono maintenance** - Have migration path to `time` crate ready
- **Cross-platform testing** - Test on all target platforms before release

---

*Research completed: 2026-03-28*
*Author: Qwen Code Research Agent*
