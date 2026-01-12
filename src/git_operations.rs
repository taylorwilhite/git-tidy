use anyhow::Result;
use chrono::{DateTime, TimeZone, Utc};
use git2::{BranchType, Repository};

use crate::config::Config;

#[derive(Clone)]
pub struct BranchInfo {
    pub name: String,
    pub is_merged: bool,
    pub last_commit_date: DateTime<Utc>,
    #[allow(dead_code)]
    pub is_remote: bool,
}

pub fn list_branches(repo: &Repository) -> Result<Vec<BranchInfo>> {
    let mut branches = Vec::new();

    for branch_type in [BranchType::Local] {
        let branch_names = repo.branches(Some(branch_type))?;

        for branch in branch_names {
            let (branch_obj, _branch_type) = branch?;
            let name = branch_obj.name()?.unwrap_or("unknown").to_string();

            let commit = branch_obj.get().peel_to_commit()?;
            let time = commit.time();
            let last_commit_date = Utc.timestamp_opt(time.seconds(), 0).unwrap();

            let is_merged = is_branch_merged(repo, &name)?;

            branches.push(BranchInfo {
                name,
                is_merged,
                last_commit_date,
                is_remote: branch_type == BranchType::Remote,
            });
        }
    }

    branches.sort_by(|a, b| b.last_commit_date.cmp(&a.last_commit_date));

    Ok(branches)
}

pub fn safe_delete_branch(
    repo: &mut git2::Repository,
    branch_name: &str,
    config: &Config,
    current_branch: Option<&str>,
    force: bool,
) -> Result<()> {
    if current_branch == Some(branch_name) {
        anyhow::bail!(
            "Cannot delete current branch '{}'. Switch to another branch first.",
            branch_name
        );
    }

    if config
        .get_protected_branches()
        .iter()
        .any(|b| b == branch_name)
    {
        anyhow::bail!(
            "Branch '{}' is protected and cannot be deleted. Update your config if you want to delete it.",
            branch_name
        );
    }

    if config.is_protected(branch_name) {
        anyhow::bail!(
            "Branch '{}' is protected by a glob pattern and cannot be deleted. Update your config if you want to delete it.",
            branch_name
        );
    }

    if !is_branch_merged(repo, branch_name)? {
        anyhow::bail!(
            "Branch '{}' is not merged. Refusing to delete unmerged branch. Use 'git branch -D {}' if you really want to delete it.",
            branch_name,
            branch_name
        );
    }

    if !force {
        confirm_deletion(branch_name)?;
    }

    let mut branch = repo.find_branch(branch_name, BranchType::Local)?;
    branch.delete()?;

    Ok(())
}

#[allow(dead_code)]
pub fn delete_branch(repo: &mut git2::Repository, branch_name: &str) -> Result<()> {
    let mut branch = repo.find_branch(branch_name, BranchType::Local)?;
    branch.delete()?;
    Ok(())
}

pub fn get_current_branch(repo: &Repository) -> Result<Option<String>> {
    let head = repo.head()?;

    if head.is_branch() {
        let branch_name = head.shorthand().map(|s| s.to_string());
        Ok(branch_name)
    } else {
        Ok(None)
    }
}

fn is_branch_merged(repo: &Repository, branch_name: &str) -> Result<bool> {
    let branch = repo.find_branch(branch_name, BranchType::Local)?;
    let branch_commit = branch.get().peel_to_commit()?;

    if let Ok(main) = repo.find_branch("main", BranchType::Local) {
        let main_commit = main.get().peel_to_commit()?;

        return Ok(repo
            .graph_descendant_of(branch_commit.id(), main_commit.id())
            .unwrap_or(false));
    }

    if let Ok(master) = repo.find_branch("master", BranchType::Local) {
        let master_commit = master.get().peel_to_commit()?;

        return Ok(repo
            .graph_descendant_of(branch_commit.id(), master_commit.id())
            .unwrap_or(false));
    }

    Ok(false)
}

fn confirm_deletion(branch_name: &str) -> Result<bool> {
    println!("Delete branch '{}'? [y/N]: ", branch_name);

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    if input.trim().to_lowercase() != "y" {
        anyhow::bail!("Deletion cancelled by user");
    }

    Ok(true)
}
