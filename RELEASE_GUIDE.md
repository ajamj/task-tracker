# 🚀 GitHub Release Guide

## Pre-Release Checklist

### 1. Verify Build Status
- [ ] All tests passing (`cargo test`)
- [ ] Clippy clean (`cargo clippy -- -D warnings`)
- [ ] Formatting clean (`cargo fmt --check`)
- [ ] Build successful (`cargo build --release`)

### 2. Update Documentation
- [ ] README.md updated with latest features
- [ ] CHANGELOG.md updated with release notes
- [ ] Version bumped in Cargo.toml
- [ ] Git tag created

### 3. Test Dashboard
- [ ] Dashboard starts successfully
- [ ] All views working (List, Kanban, Calendar, Charts)
- [ ] Dark mode working
- [ ] Export/Import working

---

## Release Process

### Step 1: Create Git Tag

```bash
# Create annotated tag
git tag -a v0.3.0 -m "Release v0.3.0 - Dashboard v3.0 with all features"

# Push tag to GitHub
git push origin v0.3.0
```

### Step 2: Create GitHub Release

1. Go to https://github.com/yourusername/tt/releases
2. Click "Draft a new release"
3. Select tag: `v0.3.0`
4. Release title: `v0.3.0 - Dashboard v3.0`
5. Add release notes (see template below)
6. Click "Publish release"

### Step 3: GitHub Actions Will Auto-Build

Once release is created:
- ✅ CI/CD pipeline will trigger automatically
- ✅ Build binaries for Windows, macOS, Linux
- ✅ Run all tests
- ✅ Upload release assets
- ✅ Publish to crates.io (if token configured)

---

## Release Notes Template

```markdown
## What's New

### 🎨 Dashboard v3.0
- ✅ 4 view modes (List, Kanban, Calendar, Charts)
- ✅ Dark mode toggle
- ✅ Drag & drop kanban board
- ✅ Progress charts (4 types)
- ✅ Export/Import functionality
- ✅ Task detail modal
- ✅ Real-time filters & search

### 🔧 CLI Improvements
- ✅ Enhanced error messages with suggestions
- ✅ Better formatting with colors
- ✅ Git suggestions for all commands

### 📊 Infrastructure
- ✅ GitHub Actions CI/CD
- ✅ Automated release builds
- ✅ Integration tests (78% pass rate)
- ✅ Performance benchmarks

### 🐛 Bug Fixes
- Fixed stack overflow in `tt add` (toml_edit → toml)
- Fixed duplicate CLI short options
- Fixed workspace loading recursion
- Fixed dashboard navigation

## Installation

### From Source
```bash
git clone https://github.com/yourusername/tt
cd tt
cargo install --path .
```

### From Release Assets
Download the binary for your platform:
- **Windows:** `tt-v0.3.0-windows.zip`
- **Linux:** `tt-v0.3.0-ubuntu-latest.tar.gz`
- **macOS:** `tt-v0.3.0-macos-latest.tar.gz`

Extract and add to PATH.

## Quick Start

```bash
# Initialize workspace
tt init

# Add a task
tt add "My first task" --priority P1

# Start dashboard
tt dashboard
```

## Dashboard

Open http://localhost:3000 after running `tt dashboard`

Features:
- 📋 List View
- 📊 Kanban Board (drag & drop)
- 📅 Calendar View
- 📈 Charts (4 types)
- 🌙 Dark Mode
- 📥 Export/Import

## Known Issues

- Search command WIP (use `tt ls` with filters)
- 2 integration tests failing (minor path issues)

## Contributors

Thanks to everyone who contributed to this release!

## Full Changelog

https://github.com/yourusername/tt/compare/v0.2.0...v0.3.0
```

---

## Post-Release Checklist

### 1. Verify Release
- [ ] Binaries uploaded to release
- [ ] SHA256 checksums available
- [ ] Release notes published
- [ ] GitHub Actions completed successfully

### 2. Test Downloaded Binaries
- [ ] Windows binary works
- [ ] Linux binary works
- [ ] macOS binary works
- [ ] All commands functional

### 3. Update Documentation
- [ ] README.md installation instructions
- [ ] Website/docs updated
- [ ] Social media announcement

### 4. Monitor Issues
- [ ] Watch for bug reports
- [ ] Respond to user feedback
- [ ] Update troubleshooting docs

---

## crates.io Publishing

### Manual Publish

```bash
# Login to crates.io
cargo login <your-api-token>

# Publish
cargo publish
```

### Auto-Publish via GitHub Actions

Configure in GitHub Secrets:
- `CARGO_REGISTRY_TOKEN`: Your crates.io API token

GitHub Actions will auto-publish on release.

---

## Troubleshooting

### Build Fails on Release
1. Check GitHub Actions logs
2. Verify all tests passing locally
3. Check Cargo.toml version format

### Binary Not Working
1. Check OS compatibility
2. Verify dependencies installed
3. Try building from source

### crates.io Publish Fails
1. Check if version already exists
2. Verify API token is valid
3. Check Cargo.toml metadata

---

## Version Numbering

This project follows [Semantic Versioning](https://semver.org/):

- **MAJOR** (1.0.0 → 2.0.0): Breaking changes
- **MINOR** (0.2.0 → 0.3.0): New features (backward compatible)
- **PATCH** (0.3.0 → 0.3.1): Bug fixes

Current version: **0.3.0**

---

**Last Release:** v0.3.0 - Dashboard v3.0  
**Release Date:** 2026-03-28  
**Next Release:** v0.4.0 (TBD)
