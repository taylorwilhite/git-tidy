use std::fmt;
use std::path::PathBuf;

#[derive(Debug)]
pub enum GitTidyError {
    RepositoryNotFound(PathBuf),
    NotAGitRepository(PathBuf),
    BranchNotFound(String),
    BranchProtected(String),
    CurrentBranchCannotBeDeleted(String),
    GitError(git2::Error),
    ConfigError(String),
    InvalidRegex(String),
    PermissionDenied(PathBuf),
    ConcurrentGitOperation,
}

impl fmt::Display for GitTidyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RepositoryNotFound(path) => {
                write!(
                    f,
                    "Git repository not found at {}. Run this command inside a git repository.",
                    path.display()
                )
            }
            Self::NotAGitRepository(path) => {
                write!(f, "{} is not a git repository.", path.display())
            }
            Self::BranchNotFound(name) => {
                write!(f, "Branch '{}' not found.", name)
            }
            Self::BranchProtected(name) => {
                write!(
                    f,
                    "Branch '{}' is protected and cannot be deleted. Update your config if you want to delete it.",
                    name
                )
            }
            Self::CurrentBranchCannotBeDeleted(name) => {
                write!(
                    f,
                    "Cannot delete current branch '{}'. Switch to another branch first.",
                    name
                )
            }
            Self::GitError(err) => {
                write!(f, "Git error: {}", err)
            }
            Self::ConfigError(msg) => {
                write!(f, "Configuration error: {}", msg)
            }
            Self::InvalidRegex(pattern) => {
                write!(f, "Invalid regex pattern '{}': syntax error", pattern)
            }
            Self::PermissionDenied(path) => {
                write!(
                    f,
                    "Permission denied accessing {}. Check file permissions.",
                    path.display()
                )
            }
            Self::ConcurrentGitOperation => {
                write!(
                    f,
                    "Concurrent git operation detected. Please wait and try again."
                )
            }
        }
    }
}

impl std::error::Error for GitTidyError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::GitError(err) => Some(err),
            _ => None,
        }
    }
}

impl From<git2::Error> for GitTidyError {
    fn from(err: git2::Error) -> Self {
        GitTidyError::GitError(err)
    }
}
