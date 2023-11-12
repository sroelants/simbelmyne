use crate::{search::Score, square_piece_tables::{MIDGAME_TABLES, ENDGAME_TABLES}};
use chess::board::{PieceType, Board, Square};

#[rustfmt::skip]
const MIDGAME_VALUES: [i32; PieceType::COUNT] = [
    // Pawn, Knight, Bishop, Rook, Queen, King
       82,   337,    365,    477,  1025,  10000
];

#[rustfmt::skip]
const ENDGAME_VALUES: [i32; PieceType::COUNT] = [
    // Pawn, Knight, Bishop, Rook, Queen, King
       94,   281,    297,    512,  936,   10000
];


pub trait Eval {
    fn eval(&self) -> Score;

    fn eval_material(&self) -> Score;

    fn eval_position(&self) -> Score;
}

impl Eval for Board {
    /// Return a static evaluation for the given board
    fn eval(&self) -> Score {
        self.eval_material()
        + self.eval_position()
    }

    fn eval_material(&self) -> Score {
        let us = self.current;
        let them = us.opp();

        let mut mg_scores: [i32; 2] = [0, 0];
        let mut eg_scores: [i32; 2] = [0, 0];
        let mut game_phase = 0;

        for piece in self.piece_list.into_iter().flatten() {
            let color = piece.color();
            let ptype = piece.piece_type();

            mg_scores[color as usize] += MIDGAME_VALUES[ptype as usize];
            eg_scores[color as usize] += ENDGAME_VALUES[ptype as usize];
            game_phase += ptype.get_increment();
        }

        let mg_weight = game_phase;
        let eg_weight = 24 - game_phase;

        let mg_score = mg_scores[us as usize] - mg_scores[them as usize];
        let eg_score = eg_scores[us as usize] - eg_scores[them as usize];

        (mg_score * mg_weight + eg_score * eg_weight) / 24
    }

    /// Calculate the positional score for the board using PeSTO's eval function
    /// See more: https://www.chessprogramming.org/PeSTO%27s_Evaluation_Function
    fn eval_position(&self) -> Score {
        let us = self.current;
        let them = us.opp();

        let mut mg_scores: [i32; 2] = [0, 0];
        let mut eg_scores: [i32; 2] = [0, 0];
        let mut game_phase = 0;

        for (idx, piece) in self.piece_list.iter().enumerate() {
            if let Some(piece) = piece {
                let color = piece.color();
                let ptype = piece.piece_type();

                let mut sq: Square = idx.into();
                if color.is_black() { sq = sq.flip() };

                mg_scores[color as usize] += MIDGAME_TABLES[ptype as usize][sq as usize];
                eg_scores[color as usize] += ENDGAME_TABLES[ptype as usize][sq as usize];

                game_phase += ptype.get_increment();
            }
        }

        let mg_weight = game_phase;
        let eg_weight = 24 - game_phase;

        let mg_score = mg_scores[us as usize] - mg_scores[them as usize];
        let eg_score = eg_scores[us as usize] - eg_scores[them as usize];

        (mg_score * mg_weight + eg_score * eg_weight) / 24
    }
}

trait GamePhaseIncrement {
    const GAME_PHASE_INCREMENTS: [i32; 6] = [0, 1, 1, 2, 4, 0];

    fn get_increment(&self) -> i32;
}

impl GamePhaseIncrement for PieceType {
    fn get_increment(&self) -> i32 {
        Self::GAME_PHASE_INCREMENTS[*self as usize]
    }
}

