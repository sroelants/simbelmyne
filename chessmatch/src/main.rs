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
    let mut msg: Option<UciEngineMessage> = None;
    let mut next_player = if board.current.is_white() {
        &mut white
    } else {
        &mut black
    };

    next_player.set_pos(board).await;
    next_player.go().await;

    loop {
        select! {
            white_msg = (&mut white).read() => {
                msg = Some(white_msg);
            },

            black_msg = (&mut black).read() => {
                msg = Some(black_msg);
            },
        }

        next_player = if board.current.is_white() {
            &mut white
        } else {
            &mut black
        };

        if let Some(UciEngineMessage::BestMove(mv)) = msg {
            board = board.play_move(mv);
            next_player.set_pos(board).await;
            next_player.go().await;

            println!("New board:\n{board}");
        }
    }
}
