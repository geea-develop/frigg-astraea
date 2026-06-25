use crate::{Action, Decision};
use chrono::Utc;
use serde::Serialize;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;

#[derive(Debug, Serialize)]
pub struct LogEvent {
    pub timestamp: String,
    pub action: Action,
    pub decision: Decision,
}

pub struct Logger {
    file: File,
}

impl Logger {
    pub fn new(path: &Path) -> Result<Self, String> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|e| e.to_string())?;
        Ok(Self { file })
    }

    pub fn log(&mut self, action: &Action, decision: &Decision) -> Result<(), String> {
        let event = LogEvent {
            timestamp: Utc::now().to_rfc3339(),
            action: action.clone(),
            decision: decision.clone(),
        };
        let mut line = serde_json::to_string(&event).map_err(|e| e.to_string())?;
        line.push('\n');
        self.file.write_all(line.as_bytes()).map_err(|e| e.to_string())
    }
}
