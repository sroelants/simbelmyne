use chess::{board::Board, movegen::moves::Move};
use crate::evaluate::Eval;

pub type Score = i32;

pub(crate) struct BoardState {
    pub(crate) board: Board,
    pub(crate) score: Score
}

impl BoardState {
    pub fn new(board: Board) -> BoardState {
        BoardState { 
            board, 
            score: board.eval(),
        }
    }

    pub fn best_move(&self, depth: usize) -> Move {
        let mut best_move: Move = Move::NULL;
        let mut best_score: Score = Score::MIN/2;

        for mv in self.board.legal_moves() {
            let new_search = Self::new(self.board.play_move(mv));
            let score = new_search.negamax(depth - 1);

            if score > best_score {
                best_score = score;
                best_move = mv;
            }
        }

        println!("Found best move {best_move} with score {best_score}");
        best_move
    }

    fn negamax(&self, depth: usize) -> Score {
        let mut best_score = i32::MIN / 2;

        if depth == 0 {
            return self.board.eval()
        }

        for mv in self.board.legal_moves() {
            let new_search = Self::new(self.board.play_move(mv));
            let score = -new_search.negamax(depth - 1);

            if score > best_score {
                best_score = score;
            }
        }

        best_score
    }
}

