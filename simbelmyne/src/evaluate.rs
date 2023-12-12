use crate::square_piece_tables::{MIDGAME_TABLES, ENDGAME_TABLES};
use chess::board::Board;
use chess::piece::Piece;
use chess::square::Square;
use chess::piece::PieceType;
use chess::piece::Color;

#[rustfmt::skip]
const MIDGAME_VALUES: [Eval; PieceType::COUNT] = [
    // Pawn, Knight, Bishop, Rook, Queen, King
       82,   337,    365,    477,  1025,  0
];

#[rustfmt::skip]
const ENDGAME_VALUES: [Eval; PieceType::COUNT] = [
    // Pawn, Knight, Bishop, Rook, Queen, King
       94,   281,    297,    512,  936,   0
];

pub type Eval = i32;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Score {
    game_phase: Eval,
    mg_score: Eval,
    eg_score: Eval,
}

impl Score {
    const GAME_PHASE_VALUES: [Eval; 6] = [0, 1, 1, 2, 4, 0];
    pub const MIN: Eval = Eval::MIN + 1;
    pub const MAX: Eval = Eval::MAX;

    pub fn new(board: &Board) -> Self {
        let mut score = Self::default();
        let us = board.current;

        for (sq_idx, piece) in board.piece_list.into_iter().enumerate() {
            if let Some(piece) = piece {
                let sq: Square = sq_idx.into();

                score.add(us, piece, sq);
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

    pub fn flipped(&self) -> Self {
        Self {
            game_phase: self.game_phase,
            mg_score: -self.mg_score,
            eg_score: -self.mg_score
        }
    }

    pub fn mg_weight(&self) -> Eval {
        self.game_phase
    }

    pub fn eg_weight(&self) -> Eval {
        24 - self.game_phase
    }

    pub fn total(&self) -> Eval {
        (self.mg_score * self.mg_weight() + self.eg_score * self.eg_weight()) / 24
    }

    //TODO: Tweak this signature to take a Piece instead of piecetype and color
    pub fn add(&mut self, us: Color, piece: Piece, sq: Square) {
        let color = piece.color();
        let ptype_idx = piece.piece_type() as usize;
        let sq = if color.is_white() { sq } else { sq.flip() };

        let sq_idx = sq as usize;

        self.game_phase += Self::GAME_PHASE_VALUES[ptype_idx];

        if us == color {
            self.mg_score += MIDGAME_VALUES[ptype_idx]
                + MIDGAME_TABLES[ptype_idx][sq_idx];

            self.eg_score += ENDGAME_VALUES[ptype_idx]
                + ENDGAME_TABLES[ptype_idx][sq_idx];
        } else {
            self.mg_score -= MIDGAME_VALUES[ptype_idx]
                + MIDGAME_TABLES[ptype_idx][sq_idx];

            self.eg_score -= ENDGAME_VALUES[ptype_idx]
                + ENDGAME_TABLES[ptype_idx][sq_idx];
        }
    }

    //TODO: Tweak this signature to take a Piece instead of piecetype and color
    pub fn remove(&mut self, us: Color, piece: Piece, sq: Square) {
        let color = piece.color();
        let sq = if color.is_white() { sq } else { sq.flip() };
        let ptype_idx = piece.piece_type() as usize;
        let sq_idx = sq as usize;

        self.game_phase -= Self::GAME_PHASE_VALUES[ptype_idx];

        if us == color {
            self.mg_score -= MIDGAME_VALUES[ptype_idx]
                + MIDGAME_TABLES[ptype_idx][sq_idx];

            self.eg_score -= ENDGAME_VALUES[ptype_idx]
                + ENDGAME_TABLES[ptype_idx][sq_idx];
        } else {
            self.mg_score += MIDGAME_VALUES[ptype_idx]
                + MIDGAME_TABLES[ptype_idx][sq_idx];

            self.eg_score += ENDGAME_VALUES[ptype_idx]
                + ENDGAME_TABLES[ptype_idx][sq_idx];
        }
    }
}

impl From<Board> for Score {
    fn from(value: Board) -> Self {
        Score::new(&value)
    }
}
