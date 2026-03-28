# Troubleshooting Guide for tt CLI

This guide covers common issues, error messages, and solutions for using `tt`.

---

## Quick Reference

| Error | Quick Fix |
|-------|-----------|
| "Workspace not initialized" | Run `tt init` |
| "Task not found" | Check ID format, run `tt ls` |
| "Project not found" | Run `tt ls` to see projects |
| Search not working | Delete `.tt/index/`, rebuild |
| Template errors | Check Jinja2 syntax |

---

## Common Issues

### 1. "Workspace not initialized. Run 'tt init' first."

**Cause:** You're trying to use `tt` in a directory that hasn't been initialized.

**Solution:**
```bash
# Initialize workspace in current directory
tt init

# Or navigate to an existing workspace
cd /path/to/workspace
```

**Prevention:** Always run `tt init` in a new directory before using other commands.

---

### 2. "Task 'tt-000001' not found"

**Possible causes:**
- Task ID is incorrect (typo, wrong format)
- Task is in a different project
- Task was deleted

**Solution:**
```bash
# List all tasks to verify ID
tt ls

# Check specific project
tt ls --project work

# Use correct ID format (tt-XXXXXX)
tt show tt-000001
```

---

### 3. "Invalid status transition: todo → done"

**Cause:** You can't transition directly from `todo` to `done`.

**Valid transitions:**
```
todo → doing → done
todo → blocked
todo → canceled
doing → done
doing → blocked
doing → canceled
blocked → doing
blocked → canceled
```

**Solution:**
```bash
# Start the task first
tt start tt-000001

# Then mark as done
tt done tt-000001
```

---

### 4. "Search index error" / Search not returning results

**Cause:** Search index is corrupted or empty.

**Solution:**
```bash
# Delete and rebuild index
rm -rf .tt/index

# Index will be recreated on next search
tt search "query"
```

**Note:** If you have many tasks, consider rebuilding the index after major changes.

---

### 5. "Template error"

**Cause:** Custom template has syntax errors or missing variables.

**Solution:**
```bash
# Check template syntax (Jinja2 format)
# Ensure all variables are defined

# Temporarily use embedded template
# Remove or rename custom template file
mv templates/weekly_report.j2 templates/weekly_report.j2.bak
```

**Common template issues:**
- Missing closing braces: `{{ variable }` → `{{ variable }}`
- Invalid filters: `{{ var|unknown }}` → `{{ var|default('') }}`
- Undefined variables: Check available variables in template docs

---

### 6. "Editor not found"

**Cause:** `$EDITOR` environment variable not set.

**Solution:**

**Windows:**
```cmd
setx EDITOR "notepad.exe"
```

**macOS/Linux:**
```bash
export EDITOR="vim"
# Add to ~/.bashrc or ~/.zshrc for persistence
```

**Verify:**
```bash
echo $EDITOR
```

---

### 7. "Invalid week format"

**Cause:** Week argument not in ISO format.

**Correct format:** `YYYY-Www`

**Examples:**
```bash
# Correct
tt report week --week 2026-W13
tt report week --week 2026-W01

# Incorrect
tt report week --week 2026-13
tt report week --week W13-2026
```

---

### 8. "Date parsing error"

**Cause:** Date not in `YYYY-MM-DD` format.

**Correct format:** `YYYY-MM-DD`

**Examples:**
```bash
# Correct
tt add "Task" --due 2026-04-03
tt log "Work" --date 2026-03-28

# Incorrect
tt add "Task" --due 04-03-2026
tt add "Task" --due 2026/04/03
```

---

### 9. "File lock failed" / "ID generation failed"

**Cause:** Another `tt` process is running, or lock file wasn't cleaned up.

**Solution:**
```bash
# Check for running tt processes
# Windows: Task Manager
# macOS/Linux: ps aux | grep tt

# If no other process is running, remove lock file
rm .tt/lock

# Retry command
tt add "Task"
```

---

### 10. Git suggestions not showing

**Cause:** Workspace doesn't have a `.git` directory.

**Solution:**
```bash
# Initialize git repo
git init

# Git suggestions will appear after commands
tt add "Task"
```

---

## Performance Troubleshooting

### Slow `tt ls` command

**Symptoms:** Listing tasks takes > 1 second.

**Causes:**
- Large workspace (1000+ tasks)
- Slow disk I/O

**Solutions:**
```bash
# Filter by status
tt ls --status todo

# Filter by project
tt ls --project work

# Use search instead (faster for large workspaces)
tt search "keyword"
```

---

### Slow search

**Symptoms:** Search takes > 500ms.

**Solutions:**
```bash
# Rebuild index
rm -rf .tt/index

# Use more specific filters
tt search "config" --project work
tt search "bug" --status todo

# Limit results
tt search "task" --limit 10
```

---

### Slow report generation

**Symptoms:** `tt report week` takes > 2 seconds.

**Solutions:**
```bash
# Reduce log size (archive old logs)
# Use specific week
tt report week --week 2026-W13

# Check for very large logs
# Split large daily logs
```

---

## Error Message Reference

### TtError Variants

| Error | Description | Suggestions |
|-------|-------------|-------------|
| `WorkspaceNotFound` | No workspace in current directory | Run `tt init` |
| `WorkspaceNotFoundAtPath` | No workspace at specified path | Check path, run `tt init` |
| `ProjectNotFound` | Project doesn't exist | Run `tt ls`, check project name |
| `TaskNotFound` | Task ID not found | Check ID, run `tt ls` |
| `InvalidStatusTransition` | Invalid status change | Use valid transitions |
| `TomlParseError` | TOML syntax error | Check TOML syntax |
| `IoError` | File system error | Check permissions, paths |
| `TemplateError` | Template rendering error | Check Jinja2 syntax |
| `IdGenerationError` | Task ID generation failed | Check permissions, locks |
| `LockError` | File locking failed | Close other tt processes |
| `DateParseError` | Invalid date format | Use YYYY-MM-DD |
| `InvalidWeekFormat` | Invalid week format | Use YYYY-Www |
| `EditorNotFound` | $EDITOR not set | Set environment variable |
| `SearchIndexError` | Search index error | Rebuild index |

---

## Template Debugging

### Template not rendering

**Check:**
1. Template file exists at configured path
2. Template syntax is valid Jinja2
3. All variables are defined

**Debug mode:**
```bash
# Enable debug logging
export RUST_LOG=debug
tt report week
```

**Common issues:**
- `{{ variable }}` vs `{% if variable %}` — different syntax
- Filters must exist: `{{ var|filter }}`
- Whitespace matters in Jinja2

---

## Getting Help

### Enable verbose output

```bash
# Debug logging
export RUST_LOG=debug
tt <command>

# Or use --help for command info
tt <command> --help
```

### Check version

```bash
tt --version
```

### Report a bug

1. Run with `RUST_LOG=debug`
2. Copy error output
3. Include `tt --version`
4. Open issue on GitHub

---

## Configuration Reference

### tt.toml

```toml
version = 1

[workspace]
default_project = "work"
week_starts_on = "monday"
task_id_prefix = "tt-"
task_id_width = 6

[storage]
projects_dir = "projects"

[reports]
template_path = "templates/weekly_report.j2"  # Optional
track_in_git = true
weekly_dir = "reports/weekly"

[logs]
template_path = "templates/daily_log.j2"  # Optional

[git]
suggest_branch = true
suggest_commit = true
```

---

## Additional Resources

- **Documentation:** `README.md`, `docs/templates.md`
- **Examples:** `tests/fixtures/`
- **Source code:** `src/`
- **Issues:** GitHub Issues page

---

## Maintenance

### Backup workspace

```bash
# Backup entire workspace
tar czf worklog-backup.tar.gz my-worklog/

# Or use git
git add -A
git commit -m "Backup"
```

### Clean up old data

```bash
# Remove old index (will be rebuilt)
rm -rf .tt/index

# Archive old reports
mkdir archive
mv projects/*/reports/weekly/2025-* archive/
```

### Update tt

```bash
# From crates.io
cargo install tt --force

# From source
cd /path/to/tt
git pull
cargo install --path .
```

---

**Last updated:** 2026-03-28  
**Version:** v0.3.0
