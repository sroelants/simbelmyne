use crate::square_piece_tables::{MIDGAME_TABLES, ENDGAME_TABLES};
use chess::board::{PieceType, Board, Square, Color};

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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Score {
    game_phase: i32,
    mg_score: i32,
    eg_score: i32,
}

impl Score {
    const GAME_PHASE_VALUES: [i32; 6] = [0, 1, 1, 2, 4, 0];
    pub const MIN: i32 = i32::MIN;
    pub const MAX: i32 = i32::MAX;

    pub fn new(board: &Board) -> Self {
        let mut score = Self::default();

        for (idx, piece) in board.piece_list.iter().enumerate() {
            if let Some(piece) = piece {
                let color = piece.color();
                let ptype = piece.piece_type();
                let mut sq: Square = idx.into();

                score.add(ptype, color, sq);
            }
        }

        score
    }

    pub fn default() -> Self {
        Self {
            game_phase: 0,
            mg_score: 0,
            eg_score: 0,
        }
    }


    pub fn mg_weight(&self) -> i32 {
        self.game_phase
    }

    pub fn eg_weight(&self) -> i32 {
        24 - self.game_phase
    }

    pub fn score(&self) -> i32 {
        (self.mg_score * self.mg_weight() + self.eg_score * self.eg_weight()) / 24
    }

    pub fn add(&self, ptype: PieceType, color: Color, sq: Square) -> Self {
        let game_phase = self.game_phase + Self::GAME_PHASE_VALUES[ptype as usize];

        let mg_score = self.mg_score 
            + MIDGAME_VALUES[ptype as usize]
            + MIDGAME_TABLES[ptype as usize][sq as usize];

        let eg_score = self.eg_score 
            + ENDGAME_VALUES[ptype as usize]
            + ENDGAME_TABLES[ptype as usize][sq as usize];

        Score { game_phase, mg_score, eg_score }
    }

    pub fn remove(&self, ptype: PieceType, color: Color, sq: Square) -> Self {
        let game_phase = self.game_phase - Self::GAME_PHASE_VALUES[ptype as usize];
        let sq = if color.is_black() { sq } else { sq.flip() };

        let mg_score = self.mg_score 
            - MIDGAME_VALUES[ptype as usize]
            - MIDGAME_TABLES[ptype as usize][sq as usize];

        let eg_score = self.eg_score 
            - ENDGAME_VALUES[ptype as usize]
            - ENDGAME_TABLES[ptype as usize][sq as usize];

        Score { game_phase, mg_score, eg_score }
    }
}
