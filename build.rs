use std::env;
use std::fs;
use std::path::Path;

fn copy_settings() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")?;
    let manifest_dir = Path::new(&manifest_dir);

    let out_dir = env::var("OUT_DIR")?;
    let out_dir = Path::new(&out_dir);

    let settings_src = manifest_dir.join("settings.json");
    let settings_dst = out_dir.join("settings.json");
    fs::copy(settings_src, settings_dst)?;

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    copy_settings()?;

    tonic_prost_build::configure()
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .build_server(true)
        .build_client(false)
        .compile_protos(&["proto/controller.proto"], &["proto"])?;
    Ok(())
}
