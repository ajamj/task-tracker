# Release Checklist for tt CLI

This document outlines the steps to release a new version of `tt`.

---

## Pre-Release Checklist

### 1. Update Version Numbers

- [ ] Update version in `Cargo.toml`:
  ```toml
  [package]
  version = "0.3.0"  # Update this
  ```

- [ ] Update version in `src/main.rs` if hardcoded anywhere

### 2. Update CHANGELOG.md

- [ ] Add new section for this version with date
- [ ] List all changes under appropriate categories (Added, Changed, Fixed, etc.)
- [ ] Update comparison links at bottom of CHANGELOG.md

Example:
```markdown
## [0.3.0] - 2026-03-28

### Added
- Distribution via GitHub Releases and crates.io
- Template system for customizing reports and logs
- Full-text search with `tt search` command
- Enhanced report intelligence with smart task merging
- Quality hardening with benchmarks and improved error messages

[0.3.0]: https://github.com/yourusername/tt/compare/v0.2.0...v0.3.0
```

### 3. Update README.md

- [ ] Update installation instructions if needed
- [ ] Update feature list if new features added
- [ ] Verify all examples still work

### 4. Run Final Checks

```bash
# Build release
cargo build --release

# Run all tests
cargo test

# Run clippy
cargo clippy -- -D warnings

# Check formatting
cargo fmt --check

# Verify package
cargo package --list
cargo package --verify
```

### 5. Git Tag

```bash
# Create annotated tag
git tag -a v0.3.0 -m "Release v0.3.0: Distribution & Polish"

# Push tag to GitHub
git push origin v0.3.0
```

---

## Release Process

### Automatic (GitHub Actions)

Once you push a version tag (`v*`), GitHub Actions will:

1. **Build binaries** for:
   - Linux (x86_64-unknown-linux-gnu)
   - macOS (x86_64-apple-darwin)
   - Windows (x86_64-pc-windows-msvc)

2. **Create release artifacts**:
   - `tt-linux-amd64.tar.gz`
   - `tt-macos-amd64.tar.gz`
   - `tt-windows-amd64.tar.gz`

3. **Upload to GitHub Releases** automatically

4. **Publish to crates.io** (if `CARGO_REGISTRY_TOKEN` secret is set)

### Manual Verification

After GitHub Actions completes:

- [ ] Verify all binaries are attached to release
- [ ] Download and test each binary
- [ ] Check release notes are correct
- [ ] Verify crates.io publication (if applicable)

---

## Post-Release

### 1. Update Installation Docs

- [ ] Update README.md with new version number if mentioned
- [ ] Update docs/installation.md if exists
- [ ] Announce release on relevant channels

### 2. Test Installation

```bash
# Test cargo install (if published to crates.io)
cargo install tt
tt --version

# Test GitHub Releases binary
# Download from releases page and verify
```

### 3. Monitor for Issues

- [ ] Watch GitHub Issues for bug reports
- [ ] Monitor crates.io download stats (if published)
- [ ] Be ready to patch critical bugs quickly

---

## Secrets Required (GitHub)

Set these in your repository settings under **Settings → Secrets and variables → Actions**:

| Secret | Description | Required |
|--------|-------------|----------|
| `CARGO_REGISTRY_TOKEN` | crates.io API token for publishing | Optional (for crates.io) |

### Getting CARGO_REGISTRY_TOKEN

1. Go to https://crates.io/settings/tokens
2. Create a new token with a descriptive name
3. Copy the token
4. Add to GitHub repository secrets as `CARGO_REGISTRY_TOKEN`

---

## Rollback Procedure

If a release has critical issues:

### 1. Yank from crates.io

```bash
cargo yank --vers 0.3.0
```

### 2. Delete GitHub Release

- Go to Releases page
- Delete the problematic release
- Delete the tag

### 3. Fix and Re-release

```bash
# Fix issues
# Update version to 0.3.1
git tag -a v0.3.1 -m "Release v0.3.1: Hotfix"
git push origin v0.3.1
```

---

## Version Numbering

This project follows [Semantic Versioning](https://semver.org/):

- **MAJOR** (1.0.0 → 2.0.0): Breaking changes
- **MINOR** (0.2.0 → 0.3.0): New features (backward compatible)
- **PATCH** (0.2.1 → 0.2.2): Bug fixes (backward compatible)

Current version: **0.1.0** (MVP)

Next release: **0.3.0** (Distribution & Polish - Phase 03)

---

## Release Notes Template

```markdown
## [VERSION] - DATE

### Added
- New features

### Changed
- Changes to existing functionality

### Fixed
- Bug fixes

### Removed
- Removed features

### Security
- Security improvements

[VERSION]: https://github.com/yourusername/tt/compare/vPREV...vVERSION
```

---

## Contact

For questions about the release process, open an issue on GitHub.
