# Phase 3 Research: Distribution & Polish (v0.3)

**Status:** Draft  
**Created:** 2026-03-28  
**Phase Goal:** Make `tt` installable and production-ready for public adoption

---

## Overview

Phase 03 focuses on transforming the MVP into a polished, distributable product that users can easily install and rely on for daily use.

### Key Objectives

1. **Easy Installation** — Binary releases for Windows, macOS, Linux
2. **Power Features** — Search, indexing, enhanced reports
3. **Quality Hardening** — Extensive testing, performance benchmarks
4. **Documentation** — Complete user guide, troubleshooting

---

## R3.x Requirements Breakdown

### R3.1: Distribution

**Goal:** Users can install `tt` easily without building from source.

#### Installation Methods

| Method | Priority | Effort | Notes |
|--------|----------|--------|-------|
| **GitHub Releases** | P0 | Medium | Binary assets for each platform |
| **cargo install** | P0 | Low | Already works via `cargo install --path .` |
| **cargo-binstall** | P1 | Low | Faster installs, pre-built binaries |
| **Homebrew (macOS)** | P2 | Medium | `brew install tt` |
| **Scoop (Windows)** | P2 | Low | `scoop install tt` |
| **crates.io** | P1 | Medium | Publish to crates.io registry |

#### Binary Release Strategy

**Recommended:** GitHub Actions CI/CD with `cross` crate for cross-compilation

```yaml
# .github/workflows/release.yml
name: Release

on:
  push:
    tags: ['v*']

jobs:
  build:
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
          - os: macos-latest
            target: x86_64-apple-darwin
          - os: windows-latest
            target: x86_64-pc-windows-msvc
    
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      - uses: swatinem/rust-cache@v2
      - run: cargo build --release --target ${{ matrix.target }}
      - uses: softprops/action-gh-release@v1
        with:
          files: target/${{ matrix.target }}/release/tt*
```

#### crates.io Publishing

**Checklist:**
- [ ] Verify license fields in Cargo.toml (MIT OR Apache-2.0 ✅)
- [ ] Add repository URL ✅
- [ ] Add keywords and categories ✅
- [ ] Ensure README.md is included
- [ ] Run `cargo package --list` to verify files
- [ ] `cargo publish`

**Crate name:** `tt` (check availability — may need `tt-cli` or `task-tracker`)

---

### R3.2: Templates + Customization

**Goal:** Users can customize report and log templates.

#### Report Template Customization

**Approach:** External Jinja2 template files

```
my-worklog/
├── tt.toml
├── templates/
│   ├── weekly_report.j2
│   └── daily_log.j2
└── projects/
    └── ...
```

**Default template embedded in binary** (for users who don't customize)

**Template variables for weekly report:**
```jinja2
# Weekly Report — {{ week.iso }} ({{ project.name }})
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
- {{ task.id }} — {{ task.title }}
{% endfor %}

## Mentioned in Logs
{% for task_id, dates in mentioned_tasks %}
- {{ task_id }} — Mentioned on: {{ dates|join(', ') }}
{% endfor %}

## Missing tasks referenced in logs
{% for task_id in missing_tasks %}
- {{ task_id }} (file not found)
{% endfor %}

## Worklog Highlights
{% for log in logs %}
### {{ log.date }}
{% for highlight in log.highlights %}
- {{ highlight }}
{% endfor %}
{% endfor %}
```

**Config option:**
```toml
[reports]
template_path = "templates/weekly_report.j2"  # Optional, uses embedded if missing
```

#### Daily Log Template Customization

Similar approach for log templates:

```jinja2
# {{ date }} ({{ project.name }})

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
{% for note in notes %}
- {{ note }}
{% endfor %}
```

---

### R3.3: Search + Indexing

**Goal:** Fast search across tasks and logs.

#### Search Scope

| Target | Description | Priority |
|--------|-------------|----------|
| **Task titles** | Search in task titles | P0 |
| **Task notes** | Search in task notes | P0 |
| **Log content** | Search in daily logs | P0 |
| **Tags** | Filter by tags | P1 |
| **Status** | Filter by status | P1 |
| **Date ranges** | Search within date range | P2 |

#### Indexing Strategy

**Option A: Full-text search on every query** (simplest)
- Scan all TOML and Markdown files
- Use regex for matching
- Pros: Simple, no index maintenance
- Cons: Slow for large workspaces (1000+ files)

**Option B: Build index cache** (recommended)
- Build index on first run
- Incremental updates on file changes
- Store in `.tt/index.json`
- Pros: Fast searches, scales well
- Cons: Index invalidation logic

**Recommended:** Option B with `tantivy` crate (Rust Elasticsearch)

#### Search Command Design

```bash
# Basic search
tt search "config loader"

# Filter by project
tt search "config" --project work

# Filter by status
tt search "refactor" --status todo,doing

# Filter by tag
tt search "cli" --tag rust

# Date range
tt search "meeting" --from 2026-03-01 --to 2026-03-31

# JSON output (for scripting)
tt search "task" --json
```

#### Implementation with tantivy

```rust
use tantivy::{doc, Index, IndexWriter, Schema, TEXT};

pub struct SearchIndex {
    index: Index,
    schema: SearchSchema,
}

struct SearchSchema {
    id: Field,
    title: Field,
    content: Field,
    project: Field,
    status: Field,
    tags: Field,
    created_at: Field,
}

impl SearchIndex {
    pub fn create(path: &Path) -> Result<Self> {
        let mut schema_builder = Schema::builder();
        let id = schema_builder.add_text_field("id", STORED);
        let title = schema_builder.add_text_field("title", TEXT | STORED);
        let content = schema_builder.add_text_field("content", TEXT);
        let project = schema_builder.add_text_field("project", STORED);
        let status = schema_builder.add_text_field("status", STORED);
        let tags = schema_builder.add_text_field("tags", STORED);
        let created_at = schema_builder.add_date_field("created_at", STORED);

        let schema = schema_builder.build();
        let index = Index::create_in_dir(path, schema.clone())?;

        Ok(Self {
            index,
            schema: SearchSchema { id, title, content, project, status, tags, created_at },
        })
    }

    pub fn add_task(&mut self, task: &Task) -> Result<()> {
        let mut index_writer = self.index.writer(50_000_000)?;
        index_writer.add_document(doc!(
            self.schema.id => task.id.to_string(),
            self.schema.title => task.title.clone(),
            self.schema.content => task.notes.clone().unwrap_or_default(),
            self.schema.project => task.project.clone(),
            self.schema.status => task.status.to_string(),
            self.schema.tags => task.tags.join(","),
        ));
        index_writer.commit()?;
        Ok(())
    }

    pub fn search(&self, query: &str, limit: usize) -> Vec<TaskSearchResult> {
        let reader = self.index.reader().unwrap();
        let searcher = reader.searcher();
        let query_parser = QueryParser::for_index(
            &self.index,
            vec![self.schema.title, self.schema.content],
        );
        let query = query_parser.parse_query(query).unwrap();
        let top_docs = searcher.search(&query, &TopDocs::with_limit(limit)).unwrap();

        top_docs
            .iter()
            .map(|(score, doc_address)| {
                let doc = searcher.doc(*doc_address).unwrap();
                TaskSearchResult {
                    id: doc.get_first(self.schema.id).unwrap().as_text().unwrap().to_string(),
                    title: doc.get_first(self.schema.title).unwrap().as_text().unwrap().to_string(),
                    score: *score,
                }
            })
            .collect()
    }
}
```

---

### R3.4: Enhanced Report Intelligence

**Goal:** Smarter report generation with better task linking.

#### Smart Task Mention Merging

**Current behavior (v0.1):**
- "Mentioned in Logs" is separate section
- Even if task is in "Done" section

**Enhanced behavior (v0.3):**
- Merge "Mentioned" into Done/Doing/Blocked if task status matches
- Add log dates to task entry

**Example:**
```markdown
## Done
- tt-000001 — Refactor config loader
  - Mentioned on: 2026-03-28, 2026-03-29
  - Log: "Initial implementation"

## In Progress
- tt-000002 — Add dashboard
  - Mentioned on: 2026-03-30
  - Log: "Started working on UI"
```

#### Better Highlights Extraction

**Current:** Simple bullet extraction from "Highlights" section

**Enhanced:**
- Extract from all sections if "Highlights" missing
- Use NLP-like heuristics for important bullets
- Limit highlights per day (configurable)

```toml
[reports]
max_highlights_per_day = 5
highlight_sections = ["Highlights", "Done", "Doing"]
```

#### Configurable Report Sections

Allow users to enable/disable sections:

```toml
[reports]
sections = ["done", "in_progress", "blocked", "mentioned", "highlights"]
# Disable "missing tasks" section if not needed
# sections = ["done", "in_progress", "mentioned", "highlights"]
```

---

### R3.5: Quality Hardening

**Goal:** Production-ready reliability.

#### Error Message Improvements

**Before:**
```
Error: Task 'tt-000001' not found
```

**After:**
```
Error: Task 'tt-000001' not found

Suggestions:
  - Check the task ID (format: tt-XXXXXX)
  - Run 'tt ls' to see all tasks
  - Task may be in a different project (use --project flag)
```

#### Performance Benchmarks

**Target metrics:**
| Operation | Target | Measurement |
|-----------|--------|-------------|
| `tt ls` (100 tasks) | < 100ms | Average |
| `tt ls` (1000 tasks) | < 500ms | Average |
| `tt report week` (7 logs) | < 1s | Average |
| `tt search "query"` | < 200ms | With index |
| `tt init` | < 500ms | Fresh workspace |

**Benchmark setup with `criterion`:**
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_ls_command(c: &mut Criterion) {
    let workspace = create_test_workspace_with_tasks(100);
    c.bench_function("ls_100_tasks", |b| {
        b.iter(|| {
            let tasks = workspace.list_tasks(TaskFilter::All).unwrap();
            black_box(tasks);
        })
    });
}

criterion_group!(benches, bench_ls_command);
criterion_main!(benches);
```

#### Fixture Tests

Create realistic test fixtures:
- Large workspace (1000+ tasks)
- Multi-project setup
- Edge cases (year boundaries, malformed files)

---

## Tech Stack Recommendations

### New Dependencies for v0.3

```toml
[dependencies]
# Search (R3.3)
tantivy = "0.21"

# Better CLI tables (R2.3, also useful for v0.3)
comfy-table = "7.1"

# Dialog/prompts (optional, for interactive mode)
dialoguer = "0.11"
indicatif = "0.17"  # Progress bars

# Template loading from filesystem
include_dir = "0.7"  # Embed default templates

[dev-dependencies]
# Benchmarks
criterion = "0.5"

# Property-based testing
proptest = "1.4"

[[bench]]
name = "performance"
harness = false
```

---

## Implementation Complexity

| Feature | Complexity | Risk | Effort |
|---------|------------|------|--------|
| **R3.1: Distribution** | Medium | Low | 2-3 days |
| **R3.2: Templates** | Low-Medium | Low | 1-2 days |
| **R3.3: Search** | Medium-High | Medium | 3-4 days |
| **R3.4: Enhanced Reports** | Low | Low | 1-2 days |
| **R3.5: Quality Hardening** | Medium | Low | 2-3 days |

**Total estimated effort:** 9-14 days (part-time: 2-3 weeks)

---

## Open Questions

### Q1: Should search use tantivy or simple regex?

**Trade-offs:**
- `tantivy`: Fast, scalable, but adds ~500KB binary size
- Regex: Simple, but slow for 1000+ files

**Recommendation:** Start with regex, add tantivy if users report slowness

---

### Q2: Publish to crates.io or GitHub Releases only?

**crates.io pros:**
- Standard Rust installation (`cargo install tt`)
- Discoverability

**crates.io cons:**
- Name `tt` likely taken
- Source-only (users must compile)

**Recommendation:** Both — crates.io for Rust users, GitHub Releases for binaries

---

### Q3: Interactive mode (`tt` without args)?

**Idea:** Run `tt` → interactive menu instead of showing help text

**Recommendation:** Nice-to-have, defer to post-v0.3

---

## Decisions Summary

| Decision | Choice | Rationale |
|----------|--------|-----------|
| D3.1: Binary Distribution | GitHub Releases + CI | Cross-platform, easy for users |
| D3.2: crates.io | Publish (check name) | Rust community standard |
| D3.3: Search Engine | tantivy (optional) | Fast, scalable, worth the dependency |
| D3.4: Template System | External .j2 files | Flexibility without complexity |
| D3.5: Benchmark Tool | criterion | Industry standard |
| D3.6: Interactive Mode | Defer | Not critical for v0.3 |

---

## References

- [GitHub Actions for Rust](https://github.com/actions-rs/meta)
- [cross: Zero setup cross-compilation](https://github.com/cross-rs/cross)
- [tantivy documentation](https://github.com/quickwit-oss/tantivy)
- [criterion.rs benchmarking](https://bheisler.github.io/criterion.rs/book/)
- [clap completion generation](https://docs.rs/clap/latest/clap/struct.Command.html#method.generate_completions)

---

## Next Steps

1. **Review this research** — Confirm tech choices align with project goals
2. **Run `/gsd:plan-phase 3`** — Generate atomic implementation tasks
3. **Approve plan** — Ready for execution
