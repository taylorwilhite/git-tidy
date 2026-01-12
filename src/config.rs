use anyhow::Result;
use chrono::Duration;
use regex::Regex;
use serde::Deserialize;

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
}

pub fn load_config() -> Result<Config> {
    Ok(Config::default())
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
