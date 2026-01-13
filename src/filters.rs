use chrono::{Duration, Utc};
use regex::Regex;

use crate::git_operations::BranchInfo;

#[allow(dead_code)]
pub fn filter_by_merge_status<'a>(
    branches: &'a [&'a BranchInfo],
    merged_only: bool,
) -> Vec<&'a BranchInfo> {
    branches
        .iter()
        .filter(|b| !merged_only || b.is_merged)
        .copied()
        .collect()
}

pub fn filter_by_age<'a>(
    branches: &'a [&'a BranchInfo],
    older_than: Duration,
) -> Vec<&'a BranchInfo> {
    let cutoff = Utc::now() - older_than;

    branches
        .iter()
        .filter(|b| b.last_commit_date <= cutoff)
        .copied()
        .collect()
}

#[allow(dead_code)]
pub fn filter_by_pattern<'a>(
    branches: &'a [&'a BranchInfo],
    pattern: &Regex,
) -> Vec<&'a BranchInfo> {
    branches
        .iter()
        .filter(|b| pattern.is_match(&b.name))
        .copied()
        .collect()
}

pub fn filter_out_protected<'a>(
    branches: &'a [&'a BranchInfo],
    protected_branches: &[String],
    current_branch: Option<&str>,
) -> Vec<&'a BranchInfo> {
    branches
        .iter()
        .filter(|b| {
            !protected_branches.contains(&b.name) && current_branch != Some(b.name.as_str())
        })
        .copied()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, TimeZone, Utc};

    fn create_test_branch(name: &str, is_merged: bool, days_ago: i64) -> BranchInfo {
        BranchInfo {
            name: name.to_string(),
            is_merged,
            last_commit_date: Utc::now() - Duration::days(days_ago),
            is_remote: false,
        }
    }

    #[test]
    fn test_filter_by_age() {
        let now = Utc::now();
        let branches = vec![
            create_test_branch("old-feature", true, 45),
            create_test_branch("new-feature", true, 15),
            create_test_branch("ancient-feature", true, 90),
        ];

        let branches_vec: Vec<_> = branches.iter().collect();
        let filtered = filter_by_age(&branches_vec, Duration::days(30));

        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().any(|b| b.name == "old-feature"));
        assert!(filtered.iter().any(|b| b.name == "ancient-feature"));
        assert!(!filtered.iter().any(|b| b.name == "new-feature"));
    }

    #[test]
    fn test_filter_by_age_exact_cutoff() {
        let branches = vec![
            create_test_branch("exactly-30-days", true, 30),
            create_test_branch("31-days", true, 31),
        ];

        let branches_vec: Vec<_> = branches.iter().collect();
        let filtered = filter_by_age(&branches_vec, Duration::days(30));

        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().any(|b| b.name == "31-days"));
        assert!(filtered.iter().any(|b| b.name == "exactly-30-days"));
    }

    #[test]
    fn test_filter_out_protected() {
        let branches = vec![
            create_test_branch("master", true, 1),
            create_test_branch("develop", true, 1),
            create_test_branch("feature-1", true, 1),
            create_test_branch("feature-2", true, 1),
        ];

        let protected = vec!["master".to_string(), "develop".to_string()];
        let current_branch = Some("feature-1");

        let branches_vec: Vec<_> = branches.iter().collect();
        let filtered = filter_out_protected(&branches_vec, &protected, current_branch);

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "feature-2");
    }

    #[test]
    fn test_filter_out_protected_current_branch() {
        let branches = vec![
            create_test_branch("master", true, 1),
            create_test_branch("feature-1", true, 1),
            create_test_branch("feature-2", true, 1),
        ];

        let protected = vec!["master".to_string()];
        let current_branch = Some("feature-1");

        let branches_vec: Vec<_> = branches.iter().collect();
        let filtered = filter_out_protected(&branches_vec, &protected, current_branch);

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "feature-2");
    }

    #[test]
    fn test_filter_by_merge_status() {
        let branches = vec![
            create_test_branch("merged-feature", true, 30),
            create_test_branch("unmerged-feature", false, 30),
            create_test_branch("another-merged", true, 30),
        ];

        let branches_vec: Vec<_> = branches.iter().collect();
        let filtered = filter_by_merge_status(&branches_vec, true);

        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|b| b.is_merged));
    }

    #[test]
    fn test_filter_by_pattern() {
        let branches = vec![
            create_test_branch("feature/auth", true, 30),
            create_test_branch("feature/api", true, 30),
            create_test_branch("bugfix/login", true, 30),
        ];

        let pattern = Regex::new(r"^feature/").unwrap();
        let branches_vec: Vec<_> = branches.iter().collect();
        let filtered = filter_by_pattern(&branches_vec, &pattern);

        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|b| b.name.starts_with("feature/")));
    }
}
