use std::path::Path;
use chess::board::Board;
use engine::{Config, Engine};
use serde_yaml;
use tokio::select;
use uci::UciEngineMessage;

mod uci;
mod engine;

#[tokio::main]
async fn main() -> anyhow::Result<()>{
    let config_path = Path::new("./match.yaml");
    let config = std::fs::read_to_string(config_path).unwrap();
    let config: Config = serde_yaml::from_str(&config).unwrap();

    let mut white = Engine::new(&config.white);
    white.init().await;

    let mut black = Engine::new(&config.black);
    black.init().await;

    let mut board = Board::new();

    white.set_pos(board).await;
    white.go().await;


    loop {
        select! {
            msg = white.read() => {
                if let UciEngineMessage::BestMove(mv) = msg {
                    board = board.play_move(mv);
                    black.set_pos(board).await;
                    black.go().await;
                }
            },

            msg = black.read() => {
                if let UciEngineMessage::BestMove(mv) = msg {
                    board = board.play_move(mv);
                    white.set_pos(board).await;
                    white.go().await;
                }
            },
        }

        println!("New board:\n{board}");
    }
}
