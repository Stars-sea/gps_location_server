use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::server::grpc::SendCommandToClientRequest;

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

impl From<SendCommandToClientRequest> for ClientCommand {
    fn from(req: SendCommandToClientRequest) -> Self {
        ClientCommand::new(req.imei, req.command)
    }
}

impl Into<SendCommandToClientRequest> for ClientCommand {
    fn into(self) -> SendCommandToClientRequest {
        SendCommandToClientRequest {
            imei: self.target,
            command: self.command,
        }
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

pub fn parse_command(input: &str) -> ClientCommand {
    let parts: Vec<&str> = input.splitn(2, ':').collect();
    if parts.len() == 2 {
        let target = parts[0].to_string();
        let command = parts[1].to_string();
        ClientCommand::new(Some(target), command)
    } else {
        ClientCommand::new_broadcast(input.to_string())
    }
}
