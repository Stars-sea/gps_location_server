use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
pub struct ClientInfo {
    pub imei: String,
    pub iccid: String,
    pub fver: String,
    pub csq: i32,
}

impl ClientInfo {
    pub fn from_json(json_str: &str) -> Option<Self> {
        serde_json::from_str(json_str).ok()
    }

    pub fn identifier(&self) -> String {
        self.imei.clone()
    }
}

impl Display for ClientInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Client: [{}]", self.identifier())
    }
}
