use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Log,
    Warn,
    #[serde(alias = "ask_human")]
    AskHuman,
    Block,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub id: String,
    pub description: String,
    pub pattern: String,
    pub severity: Severity,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone)]
pub struct RuleSet {
    pub rules: Vec<Rule>,
}

impl RuleSet {
    pub fn from_yaml(path: &Path) -> Result<Self, String> {
        let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        Self::from_str(&content)
    }

    pub fn from_str(yaml: &str) -> Result<Self, String> {
        let rules: Vec<Rule> = serde_yaml::from_str(yaml).map_err(|e| e.to_string())?;
        Ok(Self { rules })
    }

    pub fn enabled_rules(&self) -> impl Iterator<Item = &Rule> {
        self.rules.iter().filter(|r| r.enabled)
    }
}

/// Simple glob match: `*` matches any sequence of chars.
pub fn glob_match(pattern: &str, text: &str) -> bool {
    let p = pattern.chars().collect::<Vec<_>>();
    let t = text.chars().collect::<Vec<_>>();
    glob_match_inner(&p, &t, 0, 0)
}

fn glob_match_inner(pattern: &[char], text: &[char], pi: usize, ti: usize) -> bool {
    if pi == pattern.len() {
        return ti == text.len();
    }
    if pattern[pi] == '*' {
        // Try matching * with 0..n characters
        for skip in 0..=(text.len() - ti) {
            if glob_match_inner(pattern, text, pi + 1, ti + skip) {
                return true;
            }
        }
        false
    } else if ti < text.len() && (pattern[pi] == '?' || pattern[pi] == text[ti]) {
        glob_match_inner(pattern, text, pi + 1, ti + 1)
    } else {
        false
    }
}
