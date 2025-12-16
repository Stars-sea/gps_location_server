use anyhow::Result;
use serde::{Serialize, Deserialize};
use tokio::{fs::File, io::AsyncReadExt};

#[derive(Serialize, Deserialize, Debug)]
pub struct Settings {
    pub address: String,
    pub heartbeat_sec: u64,
    pub output_dir: String,
}

pub async fn load_from_file(path: &str) -> Result<Settings> {
    let mut file = File::open(path).await?;

    let mut data = String::new();
    file.read_to_string(&mut data).await?;

    let json: Settings = serde_json::from_str(&data)?;
    Ok(json)
}