use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClientCommand {
    pub target: Option<String>,
    pub command: String,
}

impl ClientCommand {
    pub fn new(target: Option<String>, command: String) -> Self {
        Self { target, command }
    }

    pub fn new_broadcast(command: String) -> Self {
        Self {
            target: None,
            command,
        }
    }

    pub fn is_targeted(&self, id: &str) -> bool {
        match &self.target {
            Some(t) => t == id,
            None => true,
        }
    }
}

impl FromStr for ClientCommand {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let parts: Vec<&str> = s.splitn(2, ':').collect();
        let command = if parts.len() == 2 {
            let target = parts[0].to_string();
            let command = parts[1].to_string();
            ClientCommand::new(Some(target), command)
        } else {
            ClientCommand::new_broadcast(s.to_string())
        };

        Ok(command)
    }
}

impl Display for ClientCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let target = match &self.target {
            Some(target) => &target,
            None => "all",
        };
        write!(f, "{}:{}", target, self.command)
    }
}
