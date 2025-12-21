use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClientCommand {
    target: Vec<String>,
    pub command: String,
}

impl ClientCommand {
    pub fn new(target: Vec<String>, command: String) -> Self {
        Self { target, command }
    }

    pub fn new_broadcast(command: String) -> Self {
        Self {
            target: Vec::new(),
            command,
        }
    }

    pub fn is_targeted(&self, id: &str) -> bool {
        if self.target.is_empty() {
            true
        } else {
            self.target.contains(&id.to_string())
        }
    }
}

impl FromStr for ClientCommand {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let parts: Vec<&str> = s.splitn(2, ':').collect();
        let command = if parts.len() == 2 {
            let targets = parts[0].to_string();
            let command = parts[1].to_string();

            let target = targets.split(',').map(|s| s.trim().to_string()).collect();
            ClientCommand::new(target, command)
        } else {
            ClientCommand::new_broadcast(s.to_string())
        };

        Ok(command)
    }
}

impl Display for ClientCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let target = if self.target.is_empty() {
            "ALL".to_string()
        } else {
            self.target.join(",")
        };
        write!(f, "{}:{}", target, self.command)
    }
}
