use std::fmt::{Display, Formatter};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RegisteredClientInfo {
    pub base_info: ClientInfo,

    pub registered_name: Option<String>,
    pub tags: Vec<String>,

    pub first_seen: String,
    pub last_seen: String,
}

impl RegisteredClientInfo {
    const FILE_NAME: &str = "registered_info.json";

    pub async fn load() -> Result<Vec<Self>> {
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
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            base_info: info,
            registered_name: None,
            tags: Vec::new(),
            first_seen: now.clone(),
            last_seen: now,
        }
    }

    pub async fn find_or_create(imei: &str, info: &ClientInfo) -> Self {
        match Self::find(imei).await {
            Some(registered_info) => registered_info,
            None => Self::create(info.clone()).await,
        }
    }

    pub fn update_last_seen(&mut self) {
        self.last_seen = chrono::Utc::now().to_rfc3339();
    }

    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }

    pub fn remove_tag(&mut self, tag: &str) {
        self.tags.retain(|t| t != tag);
    }

    pub fn set_registered_name(&mut self, name: String) {
        self.registered_name = Some(name);
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
