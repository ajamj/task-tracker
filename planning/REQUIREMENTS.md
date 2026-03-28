# Requirements

## Phase 1: MVP (v0.1) — "CLI core + git-friendly reports"

**Goal:** A personal user can run a dedicated repo with multiple projects, track tasks per-file, keep daily logs, and generate a weekly report that's committed.

### Core Features

#### R1.1: Workspace + Config
- `tt init` — Create workspace structure (`tt.toml`, default project, sample task/log/report)
- `tt project add <name>` — Add a new project
- `tt project ls` — List all projects
- `tt project set-default <name>` — Set default project

#### R1.2: Task Management (per-file TOML)
- `tt add <title>` — Create a new task (auto-generate ID: `tt-000001`)
- `tt ls` — List tasks by status (todo, doing, blocked, done)
- `tt show <id>` — Show task details
- `tt start <id>` — Transition task to `doing`, set `started_at`
- `tt done <id>` — Transition task to `done`, set `done_at`
- Status transitions: `todo → doing → done`, `todo|doing → blocked`, `any → canceled`

#### R1.3: Daily Logs (Markdown)
- `tt log <text>` — Append text to today's log (create if missing)
- `tt log --edit` — Open today's log in editor
- Auto-detect task IDs (`tt-\d{6}`) in log entries
- Default log template with sections: Highlights, Done, Doing, Blocked, Notes

#### R1.4: Weekly Report Generation
- `tt report week` — Generate weekly report (Monday–Sunday range)
- Report sections:
  - **Done** — Tasks with `done_at` in the week range
  - **In Progress** — Tasks with `status=doing`
  - **Blocked** — Tasks with `status=blocked`
  - **Mentioned in Logs** — Task IDs detected in logs (auto-linking)
  - **Missing tasks referenced in logs** — Warn if `tt-XXXXXX` has no TOML file
  - **Worklog Highlights** — Extracted bullets from log sections
- Report saved to `projects/<project>/reports/weekly/YYYY-Www.md`

#### R1.5: Git Suggestions (No Execution)
- Print suggested branch name: `<project>/<id>-<slug>`
- Print suggested commit message: `<type>: <id> <title>`
- Print list of modified files
- Show reminder commands: `git status`, `git add -A`, `git commit -m "..."`

#### R1.6: Auto-Linking
- Scan logs for `tt-\d{6}` pattern (configurable width)
- Build mapping: `task_id → [dates...]` for the week
- Include in weekly report even if task status is not `done`
- Detect missing tasks (ID mentioned but no TOML file exists)

### Quality Bar / Acceptance Criteria

- [ ] Works on Windows, macOS, Linux
- [ ] Handles manual TOML/log edits without crashing (warn + continue)
- [ ] Report generation is deterministic for same workspace state
- [ ] Week starts on Monday (ISO-8601)
- [ ] All CLI commands have `--help` documentation
- [ ] Error messages are clear and actionable

### Tests Required

- [ ] Week range calculation (Monday start, edge cases for year boundaries)
- [ ] Task ID generation (incremental, no collisions)
- [ ] Log task-ID extraction (regex, deduplication)
- [ ] Report snapshot tests (fixed input → fixed output)
- [ ] Status transition validation
- [ ] Timestamp update rules (`updated_at` on every change)

---

## Phase 2: Dashboard + Better UX (v0.2)

**Goal:** Introduce optional GUI dashboard for users who prefer visuals, without changing storage model.

### Features

#### R2.1: Local Dashboard
- `tt dashboard` — Start local web server (`localhost:PORT`)
- Views: Today, Doing, Upcoming, Done
- Task detail view (read-only in v0.2, read-write stretch goal)

#### R2.2: API Layer (if web UI)
- `GET /api/projects` — List projects
- `GET /api/tasks?project=<slug>&status=<status>` — List tasks
- `GET /api/task/<id>` — Get task details
- `POST /api/task` — Create task
- `POST /api/task/<id>/status` — Update status (start/done/block)
- `POST /api/report/week` — Generate weekly report

#### R2.3: Improved CLI UX
- Better formatting (colors, tables via `comfy-table` or `tabled`)
- Filtering: `tt ls --due overdue|today|week`
- Sorting: by priority, due date, created date
- `--json` output for scripting/dashboard integration

#### R2.4: Linking Support (Write Opt-In)
- `tt link sync --project <p> --from <date> --to <date>`:
  - Scan logs in range
  - Write `refs += ["log:YYYY-MM-DD"]` into task TOML
- Keep `tt report week` as "no surprise writes" by default

### Quality Bar

- [ ] Dashboard binds to `localhost` only (security)
- [ ] CLI remains first-class (dashboard is optional)
- [ ] README includes screenshots + "how to run dashboard" steps
- [ ] API documented with OpenAPI spec (optional)

---

## Phase 3: Polish, Distribution, Power Features (v0.3)

**Goal:** Make it "installable and sticky" for open-source adoption and daily use.

### Features

#### R3.1: Distribution
- GitHub Releases binaries for Windows, macOS, Linux
- Installation docs: `cargo install`, `cargo-binstall`, Homebrew tap
- `tt doctor` — Diagnose common issues (optional)

#### R3.2: Templates + Customization
- Report template customization via config (simple placeholders)
- Log template customization
- Custom status names (stretch goal)

#### R3.3: Search + Indexing
- Fast scan/index for large workspaces (cache file optional)
- `tt search "<text>"` — Search across tasks and logs

#### R3.4: Enhanced Report Intelligence
- "Mentioned in Logs" merges into Done/Doing sections when matching status
- Better highlights extraction (section-based + standalone markers)
- Configurable highlight limits per day/section

#### R3.5: Quality Hardening
- Extensive fixture tests + snapshot tests
- Clear error messages with suggestions
- Performance benchmarks for large workspaces

### Stretch Goals

- [ ] Export: `tt export --format csv|json`
- [ ] Import: Minimal CSV import (Todoist-like format)
- [ ] Recurring tasks (simple patterns: daily, weekly, monthly)
- [ ] Time tracking: `tt start --timer` with duration logging

---

## Cross-Cutting Requirements

### Non-Functional

- **Performance:** Log scanning for 1000+ files < 1 second
- **Robustness:** Handle malformed TOML gracefully (parse error → warn + skip)
- **Cross-platform:** All paths use `std::path::PathBuf`, no hardcoded separators
- **Safety:** File locking for ID generation (prevent collisions)

### Documentation

- README with quickstart (< 5 minutes)
- Workspace layout explanation
- Task format examples
- Git philosophy ("suggestions only")
- Roadmap visibility (v0.1/v0.2/v0.3)

### Testing Strategy

- Unit tests: ID generation, date calculations, regex parsing
- Integration tests: CLI commands end-to-end
- Snapshot tests: Report output, CLI formatting
- Property-based tests (optional): `proptest` for edge cases
