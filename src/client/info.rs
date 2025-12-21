use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::server::grpc::ProtoClientInfo;

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

impl From<ProtoClientInfo> for ClientInfo {
    fn from(proto: ProtoClientInfo) -> Self {
        Self {
            imei: proto.imei,
            iccid: proto.iccid,
            fver: proto.fver,
            csq: proto.csq,
        }
    }
}

impl Into<ProtoClientInfo> for ClientInfo {
    fn into(self) -> ProtoClientInfo {
        ProtoClientInfo {
            imei: self.imei,
            iccid: self.iccid,
            fver: self.fver,
            csq: self.csq,
        }
    }
}
