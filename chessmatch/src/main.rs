use std::path::Path;
use chess::board::Board;
use chess::piece::Color;
use engine::{Config, Engine, LabeledMessage};
use serde_yaml;
use uci::{UciClientMessage, UciEngineMessage};

mod uci;
mod engine;

fn main() -> anyhow::Result<()>{
    let board = Board::new();

    let config_path = Path::new("./match.yaml");
    let config = std::fs::read_to_string(config_path).unwrap();
    let config: Config = serde_yaml::from_str(&config).unwrap();

    let (tx, rx) = crossbeam::channel::unbounded::<LabeledMessage>();

    let mut white = Engine::new(Color::White, tx.clone(), &config.white);
    let mut black = Engine::new(Color::Black, tx.clone(), &config.black);

    let mut engines: [Engine; Color::COUNT] = [white, black];

    for (player, msg) in rx {
        use UciClientMessage::*;
        use UciEngineMessage::*;

        // match msg {
        //     _ => todo!()
        // }
    }

    Ok(())
}
