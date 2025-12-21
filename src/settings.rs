use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::{fs::File, io::AsyncReadExt};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Settings {
    pub address: String,
    #[cfg(feature = "grpc")]
    pub grpc: ServiceConfig,
    #[cfg(feature = "rest")]
    pub rest: ServiceConfig,

    pub heartbeat_sec: u64,
    pub output_dir: String,
    pub verify_timeout: u64,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct ServiceConfig {
    pub enabled: bool,
    pub address: String,
}

pub async fn load_from_file(path: &str) -> Result<Settings> {
    let mut file = File::open(path).await?;

    let mut data = String::new();
    file.read_to_string(&mut data).await?;

    let json: Settings = serde_json::from_str(&data)?;
    Ok(json)
}
