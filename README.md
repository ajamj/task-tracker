# tt — Git-friendly personal task tracking CLI

[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)
[![Crates.io](https://img.shields.io/crates/v/tt.svg)](https://crates.io/crates/tt)

**tt** is a local-first, plain-text task tracking CLI tool written in Rust. It stores tasks in TOML files and daily logs in Markdown, all within a dedicated Git repository. Generate weekly reports and get git commit/branch suggestions (but tt never executes git commands automatically).

## Features

- **Per-file task storage** — One task = one TOML file (`tt-000001.toml`)
- **Markdown daily logs** — Write worklogs with auto-linking to tasks
- **Weekly reports** — Auto-generated Markdown reports (committed to Git)
- **Git-native UX** — Branch/commit message suggestions for traceability
- **Cross-platform** — Works on Windows, macOS, and Linux
- **Plain-text safe** — Handles manual edits gracefully (warn + continue)

## Quickstart

```bash
# Create a new workspace
mkdir my-worklog && cd my-worklog
git init
tt init

# Add your first task
tt add "Refactor config loader" --due 2026-04-03 --tag rust --tag cli

# Start working
tt ls
tt start tt-000001

# Log your work (auto-detects task IDs)
tt log "Worked on tt-000001: initial implementation"

# Complete the task
tt done tt-000001

# Generate weekly report
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

## Task Format

Each task is a TOML file:

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

## Daily Log Format

Logs are Markdown files with sections:

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

Mention `tt-000001` anywhere in the log, and it will be auto-linked in weekly reports.

## Weekly Report

Generated reports include:

- **Done** — Tasks completed this week (by `done_at` date)
- **In Progress** — Tasks currently in `doing` status
- **Blocked** — Tasks with `blocked` status
- **Mentioned in Logs** — Task IDs detected in daily logs
- **Missing tasks referenced in logs** — Warn if task ID has no TOML file
- **Worklog Highlights** — Extracted bullets from log sections

## Git Philosophy

**Suggestions only** — tt never runs `git` commands automatically. After each operation, tt prints suggested commands:

```
Git suggestions (not executed):
  Suggested branch: work/tt-000001-refactor-config-loader
  Suggested commit: task(add): tt-000001 Refactor config loader
  Files changed:
    - projects/work/tasks/2026/03/tt-000001.toml

  git add -A
  git checkout -b work/tt-000001-refactor-config-loader
  git commit -m "task(add): tt-000001 Refactor config loader"
```

## Installation

### From Source

```bash
cargo install --path .
```

### From crates.io (coming soon)

```bash
cargo install tt
```

## Configuration

### tt.toml (Root Config)

```toml
version = 1

[workspace]
default_project = "work"
week_starts_on = "monday"  # Monday start (ISO-8601)
task_id_prefix = "tt-"
task_id_width = 6

[storage]
projects_dir = "projects"

[reports]
track_in_git = true
weekly_dir = "reports/weekly"

[git]
suggest_branch = true
suggest_commit = true
```

## Roadmap

### v0.1 — MVP (Current)
- [x] Project scaffolding
- [x] Data models + error types
- [ ] Task CRUD + status transitions
- [ ] Daily logs + auto-linking
- [ ] Weekly report generation
- [ ] Git suggestions

### v0.2 — Dashboard + Better UX
- Local web dashboard (`tt dashboard`)
- API layer for GUI integration
- Improved CLI formatting (colors, tables)
- Link sync command (opt-in)

### v0.3 — Polish + Distribution
- GitHub Releases binaries
- Template customization
- Search + indexing
- Enhanced report intelligence

## Development

### Build

```bash
cargo build
```

### Test

```bash
cargo test
cargo insta test  # Snapshot tests
```

### Lint

```bash
cargo clippy -- -D warnings
cargo fmt --check
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Run tests: `cargo test`
4. Run clippy: `cargo clippy -- -D warnings`
5. Commit your changes (`git commit -m 'Add amazing feature'`)
6. Push to the branch (`git push origin feature/amazing-feature`)
7. Open a Pull Request

---

**Status:** 🚧 Under development (v0.1 in progress)
