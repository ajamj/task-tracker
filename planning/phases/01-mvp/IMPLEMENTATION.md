# Implementation Plan: Phase 1 MVP

**Phase:** v0.1 — CLI core + git-friendly reports  
**Created:** 2026-03-28  
**Status:** Ready for implementation

---

## Task Grouping

### Wave 1: Foundation (Tasks 1-3)

**Goal:** Establish project structure and core data types

| Task | Description | Dependencies |
|------|-------------|--------------|
| 1.1 | Project scaffolding + `Cargo.toml` with all v0.1 dependencies | None |
| 1.2 | Data models: `Task`, `TaskStatus`, `Priority`, `NewTask` builder | 1.1 |
| 1.3 | Error types: `TtError`, `StorageError` with `thiserror` | 1.2 |

**Why together:** These form the foundation—nothing else can be built without them. Data models are pure structs with no I/O, making them easy to test in isolation.

---

### Wave 2: Storage Layer (Tasks 4-7)

**Goal:** Enable reading/writing workspace, projects, tasks, and logs

| Task | Description | Dependencies |
|------|-------------|--------------|
| 2.1 | Workspace loading + config parsing (`tt.toml`, `project.toml`) | 1.3 |
| 2.2 | Project discovery + `Project` struct initialization | 2.1 |
| 2.3 | Task file I/O: read/write TOML with `toml_edit` | 1.2, 2.2 |
| 2.4 | ID generation with file locking (`fs2` crate) | 2.3 |
| 2.5 | Log file I/O: create/append markdown files | 2.2 |
| 2.6 | Log template management (default template with sections) | 2.5 |

**Why together:** All storage operations share the same patterns: load config → resolve paths → read/write files → handle errors. Building them together ensures consistent error handling and path resolution logic.

---

### Wave 3: CLI Commands - Basic (Tasks 8-10)

**Goal:** Implement core task management commands

| Task | Description | Dependencies |
|------|-------------|--------------|
| 3.1 | `tt init` command: create workspace structure + sample files | 2.1, 2.6 |
| 3.2 | `tt add` command: create task with auto-generated ID | 2.4, 3.1 |
| 3.3 | `tt ls` command: list tasks by status (plain text formatting) | 2.3 |
| 3.4 | `tt show <id>` command: display task details | 2.3 |

**Why together:** These are the most frequently used commands. They all follow the pattern: parse args → load workspace → execute storage operation → format output.

---

### Wave 4: CLI Commands - Status Transitions (Tasks 11-12)

**Goal:** Implement task lifecycle commands with validation

| Task | Description | Dependencies |
|------|-------------|--------------|
| 4.1 | `tt start <id>` command: transition to `doing`, set `started_at` | 3.4, 1.2 |
| 4.2 | `tt done <id>` command: transition to `done`, set `done_at` | 4.1 |
| 4.3 | Status transition validation (`can_transition_to` logic) | 1.2 |

**Why together:** Both commands use the same update pattern and share transition validation logic. Testing them together ensures all state transitions work correctly.

---

### Wave 5: Logs + Linking (Tasks 13-15)

**Goal:** Enable daily logging with auto-linking detection

| Task | Description | Dependencies |
|------|-------------|--------------|
| 5.1 | `tt log <text>` command: append to today's log | 2.5, 2.6 |
| 5.2 | `tt log --edit` command: open log in `$EDITOR` | 5.1 |
| 5.3 | Task ID scanner: regex `tt-\d{6}` with `once_cell` cache | 1.3 |
| 5.4 | Auto-linking detection: print detected task IDs after log append | 5.3 |

**Why together:** Log creation and task ID scanning are tightly coupled—every log append should scan for task IDs. Building them together ensures the scanning is integrated from the start.

---

### Wave 6: Reports (Tasks 16-18)

**Goal:** Generate weekly reports with all required sections

| Task | Description | Dependencies |
|------|-------------|--------------|
| 6.1 | `WeekRange` struct: Monday-start week calculation (ISO-8601) | 1.2 |
| 6.2 | `tt report week` command: generate report structure | 2.3, 5.3, 6.1 |
| 6.3 | Highlights extraction: parse log sections (Highlights, Done, Doing, etc.) | 5.1 |
| 6.4 | Missing task detection: warn if log references non-existent task | 5.3, 2.3 |

**Why together:** Report generation depends on week calculation, task querying, log scanning, and highlights extraction. These must work together to produce a complete report.

---

### Wave 7: Git Suggestions + Polish (Tasks 19-20)

**Goal:** Add git-friendly output and error handling

| Task | Description | Dependencies |
|------|-------------|--------------|
| 7.1 | Git suggestions formatting: branch name, commit message, file list | 3.2, 4.1, 4.2 |
| 7.2 | Error message improvements: clear, actionable messages with suggestions | All waves |

**Why together:** Git suggestions are printed after most commands, so they need the core commands to be complete. Error handling improvements are iterative and benefit from seeing all error paths.

---

### Wave 8: Testing + Documentation (Tasks 21-23)

**Goal:** Ensure quality and usability

| Task | Description | Dependencies |
|------|-------------|--------------|
| 8.1 | Unit tests: ID generation, week calculation, status transitions, regex | Waves 1-6 |
| 8.2 | Integration tests: full CLI workflows (`init → add → start → done → log → report`) | All waves |
| 8.3 | Snapshot tests: `tt --help`, `tt ls`, weekly report output (using `insta`) | Waves 3-6 |
| 8.4 | README + quickstart documentation | All waves |

**Why together:** Testing should happen incrementally, but final test coverage and documentation are done after all features are implemented.

---

## Verification Strategy

### Wave 1: Foundation

**Files that should exist:**
- `src/lib.rs` — Library root with exports
- `src/error.rs` — Error type definitions
- `src/models/mod.rs`, `src/models/task.rs` — Data models

**Commands that should work:**
- `cargo build` — Compiles without errors
- `cargo clippy` — No warnings

**Tests that should pass:**
- `TaskStatus::can_transition_to()` — All valid/invalid transitions
- `NewTask::builder()` — Builder pattern creates correct structs

---

### Wave 2: Storage Layer

**Files that should exist:**
- `src/storage/mod.rs`, `src/storage/workspace.rs`, `src/storage/task.rs`, `src/storage/log.rs`
- `src/models/config.rs` — Config structs

**Commands that should work:**
- Manual test: Create `tt.toml` + `project.toml`, verify workspace loads
- Manual test: Create task TOML, verify it parses correctly

**Tests that should pass:**
- `Workspace::load()` — Loads valid workspace, errors on missing config
- `next_id()` — Generates incremental IDs with no collisions (concurrent test)
- `Log::new()` — Creates log with correct template structure

---

### Wave 3: CLI Commands - Basic

**Files that should exist:**
- `src/cli/mod.rs`, `src/cli/args.rs`, `src/cli/commands.rs`
- `src/main.rs` — Entry point with CLI dispatch

**Commands that should work:**
- `tt --help` — Shows all subcommands
- `tt init` — Creates workspace structure
- `tt add "Test task"` — Creates `tt-000001.toml`
- `tt ls` — Lists tasks (empty or with test data)
- `tt show tt-000001` — Shows task details

**Tests that should pass:**
- Integration: `tt init` creates `tt.toml`, `projects/`, `projects/work/`
- Integration: `tt add` creates task file with correct ID
- Snapshot: `tt --help` output

---

### Wave 4: Status Transitions

**Files that should exist:**
- `src/models/task.rs` — Updated with `started_at`, `done_at` fields
- `src/cli/commands.rs` — Updated with `start`, `done` commands

**Commands that should work:**
- `tt start tt-000001` — Sets status to `doing`, updates `started_at`
- `tt done tt-000001` — Sets status to `done`, updates `done_at`
- `tt start tt-000001` (already done) — Errors with clear message

**Tests that should pass:**
- Unit: `can_transition_to()` — All 8 valid transitions, all invalid rejected
- Integration: `tt start` → `tt show` shows `started_at` timestamp
- Integration: `tt done` → `tt show` shows `done_at` timestamp
- Integration: Invalid transition returns error with suggestion

---

### Wave 5: Logs + Linking

**Files that should exist:**
- `src/storage/log.rs` — Updated with append logic
- `src/linking/mod.rs`, `src/linking/scanner.rs`, `src/linking/regex.rs`

**Commands that should work:**
- `tt log "Worked on tt-000001"` — Creates/appends log, prints detected task
- `tt log --edit` — Opens editor (manual test)
- `tt log "Multiple: tt-000001 and tt-000002"` — Detects both IDs

**Tests that should pass:**
- Unit: `scan_for_task_ids()` — Extracts `tt-000001` from various formats
- Unit: `scan_for_task_ids()` — Deduplicates repeated IDs
- Integration: `tt log` creates file with correct date in name
- Integration: `tt log` appends without overwriting existing content

---

### Wave 6: Reports

**Files that should exist:**
- `src/models/week.rs` — `WeekRange` struct
- `src/reports/mod.rs`, `src/reports/weekly.rs`, `src/reports/highlights.rs`

**Commands that should work:**
- `tt report week` — Generates current week report
- `tt report week --week 2026-W13` — Generates specific week report
- Report includes: Done, In Progress, Blocked, Mentioned, Missing, Highlights

**Tests that should pass:**
- Unit: `WeekRange::from_date()` — Correct Monday start for any date
- Unit: `WeekRange::from_date()` — Edge cases (year boundaries, ISO week 1/52)
- Unit: `extract_highlights()` — Parses markdown sections correctly
- Integration: Report includes tasks with `done_at` in week range
- Integration: Report warns about missing tasks referenced in logs
- Snapshot: Full report output (deterministic with fixed test data)

---

### Wave 7: Git Suggestions + Polish

**Files that should exist:**
- `src/cli/format.rs` — Git suggestion formatting
- Updated command handlers to print suggestions

**Commands that should work:**
- `tt add "Task"` — Prints branch name + commit message suggestions
- `tt start tt-000001` — Prints commit suggestion for status change
- `tt done tt-000001` — Prints commit suggestion for completion
- All errors have clear, actionable messages

**Tests that should pass:**
- Unit: `slugify()` — Produces correct slugs from titles
- Unit: Git suggestion formatting — Correct branch/commit format
- Integration: All commands print suggestions (manual verification)

---

### Wave 8: Testing + Documentation

**Files that should exist:**
- `tests/cli_tests.rs` — Integration tests
- `tests/snapshots/` — `insta` snapshots
- `README.md` — Complete documentation
- `CHANGELOG.md` — Version history

**Commands that should work:**
- `cargo test` — All tests pass
- `cargo insta test` — Snapshot tests pass
- `cargo clippy -- -D warnings` — No clippy warnings

**Tests that should pass:**
- Full workflow: `init → add → start → log → done → report`
- Multi-project: Add tasks to different projects
- Edge case: Malformed TOML handled gracefully (warn + skip)
- All snapshot tests reviewed and accepted

---

## Implementation Order Summary

```
Wave 1 (Foundation)
  └─> Wave 2 (Storage Layer)
        └─> Wave 3 (CLI Basic)
        └─> Wave 4 (Status Transitions)
        └─> Wave 5 (Logs + Linking)
              └─> Wave 6 (Reports)
                    └─> Wave 7 (Git + Polish)
                          └─> Wave 8 (Testing + Docs)
```

**Critical path:** Waves 1 → 2 → 3 → 4 → 6 → 8  
**Parallel work:** Wave 5 (Logs) can start after Wave 2; Wave 7 (Git) can start after Wave 4

---

## Risk Mitigation

| Risk | Impact | Mitigation |
|------|--------|------------|
| File locking complexity | High | Use `fs2` crate, test concurrent ID generation early |
| Week calculation edge cases | Medium | Write tests first for year boundaries (ISO week 1/52) |
| TOML parsing errors | Medium | Use `toml_edit` for robust parsing, add error recovery |
| Regex performance | Low | Use `once_cell::Lazy` for compiled regex, benchmark with 1000+ logs |
| Cross-platform path issues | Medium | Use `PathBuf` everywhere, test on Windows early |

---

## Next Steps

1. Run `/gsd:plan-phase 1` to generate atomic implementation tasks
2. Start with Wave 1 (Foundation) — no dependencies, easy to verify
3. After each wave, run verification tests before proceeding
