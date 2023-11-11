use crate::{search::Score, square_piece_tables::{MIDGAME_TABLES, ENDGAME_TABLES}};
use chess::board::{PieceType, Board, Color, Square};

#[rustfmt::skip]
const MIDGAME_VALUES: [[i32; PieceType::COUNT]; Color::COUNT] = [
    // Pawn, Knight, Bishop, Rook, Queen, King
    [  82,   337,    365,    477,  1025,  10000], // White
    [ -82,  -337,   -365,   -477, -1025, -10000], // Black
];

#[rustfmt::skip]
const ENDGAME_VALUES: [[i32; PieceType::COUNT]; Color::COUNT] = [
    // Pawn, Knight, Bishop, Rook, Queen, King
    [  94,   281,    297,    512,  936,   10000], // White
    [ -94,  -281,   -297,   -512, -936,  -10000]  // Black
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
        let mut midgame_score = 0;
        let mut endgame_score = 0;
        let mut game_phase = 0;

        for piece in self.piece_list.into_iter().flatten() {
            midgame_score += MIDGAME_VALUES[piece.piece_type() as usize][piece.color() as usize];
            endgame_score += ENDGAME_VALUES[piece.piece_type() as usize][piece.color() as usize];
            game_phase += piece.piece_type().get_increment();
        }

        let midgame_weight = game_phase;
        let endgame_weight = 24 - game_phase;

        (midgame_score * midgame_weight + endgame_score * endgame_weight) / 24
    }

    /// Calculate the positional score for the board using PeSTO's eval function
    /// See more: https://www.chessprogramming.org/PeSTO%27s_Evaluation_Function
    fn eval_position(&self) -> Score {
        // 1. Grab the tables
        // 2. Iterate over all the pieces
        // 3. Grab the value for the piece in the table
        // 4. Add them all up
        // 5. Weight with the game_phase weight

        let mut midgame_score = 0;
        let mut endgame_score = 0;
        let mut game_phase = 0;

        for (idx, piece) in self.piece_list.iter().enumerate() {
            if let Some(piece) = piece {
                let sq: Square = idx.into(); //TODO: Flip this for black pieces
                
                if piece.color().is_white() {
                    midgame_score += MIDGAME_TABLES[piece.piece_type() as usize][sq as usize];
                    endgame_score += ENDGAME_TABLES[piece.piece_type() as usize][sq as usize];
                } else {
                    let sq = sq.flip();
                    midgame_score -= MIDGAME_TABLES[piece.piece_type() as usize][sq as usize];
                    endgame_score -= ENDGAME_TABLES[piece.piece_type() as usize][sq as usize];
                }

                game_phase += piece.piece_type().get_increment();
            }
        }

        let midgame_weight = game_phase;
        let endgame_weight = 24 - game_phase;

        (midgame_score * midgame_weight + endgame_score * endgame_weight) / 24
    }
}

trait GamePhaseIncrement {
    const GAME_PHASE_INCREMENTS: [i32; 6] = [0, 1, 2 ,3 ,4 ,0];

    fn get_increment(&self) -> i32;
}

impl GamePhaseIncrement for PieceType {
    fn get_increment(&self) -> i32 {
        Self::GAME_PHASE_INCREMENTS[*self as usize]
    }
}

