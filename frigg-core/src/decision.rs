use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Decision {
    Allow,
    Deny { rule_id: String, reason: String },
    Warn { rule_id: String, reason: String },
    AskHuman { rule_id: String, question: String },
}
