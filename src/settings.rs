use anyhow::Result;
use serde::{Serialize, Deserialize};
use tokio::{fs::File, io::AsyncReadExt};

#[derive(Serialize, Deserialize, Debug)]
pub struct Settings {
    pub ip: String,
    pub port: u16,
}

pub async fn load_from_file(path: &str) -> Result<Settings> {
    let mut file = File::open(path).await?;

    let mut data = String::new();
    file.read_to_string(&mut data).await?;

    let json: Settings = serde_json::from_str(&data)?;
    Ok(json)
}