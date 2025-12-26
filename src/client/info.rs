use std::fmt::{Display, Formatter};

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
pub struct ClientInfo {
    pub imei: String,
    pub iccid: String,
    pub fver: String,

    #[serde(skip_serializing)]
    pub csq: Option<i32>,
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RegisteredClientInfo {
    pub base_info: ClientInfo,

    pub name: Option<String>,
    pub tags: Vec<String>,

    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
}

impl RegisteredClientInfo {
    const FILE_NAME: &str = "registered_infos.json";

    pub async fn load() -> Result<Vec<Self>> {
        if !fs::try_exists(Self::FILE_NAME).await.unwrap_or(false) {
            return Ok(Vec::new());
        }

        let mut file = fs::File::open(Self::FILE_NAME).await?;
        let mut dst = String::new();
        file.read_to_string(&mut dst).await?;
        Ok(serde_json::from_str(&dst)?)
    }

    pub async fn find(imei: &str) -> Option<Self> {
        let registered_clients = Self::load().await.unwrap_or_default();
        registered_clients
            .iter()
            .find(|&info| info.base_info.imei == imei)
            .cloned()
    }

    pub async fn create(info: ClientInfo) -> Self {
        let now = chrono::Utc::now();
        Self {
            base_info: info,
            name: None,
            tags: Vec::new(),
            first_seen: now.clone(),
            last_seen: now,
        }
    }

    pub fn update_last_seen(&mut self) {
        self.last_seen = chrono::Utc::now();
    }

    pub fn set_name(&mut self, name: String) {
        self.name = if name.is_empty() { None } else { Some(name) };
    }

    pub async fn save(&self) -> Result<()> {
        let mut registered_clients = Self::load().await?;

        if let Some(pos) = registered_clients.iter().position(|info| info == self) {
            registered_clients[pos] = self.clone();
        } else {
            registered_clients.push(self.clone());
        }

        Self::save_all(&registered_clients).await?;
        Ok(())
    }

    async fn save_all(clients: &[Self]) -> Result<()> {
        let mut file = fs::File::create(Self::FILE_NAME).await?;

        let data = serde_json::to_string_pretty(clients)?;
        file.write_all(data.as_bytes()).await?;
        Ok(())
    }
}

impl PartialEq for RegisteredClientInfo {
    fn eq(&self, other: &Self) -> bool {
        self.base_info.imei == other.base_info.imei
    }
}
