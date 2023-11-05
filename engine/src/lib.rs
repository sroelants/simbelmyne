use chess::{board::Board, movegen::moves::Move};
use rand::Rng;

pub struct Engine {
    pub board: Board,
}

impl Engine {
    pub fn new() -> Engine {
        Engine { board: Board::new() }
    }

    pub fn with_board(fen: String) -> anyhow::Result<Engine> {
        let board = fen.parse()?;
        Ok(Engine {  board })
    }

    pub fn next_move(&self) -> Move {
        let moves = self.board.legal_moves();
        let rand_idx = rand::thread_rng().gen_range(0..moves.len());

        moves[rand_idx]
    }

    pub fn play_move(&mut self, mv: Move) -> anyhow::Result<()> {
        self.board = self.board.play_move(mv);
        Ok(())
    }
}
