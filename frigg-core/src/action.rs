use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub name: String,
    #[serde(default)]
    pub params: HashMap<String, serde_json::Value>,
}
