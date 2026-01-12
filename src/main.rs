mod config;
mod filters;
mod git_operations;

use anyhow::Result;
use chrono::{Duration, Utc};
use clap::Parser;
use colored::Colorize;
use regex::Regex;

use config::{load_config, parse_duration};
use filters::{filter_by_age, filter_by_merge_status, filter_out_protected};
use git_operations::{BranchInfo, delete_branch, get_current_branch, list_branches};

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
    older_than: Option<Duration>,

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

fn parse_regex(pattern: &str) -> Result<Regex, String> {
    Regex::new(pattern).map_err(|e| format!("Invalid regex: {}", e))
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = load_config()?;

    let repo = git2::Repository::open(".")?;

    let current_branch = get_current_branch(&repo)?;

    let branches = list_branches(&repo)?;

    let protected_patterns = config.get_protected_patterns()?;

    let mut branches_to_delete: Vec<BranchInfo> = Vec::new();
    let mut protected_branches: Vec<BranchInfo> = Vec::new();

    for branch in branches {
        let is_protected_exact = config.get_protected_branches().contains(&branch.name);
        let is_protected_glob = config.is_protected(&branch.name);
        let is_protected_regex = protected_patterns.iter().any(|p| p.is_match(&branch.name));
        let is_current_branch = current_branch.as_ref() == Some(&branch.name);
        let is_protected_cli = cli
            .keep_pattern
            .as_ref()
            .is_some_and(|p| p.is_match(&branch.name));

        let is_protected = is_protected_exact
            || is_protected_glob
            || is_protected_regex
            || is_current_branch
            || is_protected_cli;

        if is_protected {
            protected_branches.push(branch);
        } else {
            branches_to_delete.push(branch);
        }
    }

    let filtered: Vec<&BranchInfo> = branches_to_delete.iter().collect();

    let filtered = if cli.merged {
        filter_by_merge_status(&filtered, true)
    } else {
        filtered
    };

    let filtered = if let Some(older_than) = cli.older_than {
        filter_by_age(&filtered, older_than)
    } else {
        filtered
    };

    let filtered = filter_out_protected(
        &filtered,
        &config.get_protected_branches(),
        current_branch.as_deref(),
    );

    let branches_to_delete: Vec<&BranchInfo> = filtered;

    println!("{}:", "Branches to delete".bold(),);
    for branch in &branches_to_delete {
        println!(
            "   {} {} - {}",
            "✗".red(),
            branch.name,
            format_age(branch.last_commit_date)
        );
    }

    println!("\n{}:", "Protected branches".bold(),);
    for branch in &protected_branches {
        let reason = if current_branch.as_ref() == Some(&branch.name) {
            "current"
        } else if cli
            .keep_pattern
            .as_ref()
            .is_some_and(|p| p.is_match(&branch.name))
        {
            "cli pattern"
        } else if protected_patterns.iter().any(|p| p.is_match(&branch.name)) {
            "regex pattern"
        } else if config.is_protected(&branch.name) {
            "glob pattern"
        } else if config.get_protected_branches().contains(&branch.name) {
            "protected"
        } else {
            "pattern"
        };
        println!(
            "   {} {} - {}",
            "✓".green(),
            branch.name,
            format!("({})", reason).dimmed()
        );
    }

    if branches_to_delete.is_empty() {
        println!("\n{}", "No branches to delete.".green().bold());
        return Ok(());
    }

    if !cli.clean && cli.dry_run {
        println!(
            "\n{}",
            "Run with --clean to delete these branches.".blue().bold()
        );
        return Ok(());
    }

    if !cli.force && !confirm_deletion(&branches_to_delete)? {
        println!("{}", "Cancelled.".yellow());
        return Ok(());
    }

    let mut repo = git2::Repository::open(".")?;
    let mut deleted_count = 0;

    for branch in branches_to_delete {
        if cli.clean {
            match delete_branch(&mut repo, &branch.name) {
                Ok(_) => {
                    println!("{} {}", "Deleted".green(), branch.name);
                    deleted_count += 1;
                }
                Err(e) => {
                    println!("{} {}: {}", "Failed to delete".red(), branch.name, e);
                }
            }
        }
    }

    if cli.clean {
        println!(
            "\n{}",
            format!("Deleted {} branches.", deleted_count)
                .green()
                .bold()
        );
    }

    Ok(())
}

fn confirm_deletion(branches: &[&BranchInfo]) -> Result<bool> {
    println!("\nDelete {} branches? [y/N]: ", branches.len());

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    Ok(input.trim().to_lowercase() == "y")
}

fn format_age(date: chrono::DateTime<Utc>) -> String {
    let now = Utc::now();
    let duration = now.signed_duration_since(date);

    let days = duration.num_days();

    if days == 0 {
        let hours = duration.num_hours();
        if hours == 0 {
            format!("{} hours ago", duration.num_minutes())
        } else {
            format!("{} hours ago", hours)
        }
    } else if days == 1 {
        "1 day ago".to_string()
    } else if days < 30 {
        format!("{} days ago", days)
    } else if days < 365 {
        let months = days / 30;
        format!("{} month{} ago", months, if months > 1 { "s" } else { "" })
    } else {
        let years = days / 365;
        format!("{} year{} ago", years, if years > 1 { "s" } else { "" })
    }
}
