use chrono::{Duration, Utc};
use regex::Regex;

use crate::git_operations::BranchInfo;

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
