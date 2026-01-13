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

pub fn load_config_from_path(path: &Path) -> Result<Option<Config>> {
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

    if duration_str.len() < 2 {
        return Err(format!(
            "Invalid duration: '{}'. Expected format like '30d'",
            duration_str
        ));
    }

    let split_pos = duration_str.len() - 1;
    let (num_str, unit) = duration_str.split_at(split_pos);

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_config_new() {
        let config = Config::new();

        let branches = config.get_protected_branches();
        assert!(branches.contains(&"master".to_string()));
        assert!(branches.contains(&"develop".to_string()));
        assert!(branches.contains(&"main".to_string()));
    }

    #[test]
    fn test_is_protected_exact_match() {
        let mut config = Config::new();
        config.protected_branches.additional = Some(vec!["staging".to_string()]);

        assert!(
            config
                .get_protected_branches()
                .contains(&"staging".to_string())
        );
    }

    #[test]
    fn test_is_protected_glob_pattern() {
        let mut config = Config::new();
        config.protected_branches.additional = Some(vec!["release/*".to_string()]);

        assert!(config.is_protected("release/1.0.0"));
        assert!(config.is_protected("release/2.0.0"));
        assert!(!config.is_protected("release"));
        assert!(!config.is_protected("feature/test"));
    }

    #[test]
    fn test_is_protected_regex_pattern() {
        let mut config = Config::new();
        config.protected_branches.patterns = Some(vec![r"^feature/.*-wip$".to_string()]);

        let patterns = config.get_protected_patterns().unwrap();
        assert!(patterns[0].is_match("feature/auth-wip"));
        assert!(patterns[0].is_match("feature/api-wip"));
        assert!(!patterns[0].is_match("feature/auth"));
        assert!(!patterns[0].is_match("bugfix/login"));
    }

    #[test]
    fn test_get_glob_patterns() {
        let mut config = Config::new();
        config.protected_branches.additional = Some(vec![
            "release/*".to_string(),
            "hotfix/*".to_string(),
            "master".to_string(),
        ]);

        let patterns = config.get_glob_patterns();
        assert_eq!(patterns.len(), 2);
        assert!(patterns.iter().any(|p| p.matches("release/1.0.0")));
        assert!(patterns.iter().any(|p| p.matches("hotfix/critical")));
    }

    #[test]
    fn test_merge_config() {
        let mut base = Config::new();
        let overlay = Config {
            protected_branches: ProtectedBranches {
                defaults: Some(vec!["production".to_string()]),
                additional: Some(vec!["staging".to_string()]),
                patterns: Some(vec![r"^feature/.*-wip$".to_string()]),
            },
        };

        merge_config(&mut base, &overlay);

        assert_eq!(
            base.protected_branches.defaults,
            Some(vec!["production".to_string()])
        );
        assert!(base.protected_branches.additional.is_some());
        assert!(
            base.protected_branches
                .additional
                .as_ref()
                .unwrap()
                .contains(&"staging".to_string())
        );
        assert!(base.protected_branches.patterns.is_some());
    }

    #[test]
    fn test_parse_duration() {
        assert_eq!(parse_duration("30s").unwrap(), Duration::seconds(30));
        assert_eq!(parse_duration("15m").unwrap(), Duration::minutes(15));
        assert_eq!(parse_duration("2h").unwrap(), Duration::hours(2));
        assert_eq!(parse_duration("30d").unwrap(), Duration::days(30));
        assert_eq!(parse_duration("4w").unwrap(), Duration::weeks(4));
    }

    #[test]
    fn test_parse_duration_invalid() {
        assert!(parse_duration("30x").is_err());
        assert!(parse_duration("abc").is_err());
        assert!(parse_duration("").is_err());
    }

    #[test]
    fn test_load_config_from_path() {
        let dir = std::env::temp_dir();
        let config_path = dir.join(format!("git-tidy-test-{}.toml", std::process::id()));

        let config_content = r#"
            [protected_branches]
            defaults = ["production"]
            additional = ["staging", "uat"]
            patterns = ["^feature/.*-wip$"]
        "#;

        fs::write(&config_path, config_content).unwrap();

        let config = load_config_from_path(&config_path).unwrap().unwrap();

        assert_eq!(
            config.protected_branches.defaults,
            Some(vec!["production".to_string()])
        );
        assert!(
            config
                .protected_branches
                .additional
                .as_ref()
                .unwrap()
                .contains(&"staging".to_string())
        );
        assert!(
            config
                .protected_branches
                .additional
                .as_ref()
                .unwrap()
                .contains(&"uat".to_string())
        );
    }

    #[test]
    fn test_load_config_from_path_not_found() {
        let dir = std::env::temp_dir();
        let config_path = dir.join("git-tidy-test-nonexistent.toml");

        let config = load_config_from_path(&config_path).unwrap();
        assert!(config.is_none());
    }

    #[test]
    fn test_load_config_from_path_invalid_toml() {
        let dir = std::env::temp_dir();
        let config_path = dir.join(format!("git-tidy-test-invalid-{}.toml", std::process::id()));

        fs::write(&config_path, "invalid [toml content").unwrap();

        let result = load_config_from_path(&config_path);
        assert!(result.is_err());

        let _ = std::fs::remove_file(&config_path);
    }
}
