# tt — Git-Friendly Personal Task Tracking CLI

## Project Overview

**`tt`** is a local-first, plain-text task tracking CLI tool written in Rust. It stores tasks in TOML files and daily logs in Markdown within a dedicated Git repository. The tool generates weekly reports and provides Git commit/branch suggestions (but never executes Git commands automatically).

### Core Features

- **Per-file task storage** — One task = one TOML file (`tt-000001.toml`)
- **Markdown daily logs** — Worklogs with auto-linking to tasks (`tt-\d{6}` pattern)
- **Weekly reports** — Auto-generated Markdown reports
- **Git-native UX** — Branch/commit message suggestions for traceability
- **Multi-project workspace** — Support for multiple projects (e.g., work, personal)
- **Cross-platform** — Windows, macOS, Linux
- **Plain-text safe** — Handles manual edits gracefully (warn + continue)

### Tech Stack

| Component | Library | Version |
|-----------|---------|---------|
| **CLI Framework** | `clap` (derive) | 4.5 |
| **TOML** | `toml_edit` | 0.22 |
| **Date/Time** | `chrono` | 0.4 |
| **Markdown** | `pulldown-cmark` | 0.10 |
| **Templates** | `minijinja` | 2.0 |
| **Error Handling** | `thiserror` + `anyhow` | 1.0 |
| **Testing** | `insta` (snapshot) | 1.38 |
| **Logging** | `tracing` + `tracing-subscriber` | 0.1 / 0.3 |

## Project Structure

```
D:\GRC-Ajam\rust-playground\
├── Cargo.toml                 # Project manifest
├── README.md                  # User documentation
├── SETUP.md                   # Setup instructions
├── CHANGELOG.md               # Version history
├── LICENSE                    # MIT OR Apache-2.0
├── .gitignore                 # Git ignore patterns
├── planning/                  # Project planning docs
│   ├── PROJECT.md             # Vision & tech stack
│   ├── REQUIREMENTS.md        # Detailed requirements
│   ├── ROADMAP.md             # Phase timeline
│   ├── RESEARCH.md            # Technical research
│   ├── STATE.md               # Current development state
│   └── phases/01-mvp/         # MVP phase docs
├── src/
│   ├── lib.rs                 # Library root
│   ├── main.rs                # CLI entry point
│   ├── error.rs               # Error types (TtError, StorageError)
│   ├── cli/
│   │   ├── mod.rs
│   │   ├── args.rs            # CLI argument definitions (clap)
│   │   ├── commands.rs        # Command execution logic
│   │   └── format.rs          # Output formatting
│   ├── models/
│   │   ├── mod.rs
│   │   ├── config.rs          # WorkspaceConfig, ProjectConfig
│   │   ├── task.rs            # Task, TaskStatus, Priority, NewTask
│   │   └── week.rs            # WeekRange (ISO-8601, Monday start)
│   ├── storage/
│   │   ├── mod.rs
│   │   ├── workspace.rs       # Workspace initialization
│   │   ├── task.rs            # Task file I/O, ID generation
│   │   └── log.rs             # Log file I/O, task ID scanning
│   └── reports/
│       ├── mod.rs
│       └── weekly.rs          # Weekly report generation
└── tests/
    └── integration_tests.rs   # Integration tests
```

## Building and Running

### Prerequisites

- **Rust toolchain** (rustc, cargo) — https://rustup.rs/
- **Git** — For version control
- Minimum Rust version: **1.75**

### Build Commands

```bash
# Build (debug mode)
cargo build

# Build (release mode)
cargo build --release

# Run tests
cargo test

# Lint
cargo clippy -- -D warnings
cargo fmt --check
```

### Running the CLI

```bash
# Run directly (debug)
cargo run -- init
cargo run -- add "My task" --due 2026-04-03 --tag rust

# Or install globally
cargo install --path .
tt init
tt add "Refactor config loader" --due 2026-04-03 --tag rust --tag cli
```

### Available Commands

| Command | Description |
|---------|-------------|
| `tt init` | Initialize new workspace |
| `tt add <title>` | Create a new task |
| `tt ls` | List tasks (filter by `--status`, `--project`) |
| `tt show <id>` | Show task details |
| `tt start <id>` | Start working on a task |
| `tt done <id>` | Mark a task as complete |
| `tt log <text>` | Append to daily log |
| `tt report week [--week YYYY-Www]` | Generate weekly report |

### Example Usage

```bash
# Initialize workspace
mkdir my-worklog && cd my-worklog
git init
tt init

# Add a task
tt add "Refactor config loader" --due 2026-04-03 --tag rust --tag cli

# Start working
tt ls
tt start tt-000001

# Log work (auto-detects task IDs)
tt log "Worked on tt-000001: initial implementation"

# Complete task
tt done tt-000001

# Generate report
tt report week
```

## Workspace Layout

```
my-worklog/
├── tt.toml                      # Root config
├── projects/
│   ├── work/
│   │   ├── project.toml         # Project config
│   │   ├── tasks/
│   │   │   └── 2026/03/tt-000001.toml
│   │   ├── logs/
│   │   │   └── 2026/2026-03-28.md
│   │   └── reports/
│   │       └── weekly/2026-W13.md
│   └── personal/
│       └── ...
└── README.md
```

## Task Format (TOML)

```toml
version = 1
id = "tt-000001"
title = "Refactor config loader"
status = "todo"  # todo | doing | done | blocked | canceled

created_at = "2026-03-28"
updated_at = "2026-03-28"

due = "2026-04-03"
priority = "P2"  # P0 | P1 | P2 | P3
tags = ["rust", "cli"]
notes = """
Context and details here.
"""

[git_suggestions]
branch = "work/tt-000001-refactor-config-loader"
commit_add = "task(add): tt-000001 Refactor config loader"
commit_start = "task(start): tt-000001 Refactor config loader"
commit_done = "task(done): tt-000001 Refactor config loader"
```

## Daily Log Format (Markdown)

```markdown
# 2026-03-28 (work)

## Highlights
- tt-000001: Initial implementation

## Done
-

## Doing
- tt-000001: Refactor config loader

## Notes
- Discussed approach with team.
```

## Development Conventions

### Code Style

- **Formatting**: `cargo fmt` (default Rust formatting)
- **Linting**: `cargo clippy -- -D warnings` (deny all warnings)
- **Error handling**: Use `thiserror` for library errors, `anyhow` for application
- **Logging**: Use `tracing` macros (`tracing::info!`, `tracing::debug!`, etc.)

### Testing Practices

- **Unit tests**: Co-located with source code (in `#[cfg(test)]` modules)
- **Integration tests**: In `tests/` directory
- **Snapshot testing**: Using `insta` for CLI output and reports
- **Test naming**: Descriptive function names (e.g., `test_cli_parse_add_with_options`)

### Git Workflow

1. Create feature branch: `git checkout -b feature/description`
2. Make changes with tests
3. Run `cargo test` and `cargo clippy -- -D warnings`
4. Commit with conventional commits format:
   - `feat: add new feature`
   - `fix: fix bug`
   - `docs: update documentation`
   - `test: add tests`
5. Push and create PR

### Commit Message Format

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

## Key Design Principles

1. **Suggestions only** — Never run `git` commands automatically
2. **Plain-text safe** — Handle manual edits gracefully (warn + continue)
3. **Deterministic** — Same workspace state → same report output
4. **Fast** — Regex scanning with `once_cell` for efficiency
5. **Cross-platform** — Windows, macOS, Linux from day one

## Current State (as of 2026-03-28)

**Phase**: 01-mvp (implementation-complete)

### Completed Features

- [x] All 8 CLI commands (`init`, `add`, `ls`, `show`, `start`, `done`, `log`, `report`)
- [x] Git suggestions for all commands
- [x] Weekly report generation with all sections
- [x] Auto-linking (task ID detection in logs)
- [x] Status transition validation
- [x] File locking for ID generation
- [x] ISO-8601 week calculations (Monday start)
- [x] Multi-project support
- [x] 57+ unit tests

### Code Statistics

- **Total Files**: 16 Rust source files
- **Total Lines**: ~3,200 lines of Rust
- **Total Tests**: 57+ unit tests

## Roadmap

### v0.1 — MVP (Current)
- [x] Project scaffolding
- [x] Data models + error types
- [x] Task CRUD + status transitions
- [x] Daily logs + auto-linking
- [x] Weekly report generation
- [x] Git suggestions

### v0.2 — Dashboard + Better UX
- [ ] Local web dashboard (`tt dashboard`)
- [ ] API layer for GUI integration
- [ ] Improved CLI formatting (colors, tables)
- [ ] Link sync command (opt-in)

### v0.3 — Polish + Distribution
- [ ] GitHub Releases binaries
- [ ] Template customization
- [ ] Search + indexing
- [ ] Enhanced report intelligence

## Troubleshooting

### Git Not Found
Use full path: `C:\Program Files\Git\bin\git.exe`

### Build Errors
Check Rust version: `rustc --version` (minimum 1.75)

### Test Workspace
```bash
mkdir test-worklog && cd test-worklog
tt init
tt add "Test task"
tt ls
```

## References

- `planning/PROJECT.md` — Project vision and tech stack
- `planning/REQUIREMENTS.md` — Detailed requirements
- `planning/ROADMAP.md` — Phase timeline
- `planning/RESEARCH.md` — Technical research
- `planning/STATE.md` — Current development state
- `SETUP.md` — Setup instructions
