use crate::{glob_match, Action, Decision, RuleSet, Severity};

pub struct Engine {
    rules: RuleSet,
}

impl Engine {
    pub fn new(rules: RuleSet) -> Self {
        Self { rules }
    }

    pub fn check(&self, action: &Action) -> Decision {
        let mut best: Option<&crate::Rule> = None;

        for rule in self.rules.enabled_rules() {
            if glob_match(&rule.pattern, &action.name) {
                match &best {
                    None => best = Some(rule),
                    Some(current) if rule.severity > current.severity => best = Some(rule),
                    _ => {}
                }
            }
        }

        match best {
            None => Decision::Allow,
            Some(rule) => match rule.severity {
                Severity::Block => Decision::Deny {
                    rule_id: rule.id.clone(),
                    reason: rule.description.clone(),
                },
                Severity::AskHuman => Decision::AskHuman {
                    rule_id: rule.id.clone(),
                    question: format!("Rule '{}': {}. Allow?", rule.id, rule.description),
                },
                Severity::Warn => Decision::Warn {
                    rule_id: rule.id.clone(),
                    reason: rule.description.clone(),
                },
                Severity::Log => Decision::Allow,
            },
        }
    }
}
