# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial MVP release (v0.1.0)
- `tt init` - Initialize new workspace
- `tt add` - Create tasks with options (--due, --priority, --tag, --notes, --estimate)
- `tt ls` - List tasks with filtering (--status, --all, --project)
- `tt show` - Display task details
- `tt start` - Start working on tasks
- `tt done` - Complete tasks
- `tt log` - Append to daily logs with auto task ID detection
- `tt report week` - Generate weekly reports
- Git suggestions for all commands (branch names, commit messages)
- Multi-project support
- Status transition validation
- File locking for ID generation (prevents collisions)
- ISO-8601 week calculations (Monday start)
- Auto-linking: task ID detection in logs (`tt-XXXXXX` pattern)

## [0.3.0] - 2026-03-28

### Added
- **Distribution** — GitHub Actions release workflow for Windows, macOS, Linux
- **Template system** — Customizable weekly report and daily log templates
- **Full-text search** — `tt search` command with tantivy-based indexing
- **Enhanced reports** — Smart task mention merging in weekly reports
- **Better error messages** — Actionable suggestions for all errors
- **Performance benchmarks** — criterion benchmarks for key operations
- **Troubleshooting guide** — Comprehensive docs for common issues
- **Test fixtures** — Large workspace and multi-project test fixtures

### Changed
- Error messages now include suggestions for resolution
- Weekly reports support merged task data with mention dates

### Fixed
- N/A

### Security
- N/A

## [0.2.0] - 2026-03-28

### Added
- Template system for reports and logs
- Search functionality with filters

## [0.1.0] - 2026-03-28
- N/A

### Removed
- N/A

### Fixed
- N/A

### Security
- N/A

---

## [0.1.0] - 2026-03-28

### Added
- Initial release
- Complete MVP implementation
- All core features for personal task tracking
- Plain-text storage (TOML tasks + Markdown logs)
- Weekly report generation with all sections:
  - Done (by done_at date)
  - In Progress (status=doing)
  - Blocked (status=blocked)
  - Mentioned in Logs (auto-linking)
  - Missing tasks referenced in logs
  - Worklog Highlights extraction
- 57+ unit tests
- Integration tests for all CLI commands
- Comprehensive documentation (README, SETUP, planning docs)

[Unreleased]: https://github.com/yourusername/tt/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/yourusername/tt/releases/tag/v0.1.0
