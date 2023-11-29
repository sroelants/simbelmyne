use std::path::Path;
use engine::Config;
use serde_yaml;
use tui::init_tui;

mod engine;
mod tui;
mod components;

#[tokio::main]
async fn main() -> anyhow::Result<()>{
    let config_path = Path::new("./match.yaml");
    let config = std::fs::read_to_string(config_path).unwrap();
    let config: Config = serde_yaml::from_str(&config).unwrap();

    init_tui(config).await.unwrap();

    Ok(())
}
