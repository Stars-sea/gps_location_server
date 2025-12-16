use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
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
