pub mod action;
pub mod decision;
pub mod engine;
pub mod logger;
pub mod rule;

pub use action::Action;
pub use decision::Decision;
pub use engine::Engine;
pub use logger::Logger;
pub use rule::{glob_match, Rule, RuleSet, Severity};

use std::path::Path;

/// The result of a Frigg check — the final outcome after mitigation.
#[derive(Debug, Clone, PartialEq)]
pub enum CheckResult {
    Allowed,
    Blocked { rule_id: String, reason: String },
    Warned { rule_id: String, reason: String },
}

/// Main entry point. Composes engine + logger + ask-human into one call.
pub struct Frigg {
    engine: Engine,
    logger: Logger,
}

impl Frigg {
    pub fn from_config(rules_path: &Path, log_path: &Path) -> Result<Self, String> {
        let rules = RuleSet::from_yaml(rules_path)?;
        let logger = Logger::new(log_path)?;
        Ok(Self { engine: Engine::new(rules), logger })
    }

    /// Check an action against rules, log the event, and return the result.
    /// For `AskHuman` decisions, calls `ask_fn` with the question and expects a bool (true=allow).
    pub fn check_with<F>(&mut self, action: &Action, ask_fn: F) -> CheckResult
    where
        F: FnOnce(&str) -> bool,
    {
        let decision = self.engine.check(action);
        let _ = self.logger.log(action, &decision);

        match &decision {
            Decision::Allow => CheckResult::Allowed,
            Decision::Deny { rule_id, reason } => CheckResult::Blocked {
                rule_id: rule_id.clone(),
                reason: reason.clone(),
            },
            Decision::Warn { rule_id, reason } => CheckResult::Warned {
                rule_id: rule_id.clone(),
                reason: reason.clone(),
            },
            Decision::AskHuman { rule_id, question } => {
                if ask_fn(question) {
                    CheckResult::Allowed
                } else {
                    CheckResult::Blocked {
                        rule_id: rule_id.clone(),
                        reason: "Denied by human".into(),
                    }
                }
            }
        }
    }

    /// Check with CLI stdin prompt for ask-human.
    pub fn check(&mut self, action: &Action) -> CheckResult {
        self.check_with(action, |question| {
            eprintln!("[frigg] {question} (y/n)");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap_or(0);
            input.trim().eq_ignore_ascii_case("y")
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn action_roundtrip() {
        let action = Action { name: "file_delete".into(), params: HashMap::from([("path".into(), serde_json::json!("/etc"))]) };
        let json = serde_json::to_string(&action).unwrap();
        let back: Action = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, "file_delete");
    }

    #[test]
    fn severity_ordering() {
        assert!(Severity::Block > Severity::AskHuman);
        assert!(Severity::AskHuman > Severity::Warn);
        assert!(Severity::Warn > Severity::Log);
    }

    #[test]
    fn rule_yaml_roundtrip() {
        let yaml = "id: r1\ndescription: test\npattern: \"file_*\"\nseverity: block\n";
        let rule: Rule = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(rule.id, "r1");
        assert_eq!(rule.severity, Severity::Block);
        assert!(rule.enabled);
    }

    #[test]
    fn decision_roundtrip() {
        let d = Decision::Deny { rule_id: "r1".into(), reason: "not allowed".into() };
        let json = serde_json::to_string(&d).unwrap();
        let back: Decision = serde_json::from_str(&json).unwrap();
        assert_eq!(back, d);
    }

    #[test]
    fn ruleset_from_str() {
        let yaml = r#"
- id: r1
  description: block deletes
  pattern: "file_delete*"
  severity: block
- id: r2
  description: warn writes
  pattern: "file_write*"
  severity: warn
  enabled: false
"#;
        let rs = RuleSet::from_str(yaml).unwrap();
        assert_eq!(rs.rules.len(), 2);
        assert_eq!(rs.enabled_rules().count(), 1);
    }

    #[test]
    fn ruleset_from_yaml_file() {
        let rs = RuleSet::from_yaml(std::path::Path::new("../rules.example.yaml")).unwrap();
        assert_eq!(rs.rules.len(), 5);
        assert!(rs.rules.iter().any(|r| r.severity == Severity::Block));
        assert!(rs.rules.iter().any(|r| r.severity == Severity::AskHuman));
    }

    #[test]
    fn ruleset_malformed_yaml() {
        let result = RuleSet::from_str("not: valid: yaml: [");
        assert!(result.is_err());
    }

    #[test]
    fn glob_matching() {
        use crate::glob_match;
        assert!(glob_match("file_delete*", "file_delete"));
        assert!(glob_match("file_delete*", "file_delete_all"));
        assert!(!glob_match("file_delete*", "file_write"));
        assert!(glob_match("bash:rm *", "bash:rm -rf /"));
        assert!(glob_match("http_*", "http_post"));
        assert!(!glob_match("http_*", "ftp_get"));
        assert!(glob_match("*", "anything"));
    }

    #[test]
    fn engine_block_decision() {
        let rs = RuleSet::from_str("- id: r1\n  description: no deletes\n  pattern: \"file_delete*\"\n  severity: block\n").unwrap();
        let engine = Engine::new(rs);
        let action = Action { name: "file_delete_all".into(), params: HashMap::new() };
        assert!(matches!(engine.check(&action), Decision::Deny { .. }));
    }

    #[test]
    fn engine_allow_no_match() {
        let rs = RuleSet::from_str("- id: r1\n  description: no deletes\n  pattern: \"file_delete*\"\n  severity: block\n").unwrap();
        let engine = Engine::new(rs);
        let action = Action { name: "file_read".into(), params: HashMap::new() };
        assert_eq!(engine.check(&action), Decision::Allow);
    }

    #[test]
    fn engine_highest_severity_wins() {
        let yaml = r#"
- id: r1
  description: warn
  pattern: "file_*"
  severity: warn
- id: r2
  description: block
  pattern: "file_delete*"
  severity: block
"#;
        let engine = Engine::new(RuleSet::from_str(yaml).unwrap());
        let action = Action { name: "file_delete".into(), params: HashMap::new() };
        match engine.check(&action) {
            Decision::Deny { rule_id, .. } => assert_eq!(rule_id, "r2"),
            other => panic!("expected Deny, got {:?}", other),
        }
    }

    #[test]
    fn engine_ask_human() {
        let rs = RuleSet::from_str("- id: r1\n  description: needs approval\n  pattern: \"http_*\"\n  severity: ask_human\n").unwrap();
        let engine = Engine::new(rs);
        let action = Action { name: "http_post".into(), params: HashMap::new() };
        assert!(matches!(engine.check(&action), Decision::AskHuman { .. }));
    }

    #[test]
    fn engine_warn() {
        let rs = RuleSet::from_str("- id: r1\n  description: careful\n  pattern: \"write_*\"\n  severity: warn\n").unwrap();
        let engine = Engine::new(rs);
        let action = Action { name: "write_config".into(), params: HashMap::new() };
        assert!(matches!(engine.check(&action), Decision::Warn { .. }));
    }

    #[test]
    fn logger_writes_json_lines() {
        let dir = std::env::temp_dir().join("frigg_test_log");
        let _ = std::fs::remove_file(&dir);
        let mut logger = Logger::new(&dir).unwrap();
        let action = Action { name: "test_action".into(), params: HashMap::new() };
        logger.log(&action, &Decision::Allow).unwrap();
        logger.log(&action, &Decision::Deny { rule_id: "r1".into(), reason: "no".into() }).unwrap();
        drop(logger);

        let content = std::fs::read_to_string(&dir).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 2);
        for line in &lines {
            let _: serde_json::Value = serde_json::from_str(line).expect("each line must be valid JSON");
        }
        std::fs::remove_file(&dir).unwrap();
    }

    #[test]
    fn frigg_facade_block() {
        let log_path = std::env::temp_dir().join("frigg_facade_test.log");
        let _ = std::fs::remove_file(&log_path);
        let rules_path = std::path::Path::new("../rules.example.yaml");
        let mut frigg = Frigg::from_config(rules_path, &log_path).unwrap();
        let action = Action { name: "file_delete_all".into(), params: HashMap::new() };
        let result = frigg.check_with(&action, |_| true);
        assert!(matches!(result, CheckResult::Blocked { .. }));
        std::fs::remove_file(&log_path).unwrap();
    }

    #[test]
    fn frigg_facade_allow() {
        let log_path = std::env::temp_dir().join("frigg_facade_allow.log");
        let _ = std::fs::remove_file(&log_path);
        let rules_path = std::path::Path::new("../rules.example.yaml");
        let mut frigg = Frigg::from_config(rules_path, &log_path).unwrap();
        let action = Action { name: "something_safe".into(), params: HashMap::new() };
        assert_eq!(frigg.check_with(&action, |_| true), CheckResult::Allowed);
        std::fs::remove_file(&log_path).unwrap();
    }

    #[test]
    fn frigg_facade_ask_human_yes() {
        let log_path = std::env::temp_dir().join("frigg_facade_ask_y.log");
        let _ = std::fs::remove_file(&log_path);
        let rules_path = std::path::Path::new("../rules.example.yaml");
        let mut frigg = Frigg::from_config(rules_path, &log_path).unwrap();
        let action = Action { name: "http_post".into(), params: HashMap::new() };
        let result = frigg.check_with(&action, |_| true);
        assert_eq!(result, CheckResult::Allowed);
        std::fs::remove_file(&log_path).unwrap();
    }

    #[test]
    fn frigg_facade_ask_human_no() {
        let log_path = std::env::temp_dir().join("frigg_facade_ask_n.log");
        let _ = std::fs::remove_file(&log_path);
        let rules_path = std::path::Path::new("../rules.example.yaml");
        let mut frigg = Frigg::from_config(rules_path, &log_path).unwrap();
        let action = Action { name: "http_post".into(), params: HashMap::new() };
        let result = frigg.check_with(&action, |_| false);
        assert!(matches!(result, CheckResult::Blocked { .. }));
        std::fs::remove_file(&log_path).unwrap();
    }
}
