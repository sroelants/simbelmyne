pub mod uci;

use chess::{board::Board, movegen::moves::Move};
use rand::Rng;

pub struct Engine {
    board: Board,
    debug: bool,

}

impl Engine {
    pub fn new() -> Engine {
        Engine { 
            board: Board::new(),
            debug: false,
        }
    }

    pub fn with_board(fen: String) -> anyhow::Result<Engine> {
        let board = fen.parse()?;
        Ok(Engine {  board , debug: false })
    }

    fn next_move(&self) -> Move {
        let moves = self.board.legal_moves();
        let rand_idx = rand::thread_rng().gen_range(0..moves.len());

        moves[rand_idx]
    }

    fn play_move(&mut self, mv: Move) -> anyhow::Result<()> {
        self.board = self.board.play_move(mv);
        Ok(())
    }
}

