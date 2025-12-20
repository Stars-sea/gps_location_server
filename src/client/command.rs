use crate::server;

#[derive(Debug, Clone, PartialEq, Eq)]
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

    pub fn builder() -> ClientCommandBuilder {
        ClientCommandBuilder::default()
    }

    pub fn is_targeted(&self, id: &str) -> bool {
        match &self.target {
            Some(t) => t == id,
            None => true,
        }
    }
}

impl From<server::SendCommandToClientRequest> for ClientCommand {
    fn from(req: server::SendCommandToClientRequest) -> Self {
        ClientCommand::new(req.imei, req.command)
    }
}

impl Into<server::SendCommandToClientRequest> for ClientCommand {
    fn into(self) -> server::SendCommandToClientRequest {
        server::SendCommandToClientRequest {
            imei: self.target,
            command: self.command,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ClientCommandBuilder {
    target: Option<String>,
    command: String,
}

impl ClientCommandBuilder {
    pub fn target(mut self, target: String) -> Self {
        self.target = Some(target);
        self
    }

    pub fn broadcast(mut self) -> Self {
        self.target = None;
        self
    }

    pub fn command(mut self, command: String) -> Self {
        self.command = command;
        self
    }

    pub fn build(self) -> ClientCommand {
        ClientCommand {
            target: self.target,
            command: self.command,
        }
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
