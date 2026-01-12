use anyhow::Result;
use chrono::Duration;
use glob::Pattern;
use regex::Regex;
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize, Default)]
pub struct Config {
    pub protected_branches: ProtectedBranches,
}

#[derive(Debug, Deserialize, Default)]
pub struct ProtectedBranches {
    pub defaults: Option<Vec<String>>,
    pub additional: Option<Vec<String>>,
    pub patterns: Option<Vec<String>>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            protected_branches: ProtectedBranches {
                defaults: Some(vec![
                    "master".to_string(),
                    "develop".to_string(),
                    "main".to_string(),
                ]),
                additional: None,
                patterns: None,
            },
        }
    }

    pub fn get_protected_branches(&self) -> Vec<String> {
        let mut branches = self.protected_branches.defaults.clone().unwrap_or_default();

        if let Some(additional) = &self.protected_branches.additional {
            branches.extend(additional.clone());
        }

        branches
    }

    pub fn get_protected_patterns(&self) -> Result<Vec<Regex>> {
        let empty = vec![];
        let patterns = self.protected_branches.patterns.as_ref().unwrap_or(&empty);

        patterns
            .iter()
            .map(|p| Regex::new(p).map_err(|e| anyhow::anyhow!("Invalid regex '{}': {}", p, e)))
            .collect()
    }

    pub fn get_glob_patterns(&self) -> Vec<Pattern> {
        let mut patterns = Vec::new();

        for branch in self.get_protected_branches() {
            if branch.contains('*')
                && let Ok(pattern) = Pattern::new(&branch)
            {
                patterns.push(pattern);
            }
        }

        patterns
    }

    pub fn is_protected(&self, branch_name: &str) -> bool {
        for pattern in &self.get_glob_patterns() {
            if pattern.matches(branch_name) {
                return true;
            }
        }

        false
    }
}

pub fn load_config() -> Result<Config> {
    let global_config = load_global_config()?;
    let project_config = load_project_config()?;

    let mut config = Config::new();

    if let Some(global) = global_config {
        merge_config(&mut config, &global);
    }

    if let Some(project) = project_config {
        merge_config(&mut config, &project);
    }

    Ok(config)
}

fn merge_config(base: &mut Config, overlay: &Config) {
    if let Some(overlay_defaults) = &overlay.protected_branches.defaults {
        base.protected_branches.defaults = Some(overlay_defaults.clone());
    }

    if let Some(overlay_additional) = &overlay.protected_branches.additional {
        let base_additional = base
            .protected_branches
            .additional
            .get_or_insert_with(Vec::new);
        base_additional.extend(overlay_additional.clone());
        base_additional.sort();
        base_additional.dedup();
    }

    if let Some(overlay_patterns) = &overlay.protected_branches.patterns {
        let base_patterns = base
            .protected_branches
            .patterns
            .get_or_insert_with(Vec::new);
        base_patterns.extend(overlay_patterns.clone());
        base_patterns.sort();
        base_patterns.dedup();
    }
}

fn load_global_config() -> Result<Option<Config>> {
    let mut path =
        dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;

    path.push(".config");
    path.push("git-tidy");
    path.push("config.toml");

    load_config_from_path(&path)
}

fn load_project_config() -> Result<Option<Config>> {
    let path = PathBuf::from(".git-tidy.toml");
    load_config_from_path(&path)
}

fn load_config_from_path(path: &Path) -> Result<Option<Config>> {
    if !path.exists() {
        return Ok(None);
    }

    let contents = std::fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("Failed to read config file {}: {}", path.display(), e))?;

    let config: Config = toml::from_str(&contents)
        .map_err(|e| anyhow::anyhow!("Failed to parse config file {}: {}", path.display(), e))?;

    Ok(Some(config))
}

pub fn parse_duration(duration_str: &str) -> Result<Duration, String> {
    let duration_str = duration_str.trim();
    let (num_str, unit) = duration_str.split_at(duration_str.len() - 1);

    let num: i64 = num_str
        .parse()
        .map_err(|_| format!("Invalid number: {}", num_str))?;

    match unit {
        "s" => Ok(Duration::seconds(num)),
        "m" => Ok(Duration::minutes(num)),
        "h" => Ok(Duration::hours(num)),
        "d" => Ok(Duration::days(num)),
        "w" => Ok(Duration::weeks(num)),
        _ => Err(format!("Invalid unit: {}. Use s, m, h, d, or w", unit)),
    }
}
