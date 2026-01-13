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
