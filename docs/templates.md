# Template System Guide

The `tt` CLI uses a flexible template system for generating weekly reports and daily logs. This guide explains how to customize templates to match your workflow.

## Overview

`tt` uses [Jinja2](https://jinja.palletsprojects.com/)-style templates (via the `minijinja` Rust crate) for report generation. You can:

- Use the **embedded default templates** (built into the binary)
- Create **custom templates** in your workspace
- Configure template paths in `tt.toml`

## Default Templates

By default, `tt` uses embedded templates that require no configuration. These are stored in the binary and used automatically.

### Default Template Locations

If you want to customize, create templates at:

```
my-worklog/
├── tt.toml
├── templates/
│   ├── weekly_report.j2    # Weekly report template
│   └── daily_log.j2        # Daily log template
└── projects/
    └── ...
```

## Weekly Report Template

### Template Variables

The following variables are available in weekly report templates:

| Variable | Type | Description |
|----------|------|-------------|
| `week` | `WeekRange` | Week information (start, end, iso_week) |
| `project` | `String` | Project slug/name |
| `done_tasks` | `Vec<Task>` | Tasks completed this week |
| `in_progress_tasks` | `Vec<Task>` | Tasks currently in progress |
| `blocked_tasks` | `Vec<Task>` | Tasks that are blocked |
| `mentioned_tasks` | `HashMap<String, Vec<String>>` | Task IDs mentioned in logs (with dates) |
| `missing_tasks` | `HashMap<String, Vec<String>>` | Task IDs in logs but no TOML file found |
| `logs` | `Vec<LogContext>` | Daily log highlights |

### WeekRange Fields

```jinja2
{{ week.start }}       // Week start date (e.g., "2026-03-24")
{{ week.end }}         // Week end date (e.g., "2026-03-30")
{{ week.iso_week }}    // ISO week identifier (e.g., "2026-W13")
```

### Task Fields

Each task in `done_tasks`, `in_progress_tasks`, or `blocked_tasks` has:

```jinja2
{{ task.id }}              // Task ID (e.g., "tt-000001")
{{ task.title }}           // Task title
{{ task.status }}          // Current status
{{ task.created_at }}      // Creation date
{{ task.updated_at }}      // Last update date
{{ task.due }}             // Due date (optional)
{{ task.done_at }}         // Completion date (optional)
{{ task.priority }}        // Priority level (optional)
{{ task.tags }}            // Tags array
{{ task.notes }}           // Additional notes
{{ task.blocked_reason }}  // Reason if blocked
{{ task.estimate }}        // Time estimate (optional)
```

### LogContext Fields

```jinja2
{{ log.date }}         // Log date (e.g., "2026-03-28")
{{ log.highlights }}   // Array of highlight strings
```

### Example: Default Weekly Report Template

```jinja2
# Weekly Report — {{ week.iso_week }} ({{ project }})
Range: {{ week.start }} to {{ week.end }}

## Summary
- Done (by done_at): {{ done_tasks|length }}
- In progress (current status): {{ in_progress_tasks|length }}
- Blocked (current status): {{ blocked_tasks|length }}
- Mentioned in logs: {{ mentioned_tasks|length }}
- Missing tasks referenced in logs: {{ missing_tasks|length }}

## Done
{% for task in done_tasks %}
- {{ task.id }} — {{ task.title }}
{% endfor %}

## In Progress
{% for task in in_progress_tasks %}
- {{ task.id }} — {{ task.title }}
{% endfor %}

## Blocked
{% for task in blocked_tasks %}
- {{ task.id }} — {{ task.title }}{% if task.blocked_reason %} — ({{ task.blocked_reason }}){% endif %}
{% endfor %}

## Mentioned in Logs
{% for task_id, dates in mentioned_tasks %}
- {{ task_id }} — Mentioned on: {{ dates|join(', ') }}
{% endfor %}

## Missing tasks referenced in logs
{% for task_id, dates in missing_tasks %}
- {{ task_id }} — Mentioned on: {{ dates|join(', ') }}
{% endfor %}

## Worklog Highlights
{% for log in logs %}
### {{ log.date }}
{% for highlight in log.highlights %}
- {{ highlight }}
{% endfor %}
{% endfor %}
```

## Daily Log Template

### Template Variables

| Variable | Type | Description |
|----------|------|-------------|
| `date` | `String` | Log date |
| `project` | `String` | Project slug/name |
| `highlights` | `Vec<String>` | Highlight items |
| `done` | `Vec<String>` | Done items |
| `doing` | `Vec<String>` | Doing items |
| `blocked` | `Vec<String>` | Blocked items |
| `notes` | `Vec<String>` | Note items |

### Example: Daily Log Template

```jinja2
# {{ date }} ({{ project }})

## Highlights
{% for highlight in highlights %}
- {{ highlight }}
{% endfor %}

## Done
{% for item in done %}
- {{ item }}
{% endfor %}

## Doing
{% for item in doing %}
- {{ item }}
{% endfor %}

## Blocked
{% for item in blocked %}
- {{ item }}
{% endfor %}

## Notes
{% for item in notes %}
- {{ item }}
{% endfor %}
```

## Configuration

Add template paths to your `tt.toml`:

```toml
[reports]
track_in_git = true
weekly_dir = "reports/weekly"

# Optional: Custom template paths
template_path = "templates/weekly_report.j2"      # Weekly report
log_template_path = "templates/daily_log.j2"      # Daily log
```

### Configuration Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `template_path` | `String` | `None` | Path to weekly report template |
| `log_template_path` | `String` | `None` | Path to daily log template |

Paths can be:
- **Relative** to workspace root (e.g., `"templates/weekly_report.j2"`)
- **Absolute** paths (e.g., `"/home/user/.config/tt/templates/weekly.j2"`)

## Template Syntax

Jinja2 templates support:

### Variables

```jinja2
{{ variable_name }}
```

### Filters

```jinja2
{{ tasks|length }}           // Count items
{{ dates|join(', ') }}       // Join array with separator
{{ text|upper }}             // Uppercase
{{ text|lower }}             // Lowercase
```

### Conditionals

```jinja2
{% if task.blocked_reason %}
Blocked: {{ task.blocked_reason }}
{% endif %}

{% if done_tasks|length > 0 %}
Tasks completed!
{% else %}
No tasks completed.
{% endif %}
```

### Loops

```jinja2
{% for task in done_tasks %}
- {{ task.id }}: {{ task.title }}
{% endfor %}

{% for task_id, dates in mentioned_tasks %}
{{ task_id }}: {{ dates|join(', ') }}
{% endfor %}
```

### Loop Variables

```jinja2
{% for task in tasks %}
{{ loop.index }}. {{ task.title }}{% if not loop.last %}, {% endif %}
{% endfor %}
```

## How Templates Are Loaded

When generating a report, `tt` follows this order:

1. **Check config** for `template_path` or `log_template_path`
2. **Try to load** from filesystem at configured path
3. **Fallback** to default `templates/<name>.j2` if no config
4. **Use embedded** template if file not found
5. **Error** only if both custom and embedded fail

This ensures reports always work, even if custom templates are missing.

## Troubleshooting

### Template Not Loading

**Problem:** Custom template is ignored, embedded template used instead.

**Solutions:**
1. Check file path in `tt.toml` is correct
2. Verify file exists: `ls templates/weekly_report.j2`
3. Check file permissions (must be readable)
4. Look for warnings in output: `tt report week --verbose`

### Template Rendering Errors

**Problem:** Error message like "Template rendering error: unknown variable"

**Solutions:**
1. Verify variable names match documentation
2. Check for typos in variable names
3. Ensure filters are supported (see [minijinja docs](https://docs.rs/minijinja/latest/minijinja/))
4. Test with default template first

### Invalid Template Syntax

**Problem:** Error about "syntax error" or "unexpected token"

**Solutions:**
1. Check Jinja2 syntax (matching `{% %}`, `{{ }}`)
2. Verify filter syntax: `{{ var|filter }}`
3. Test template with online Jinja2 validators
4. Temporarily simplify template to isolate issue

### Graceful Fallback

If your custom template has errors, `tt` will:
- Show a warning message
- Fall back to the embedded default template
- Continue generating the report

This prevents template issues from blocking your workflow.

## Examples

### Minimal Weekly Report

```jinja2
# Week {{ week.iso_week }}

## Completed
{% for task in done_tasks %}
- {{ task.title }}
{% endfor %}
```

### Detailed Weekly Report with Tags

```jinja2
# Weekly Report — {{ week.iso_week }}

## Summary
Total done: {{ done_tasks|length }}

## Tasks
{% for task in done_tasks %}
### {{ task.id }} — {{ task.title }}
- Status: {{ task.status }}
- Tags: {{ task.tags|join(', ') }}
- Created: {{ task.created_at }}
{% if task.notes %}
- Notes: {{ task.notes }}
{% endif %}
{% endfor %}
```

### Filter by Priority

```jinja2
# High Priority Tasks This Week

{% for task in done_tasks %}
{% if task.priority and task.priority != "P2" %}
- [{{ task.priority }}] {{ task.id }}: {{ task.title }}
{% endif %}
{% endfor %}
```

### Custom Section Ordering

```jinja2
# {{ week.iso_week }} Report

{% if in_progress_tasks|length > 0 %}
## In Progress
{% for task in in_progress_tasks %}
- {{ task.id }}: {{ task.title }}
{% endfor %}
{% endif %}

{% if done_tasks|length > 0 %}
## Done
{% for task in done_tasks %}
- {{ task.id }}: {{ task.title }}
{% endfor %}
{% endif %}

## Highlights
{% for log in logs %}
### {{ log.date }}
{% for highlight in log.highlights %}
- {{ highlight }}
{% endfor %}
{% endfor %}
```

## Advanced Usage

### Custom Filters

Currently, `tt` supports these built-in filters:
- `length` — Count items in array/string
- `join` — Join array with separator
- `upper` — Convert to uppercase
- `lower` — Convert to lowercase

More filters may be added in future versions.

### Template Inheritance

Jinja2 supports template inheritance, but `tt` currently uses simple templates only. This may be added in a future release.

## References

- [minijinja documentation](https://docs.rs/minijinja/latest/minijinja/)
- [Jinja2 template designer documentation](https://jinja.palletsprojects.com/en/3.1.x/templates/)
- [tt configuration guide](./configuration.md)
