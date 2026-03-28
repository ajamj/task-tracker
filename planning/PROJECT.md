# Project Vision

## What We're Building

**`tt`** — A Git-friendly personal task tracking CLI tool written in Rust.

Core capabilities:
- Per-file TOML task storage (`tt-000001.toml`)
- Markdown daily logs with auto-linking (`tt-\d{6}` pattern detection)
- Weekly report generation (committed to Git)
- Multi-project workspace in a dedicated Git repo
- Git integration via **suggestions only** (branch names, commit messages)

## Target User

**Developers** — Technical users who want:
- Local-first, plain-text storage they control
- Git-native workflow without automation surprises
- Fast capture → execution → daily logs → weekly reports
- Cross-platform CLI (Windows, macOS, Linux)

## Problem Statement

Existing task trackers either:
- Require a server/database (not local-first)
- Lack Git integration for commit traceability
- Don't generate committed weekly reports
- Are too heavy for personal, focused use

`tt` solves this by being a lightweight, Git-friendly CLI that keeps everything in plain text (TOML + Markdown) within a dedicated Git repo.

## Tech Stack

Based on research findings:

| Component | Choice | Rationale |
|-----------|--------|-----------|
| **Language** | Rust | Performance, safety, cross-platform CLI |
| **CLI Framework** | `clap` 4.5 (derive) | Industry standard, excellent UX |
| **TOML** | `toml_edit` 0.22 | Preserves comments/format on manual edits |
| **Markdown** | `pulldown-cmark` 0.10 + `comrak` 0.24 | Parsing + rendering |
| **Date/Time** | `chrono` 0.4 | ISO-8601, week calculations (Monday start) |
| **Templates** | `minijinja` 2.0 | Lightweight report templating |
| **Testing** | `insta` 1.38 | Snapshot testing for reports/CLI output |
| **Error Handling** | `thiserror` + `anyhow` | Library + application error patterns |
| **File Locking** | `fs2` | Safe concurrent task ID generation |

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

## Key Design Principles

1. **Suggestions only** — Never run `git` commands automatically
2. **Plain-text safe** — Handle manual edits gracefully (warn + continue)
3. **Deterministic** — Same workspace state → same report output
4. **Fast** — Regex scanning with `once_cell` + `memmap2` for large logs
5. **Cross-platform** — Windows, macOS, Linux from day one
