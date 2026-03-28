# Task 3.2 Summary: Template System

**Status:** ✅ Completed  
**Date:** 2026-03-28  
**Phase:** 03-distribution (v0.3)

---

## Overview

Implemented a flexible template system for customizing weekly reports and daily logs in the `tt` CLI. The system supports embedded default templates, filesystem-based custom templates, and graceful fallback handling.

---

## What Was Implemented

### 1. Embedded Default Templates

Created two Jinja2-style templates embedded directly into the binary:

- **`src/reports/templates/weekly_report.j2`** — Default weekly report template
- **`src/reports/templates/daily_log.j2`** — Default daily log template

These templates are compiled into the binary using the `include_dir` crate, ensuring they're always available even without custom templates.

### 2. Template Loading Module

Created **`src/reports/templates.rs`** with the following functionality:

- **`TemplateType`** enum — Identifies template types (WeeklyReport, DailyLog)
- **`load_template()`** — Loads templates from filesystem with fallback to embedded
- **`get_template_path()`** — Resolves template paths from config or defaults
- **`render_template()`** — Renders templates with context data
- **`load_and_render_template()`** — Combined load + render with error handling
- **`validate_template()`** — Validates template syntax

**Key Features:**
- Graceful fallback to embedded templates on any error
- Support for custom template paths via configuration
- Debug logging for template loading decisions

### 3. Configuration Support

Updated **`src/models/config.rs`** to add:

```toml
[reports]
template_path = "templates/weekly_report.j2"      # Optional custom weekly template
log_template_path = "templates/daily_log.j2"      # Optional custom daily log template
```

Both fields are optional and default to `templates/<template_name>.j2` if not specified.

### 4. Report Integration

Enhanced **`src/reports/weekly.rs`** with:

- **`WeeklyReportContext`** — Serializable context struct for template rendering
- **`LogContext`** — Log data struct for templates
- **`build_context()`** — Converts report data to template context
- **`render_with_template()`** — Renders with provided template string
- **`render_from_workspace()`** — Loads and renders from workspace configuration

The original `render()` method remains unchanged for backward compatibility.

### 5. Documentation

Created comprehensive **`docs/templates.md`** covering:

- Template variables reference
- Configuration options
- Jinja2 syntax guide
- Troubleshooting tips
- Example templates (minimal, detailed, filtered)

---

## Template Variables Available

### Weekly Report Variables

| Variable | Type | Description |
|----------|------|-------------|
| `week` | `WeekRange` | Week info (start, end, iso_week) |
| `project` | `String` | Project slug |
| `done_tasks` | `Vec<Task>` | Completed tasks |
| `in_progress_tasks` | `Vec<Task>` | In-progress tasks |
| `blocked_tasks` | `Vec<Task>` | Blocked tasks |
| `mentioned_tasks` | `HashMap<String, Vec<String>>` | Task IDs mentioned in logs |
| `missing_tasks` | `HashMap<String, Vec<String>>` | Non-existent task IDs in logs |
| `logs` | `Vec<LogContext>` | Daily log highlights |

### Task Fields

- `id`, `title`, `status`, `created_at`, `updated_at`
- `due`, `done_at`, `priority`, `tags`, `notes`
- `blocked_reason`, `estimate`

### LogContext Fields

- `date` — Log date
- `highlights` — Array of highlight strings

---

## Files Created/Modified

### Created

| File | Purpose |
|------|---------|
| `src/reports/templates.rs` | Template loading module |
| `src/reports/templates/weekly_report.j2` | Embedded weekly template |
| `src/reports/templates/daily_log.j2` | Embedded daily template |
| `docs/templates.md` | Template user guide |
| `planning/phases/03-distribution/TASK_3_2_SUMMARY.md` | This summary |

### Modified

| File | Changes |
|------|---------|
| `Cargo.toml` | Added `include_dir = "0.7"` dependency |
| `src/reports/mod.rs` | Exported templates module and types |
| `src/reports/weekly.rs` | Added template rendering methods |
| `src/models/config.rs` | Added `template_path` and `log_template_path` fields |

---

## How to Customize Templates

### Quick Start

1. Create `templates/` directory in workspace root:
   ```bash
   mkdir templates
   ```

2. Copy default template as starting point:
   ```bash
   # Templates are embedded, so create your own
   touch templates/weekly_report.j2
   ```

3. Edit template with Jinja2 syntax (see `docs/templates.md`)

4. (Optional) Configure custom path in `tt.toml`:
   ```toml
   [reports]
   template_path = "templates/weekly_report.j2"
   ```

### Template Loading Order

1. Check `tt.toml` for `template_path` config
2. Try to load from configured path
3. Fallback to `templates/<name>.j2` if no config
4. Use embedded template if file not found
5. Error only if both custom and embedded fail

---

## Dependencies Added

```toml
[dependencies]
include_dir = "0.7"  # Embed default templates in binary
```

---

## Testing

The template module includes comprehensive tests:

- `test_template_type_file_name` — Verify file names
- `test_embedded_template_exists` — Verify embedded templates exist
- `test_load_template_fallback_to_embedded` — Test fallback behavior
- `test_load_template_uses_custom` — Test custom template loading
- `test_render_template_basic` — Test basic rendering
- `test_render_template_with_filter` — Test filter usage
- `test_validate_template_valid` — Test validation (valid)
- `test_validate_template_invalid` — Test validation (invalid)

Run tests with:
```bash
cargo test --package tt --lib reports::templates
```

---

## Acceptance Criteria Status

| Criteria | Status |
|----------|--------|
| Default templates embedded in binary | ✅ |
| Custom templates loaded from `templates/` if present | ✅ |
| Config options in `tt.toml` for template paths | ✅ |
| All template variables documented | ✅ |
| Error handling for invalid templates (graceful fallback) | ✅ |

---

## Next Steps

1. **Task 3.1** — (Parallel) Enhanced report intelligence
2. **Task 3.3** — (Parallel) Search and indexing
3. **Future** — Consider adding:
   - More built-in filters
   - Template inheritance support
   - Interactive template editor
   - Template preview command

---

## Notes

- The existing `render()` method is preserved for backward compatibility
- Template errors never crash the CLI — always fallback to embedded
- All template variables match existing data structures (no breaking changes)
- The `minijinja` crate was already in use; no new template engine needed
