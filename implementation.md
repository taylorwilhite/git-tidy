# git-tidy Implementation Plan

## Overview

This document outlines the plan to make git-tidy production-ready while maintaining simplicity and ease of use.

## Design Principles

Based on user requirements:
- **Primary Goal**: Ease of use - simple defaults, minimal configuration, works out of the box
- **Configuration**: Basic config for protected branches only (project and global levels)
- **Distribution**: Binary releases via GitHub in addition to `cargo install`

## Phase 1: Core Functionality (Foundation)

### 1.1 Add Essential Dependencies

Add to `Cargo.toml`:

```toml
[dependencies]
clap = { version = "4.5", features = ["derive"] }
git2 = "0.19"
chrono = "0.4"
anyhow = "1.0"
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
colored = "2.1"
```

**Rationale:**
- `clap` with derive feature: Modern, type-safe CLI parsing
- `git2`: Robust git operations with Rust bindings to libgit2
- `chrono`: Reliable date/time parsing and arithmetic
- `anyhow`: Simple error handling with context
- `serde` + `toml`: Configuration file parsing
- `colored`: Terminal output formatting for better UX

### 1.2 Implement Core Git Operations

**File Structure:**
```
src/
├── main.rs           # CLI entry point
├── git_operations.rs # Git wrapper functions
├── filters.rs        # Branch filtering logic
└── config.rs         # Configuration handling
```

**`git_operations.rs` Functions:**

```rust
pub struct BranchInfo {
    pub name: String,
    pub is_merged: bool,
    pub last_commit_date: chrono::DateTime<chrono::Utc>,
    pub is_remote: bool,
}

pub fn list_branches(repo: &git2::Repository) -> Result<Vec<BranchInfo>, anyhow::Error>
pub fn delete_branch(repo: &mut git2::Repository, branch_name: &str) -> Result<(), anyhow::Error>
pub fn get_current_branch(repo: &git2::Repository) -> Result<Option<String>, anyhow::Error>
```

**Implementation Details:**
- Use `git2::Repository::open()` to open repository
- Iterate through branches with `repo.branches(Some(BranchType::Local))`
- Check merge status using `repo.merge_analysis()`
- Get last commit date via `commit.time()`
- Handle detached HEAD gracefully

### 1.3 CLI Interface with Clap

**Command Line Arguments:**

```rust
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Actually delete branches (default: dry-run)
    #[arg(long)]
    clean: bool,
    
    /// Only show merged branches
    #[arg(long)]
    merged: bool,
    
    /// Filter branches older than duration (e.g., 30d, 2w, 1m)
    #[arg(long, value_parser = parse_duration)]
    older_than: Option<chrono::Duration>,
    
    /// Preview changes without deleting (default: true)
    #[arg(long, default_value = "true")]
    dry_run: bool,
    
    /// Skip confirmation prompts
    #[arg(long)]
    force: bool,
    
    /// Regex pattern to protect matching branches
    #[arg(long, value_parser = parse_regex)]
    keep_pattern: Option<Regex>,
}
```

**Default Behavior:**
- Dry-run mode enabled by default
- Show merged branches to `main`
- 30-day age filter
- Protected branches: `master`, `develop`, `main`

## Phase 2: Configuration System

### 2.1 Configuration File Structure

**Project-level config:** `.git-tidy.toml`
**Global config:** `~/.config/git-tidy/config.toml`

**Config Schema:**

```toml
[protected_branches]
# Default protected branches
defaults = ["master", "develop", "main"]

# Additional protected branches
additional = ["release/*", "hotfix/*"]

# Keep patterns (regex)
patterns = ["^feature/.*-wip$"]
```

### 2.2 Configuration Loading Logic

**Priority Order:**
1. CLI flags (highest priority)
2. Project-level config (`.git-tidy.toml`)
3. Global config (`~/.config/git-tidy/config.toml`)
4. Defaults (lowest priority)

**`config.rs` Functions:**

```rust
pub struct Config {
    pub protected_branches: Vec<String>,
    pub protected_patterns: Vec<Regex>,
}

pub fn load_config() -> Result<Config, anyhow::Error> {
    // Load and merge configs in priority order
}
```

### 2.3 Protected Branch Matching

**Matching Logic:**
- Exact match for branch names (e.g., `master`)
- Glob pattern matching (e.g., `release/*`)
- Regex pattern matching from `--keep-pattern` flag
- Always protect current branch (HEAD)

**Implementation:**
```rust
pub fn is_protected(branch_name: &str, config: &Config, current_branch: Option<&str>) -> bool {
    // Check exact matches
    // Check glob patterns (convert to regex)
    // Check regex patterns
    // Check if it's the current branch
}
```

## Phase 3: Safety & UX

### 3.1 Safety Features

**Default Safe Behavior:**
- Dry-run mode enabled by default
- Explicit `--clean` flag required for actual deletion
- Confirmation prompt before deletion (unless `--force`)
- Never delete the current branch (HEAD)
- Never delete protected branches
- Check if branch is merged before deletion

**Error Prevention:**
```rust
pub fn safe_delete_branch(
    repo: &mut git2::Repository,
    branch_name: &str,
    config: &Config,
    force: bool,
) -> Result<(), anyhow::Error> {
    // Check if protected
    // Check if current branch
    // Confirm (unless force)
    // Delete
}
```

### 3.2 User Experience

**Output Format:**

```
Branches to delete (5):
  ✗ feature/auth - Merged 15 days ago
  ✗ feature/api - Merged 30 days ago
  ✗ feature/ui - Merged 45 days ago
  ✗ bugfix/login - Merged 60 days ago
  ✗ refactor/cache - Merged 90 days ago

Protected branches (3):
  ✓ master - Protected
  ✓ develop - Protected
  ✓ main - Protected (current)

Run with --clean to delete these branches.
```

**Status Indicators:**
- `✗` (red) - Branch will be deleted
- `✓` (green) - Branch is protected
- `?` (yellow) - Branch doesn't match filters

**Color Coding:**
- Red: Branches to delete
- Green: Protected branches
- Yellow: Branches being kept (e.g., not merged)
- Blue: Informational messages

### 3.3 Confirmation Prompt

**Interactive Prompt:**
```rust
pub fn confirm_deletion(branches: &[BranchInfo]) -> Result<bool, anyhow::Error> {
    println!("\nDelete {} branches? [y/N]: ", branches.len());
    // Read user input
    // Return true if 'y' or 'Y'
}
```

## Phase 4: Testing & Robustness

### 4.1 Unit Tests

**`tests/filters.rs`:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_is_protected_exact_match() { /* ... */ }
    
    #[test]
    fn test_is_protected_glob_pattern() { /* ... */ }
    
    #[test]
    fn test_is_protected_regex_pattern() { /* ... */ }
    
    #[test]
    fn test_is_protected_current_branch() { /* ... */ }
    
    #[test]
    fn test_age_filter() { /* ... */ }
    
    #[test]
    fn test_merged_filter() { /* ... */ }
}
```

### 4.2 Integration Tests

**`tests/integration_test.rs`:**

Setup test git repository with branches:
```rust
#[test]
fn test_cleanup_merged_branches() {
    // Create test repository
    // Create main branch with commits
    // Create feature branches (some merged, some not)
    // Run git-tidy with --dry-run
    // Assert correct branches identified
    // Run git-tidy with --clean
    // Assert branches deleted correctly
}

#[test]
fn test_protected_branches_not_deleted() {
    // Create protected branches
    // Run cleanup
    // Assert protected branches remain
}

#[test]
fn test_keep_pattern_regex() {
    // Create branches matching pattern
    // Run cleanup with --keep-pattern
    // Assert pattern-matching branches kept
}

#[test]
fn test_age_filter() {
    // Create old and new branches
    // Run cleanup with --older-than
    // Assert only old branches identified
}
```

### 4.3 Edge Cases

**Test Scenarios:**
1. No branches to clean (all protected or not merged)
2. All branches merged (everything can be cleaned)
3. Detached HEAD state
4. Repository not a git repo
5. Branch with special characters in name

## Phase 5: Distribution

### 5.1 GitHub Actions for Binary Releases

**`.github/workflows/release.yml`:**

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  release:
    strategy:
      matrix:
        include:
          - target: x86_64-apple-darwin
            os: macos-latest
          - target: aarch64-apple-darwin
            os: macos-latest
          - target: x86_64-unknown-linux-gnu
            os: ubuntu-latest
          - target: aarch64-unknown-linux-gnu
            os: ubuntu-latest
          - target: x86_64-pc-windows-msvc
            os: windows-latest
    
    runs-on: ${{ matrix.os }}
    
    steps:
      - uses: actions/checkout@v4
      
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      
      - uses: Swatinem/rust-cache@v2
      
      - name: Build
        run: cargo build --release --target ${{ matrix.target }}
      
      - name: Package
        run: |
          if [[ "${{ matrix.os }}" == "windows-latest" ]]; then
            7z a git-tidy-${{ matrix.target }}.zip target/${{ matrix.target }}/release/git-tidy.exe
          else
            tar czf git-tidy-${{ matrix.target }}.tar.gz -C target/${{ matrix.target }}/release git-tidy
          fi
      
      - uses: softprops/action-gh-release@v1
        with:
          files: git-tidy-*
```

### 5.2 Documentation Updates

**Enhanced `README.md`:**

```markdown
# git-tidy

A simple command line utility to clean up your git branches safely.

## Installation

### via Cargo
```bash
cargo install git-tidy
```

### via Binary Release

Download from [GitHub Releases](https://github.com/yourusername/git-tidy/releases)

1. Download the appropriate binary for your platform
2. Extract and add to your PATH

## Usage

### Preview what would be cleaned (default behavior)
```bash
git-tidy
```

### Actually clean branches
```bash
git-tidy --clean
```

### Clean branches older than 7 days
```bash
git-tidy --clean --older-than=7d
```

### Keep branches matching a pattern
```bash
git-tidy --clean --keep-pattern="^hotfix/.*"
```

## Configuration

### Protected Branches

Default protected branches: `master`, `develop`, `main`

Add custom protected branches in `.git-tidy.toml`:

```toml
[protected_branches]
additional = ["release/*", "hotfix/*"]
```

### Global Configuration

Create `~/.config/git-tidy/config.toml`:

```toml
[protected_branches]
defaults = ["master", "develop", "main", "production"]
additional = ["release/*"]
```

## Options

- `--clean` - Actually delete branches (default: dry-run)
- `--dry-run` - Preview without deleting (default: true)
- `--merged` - Only show merged branches
- `--older-than=DURATION` - Filter by age (e.g., 30d, 2w, 1m)
- `--force` - Skip confirmation prompts
- `--keep-pattern=PATTERN` - Regex to protect matching branches

## Safety Features

- Dry-run by default - see what will be deleted before committing
- Protects current branch (HEAD)
- Respects protected branch configuration
- Confirmation prompt before deletion (unless --force)
- Never deletes unmerged branches

## Examples

See what would be cleaned:
```bash
$ git-tidy
Branches to delete (3):
  ✗ feature/auth - Merged 15 days ago
  ✗ feature/api - Merged 30 days ago
  ✗ bugfix/login - Merged 45 days ago

Run with --clean to delete these branches.
```

Clean branches with confirmation:
```bash
$ git-tidy --clean
Branches to delete (3):
  ✗ feature/auth - Merged 15 days ago
  ✗ feature/api - Merged 30 days ago
  ✗ bugfix/login - Merged 45 days ago

Delete these 3 branches? [y/N]: y
Deleted 3 branches.
```

## License

MIT
```

### 5.3 Additional Documentation

**`CONTRIBUTING.md`:**
- Development setup
- Running tests
- Code style guidelines
- Pull request process

**`CHANGELOG.md`:**
- Version history
- Breaking changes
- New features
- Bug fixes

## Implementation Checklist

### Phase 1: Core Functionality
- [ ] Add dependencies to Cargo.toml
- [ ] Create project structure (main.rs, git_operations.rs, filters.rs, config.rs)
- [ ] Implement branch listing functionality
- [ ] Implement merge status checking
- [ ] Implement age filtering
- [ ] Create CLI argument parser with clap
- [ ] Implement dry-run mode

### Phase 2: Configuration System
- [ ] Design config schema
- [ ] Implement config file loading (project and global)
- [ ] Implement config priority/merging logic
- [ ] Add protected branch matching (exact, glob, regex)
- [ ] Add --keep-pattern flag support

### Phase 3: Safety & UX
- [ ] Implement safety checks (protected, current branch)
- [ ] Add confirmation prompt
- [ ] Implement colored output
- [ ] Design and implement output format
- [ ] Add --force flag to skip confirmation
- [ ] Ensure dry-run is default

### Phase 4: Testing & Robustness
- [ ] Write unit tests for filters
- [ ] Write unit tests for config loading
- [ ] Add edge case tests

### Phase 5: Distribution
- [x] Set up GitHub Actions workflow
- [x] Update README with installation instructions
- [x] Add usage examples
- [x] Create CONTRIBUTING.md
- [x] Create CHANGELOG.md
- [ ] Test cross-platform builds
- [ ] Test binary installation process

## Success Criteria

The project will be considered production-ready when:

1. **Functionality**: All documented features work correctly
2. **Safety**: No branches are deleted unexpectedly
3. **Testing**: Test coverage > 80%
4. **Documentation**: README provides clear installation and usage instructions
5. **Distribution**: Binaries available for macOS, Linux, and Windows
6. **UX**: Tool works out of the box with sensible defaults
7. **Robustness**: Gracefully handles all error conditions
8. **Code Quality**: Passes `cargo clippy` and `cargo fmt`

## Timeline Estimate

- Phase 1: 2-3 days
- Phase 2: 1-2 days
- Phase 3: 1-2 days
- Phase 4: 2-3 days
- Phase 5: 1-2 days

**Total**: 7-12 days

## Notes

- All error messages should be actionable and include suggestions
- Default behavior should be safe (dry-run, confirmations)
- Configuration is optional - tool works without any config files
- Use Rust 2024 edition (already set in Cargo.toml)
- Follow Rust naming conventions and best practices
- Keep dependencies minimal but necessary
- Optimize for common use cases (simple cleanup of merged branches)
