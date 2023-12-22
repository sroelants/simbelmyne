//! Assign a static score to a gven board position
//!
//! Since it's impractical to search the entire game tree till the end and see
//! who wins, we have to cut the search short at some point and assign a score
//! to the current state of the board.
//!
//! We use a fairly simplistic, but effective couple of heuristics:
//! 1. Material counting: assign scores to each type of piece, and add up the
//!    sum total of pieces for a given player.
//!
//! 2. Piece Square Tables: Because given types of pieces are more valuable
//!    in certain positions on the board (pawns should be pushed, knights should
//!    stay in the center, king should hide in the corner), we create tables
//!    that assign a score to each square for each individual piece type. 
//!
//! 3. Tapered evaluation: The value and positional preference of pieces change
//!    throughout the game (e.g., king becomes much more active in the endgame,
//!    etc). To accomodate for that, we have separate piece-square tables for 
//!    the midgame and endgame, and interpolate between them, for some measure of
//!    "midgame" and "endgame".
//!
//! The values we use here are taken directly from PeSTO.
//! Note that we're doing very little to capture more granular positional
//! information (pawn structure, king safety, hanging pieces, etc...)

mod piece_square_tables;
mod lookups;

use chess::bitboard::Bitboard;
use piece_square_tables::{MG_TABLES, EG_TABLES};
use chess::board::Board;
use chess::piece::Piece;
use chess::square::Square;
use chess::piece::PieceType;
use chess::piece::Color;

use crate::evaluate::lookups::EG_PASSED_PAWN_TABLE;
use crate::evaluate::lookups::MG_PASSED_PAWN_TABLE;
use crate::evaluate::lookups::PASSED_PAWN_MASKS;

pub type Eval = i32;

#[rustfmt::skip]
const MG_VALUES: [Eval; PieceType::COUNT] = [
    // Pawn, Knight, Bishop, Rook, Queen, King
       82,   337,    365,    477,  1025,  0
];

#[rustfmt::skip]
const EG_VALUES: [Eval; PieceType::COUNT] = [
    // Pawn, Knight, Bishop, Rook, Queen, King
       94,   281,    297,    512,  936,   0
];

/// A `Score` keeps track of the granular score breakdown
///
/// Keep track of both midgame and endgame scores for a given position, as well
/// as the "game_phase" parameter. Keeping track of all of these independently
/// means we can incrementally update the score by adding/removing pieces as the
/// game progresses.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Score {
    /// Value between 0 and 24, keeping track of how far along the game we are.
    /// A score of 0 corresponds to endgame, a score of 24 is in the opening.
    game_phase: u8,

    /// Midgame score for the board
    mg_score: Eval,

    /// Endgame score for the board
    eg_score: Eval,

    /// Midgame bonus score for passed pawns
    mg_passed_pawns: Eval,

    /// Endgame bonus score for passed pawns
    eg_passed_pawns: Eval,
}

impl Score {
    /// Values assignd to each piece type to calculate the approximate stage 
    /// of the game
    const GAME_PHASE_VALUES: [u8; PieceType::COUNT] = [0, 1, 1, 2, 4, 0];

    pub const MIN: Eval = Eval::MIN + 1;
    pub const MAX: Eval = Eval::MAX;
    pub const MATE: Eval = 20_000;
    pub const DRAW: Eval = 150;

    /// Create a new score for a board
    pub fn new(board: &Board) -> Self {
        let mut score = Self { 
            game_phase: 0, 
            mg_score: 0, 
            eg_score: 0, 
            mg_passed_pawns: 0,
            eg_passed_pawns: 0
        };

        // Walk through all the pieces on the board, and add update the Score
        // counter for each one.
        for (sq_idx, piece) in board.piece_list.into_iter().enumerate() {
            if let Some(piece) = piece {
                let square = Square::from(sq_idx);

                score.add(piece, square, board);
            }
        }

        score
    }

    /// Get the midgame weight to weight the scores with
    ///
    /// Returns a value between 0 and 24. 
    /// Large values indicate a more midgame-ish position.
    fn mg_weight(&self) -> u8 {
        self.game_phase
    }

    /// Get the endgame weight to weight the scores with
    ///
    /// Returns a value between 0 and 24. 
    /// Large values indicate a more endgame-ish position.
    fn eg_weight(&self) -> u8 {
        24 - self.game_phase
    }

    /// Return the total (weighted) score for the position
    pub fn total(&self, side: Color) -> Eval {
        let mg = (self.mg_score + self.mg_passed_pawns) / 24;
        let eg = (self.eg_score + self.eg_passed_pawns) / 24;

        let score = (mg * self.mg_weight() as Eval) 
            + (eg * self.eg_weight() as Eval);

        if side.is_white() { score } else { -score }
    }

    /// Update the score by adding a piece to it
    pub fn add(&mut self, piece: Piece, sq: Square, board: &Board) {
        self.mg_score += piece.mg_score(sq);
        self.eg_score += piece.eg_score(sq);

        if piece.is_pawn() {
            self.update_passed_pawns(board);
        }

        self.game_phase += piece.game_phase();
    }

    /// Update the score by removing a piece from it
    pub fn remove(&mut self, piece: Piece, sq: Square, board: &Board) {
        self.mg_score -= piece.mg_score(sq);
        self.eg_score -= piece.eg_score(sq);

        if piece.is_pawn() {
            self.update_passed_pawns(board);
        }

        self.game_phase -= piece.game_phase();
    }

    pub fn update_passed_pawns(&mut self, board: &Board) {
        use Color::*;
        let white_pawns = board.pawns(White);
        let black_pawns = board.pawns(Black);

        // Clear the previous passed-pawn scores
        self.mg_passed_pawns = 0;
        self.eg_passed_pawns = 0;

        for sq in white_pawns {
            let mask = PASSED_PAWN_MASKS[White as usize][sq as usize];

            if black_pawns & mask == Bitboard::EMPTY {
                self.mg_passed_pawns += MG_PASSED_PAWN_TABLE[sq.flip() as usize];
                self.eg_passed_pawns += EG_PASSED_PAWN_TABLE[sq.flip() as usize];
            }
        }

        for sq in black_pawns {
            let mask = PASSED_PAWN_MASKS[Black as usize][sq as usize];

            if white_pawns & mask == Bitboard::EMPTY {
                self.mg_passed_pawns -= MG_PASSED_PAWN_TABLE[sq as usize];
                self.eg_passed_pawns -= EG_PASSED_PAWN_TABLE[sq as usize];
            }
        }
    }
}

trait Scorable {
    /// Get the midgame score for a piece
    /// We always score from White's perspective (i.e., white pieces are 
    /// positive, black pieces are negative) and negate the total score in
    /// case we're interested in Black's score
    fn mg_score(&self, sq: Square) -> Eval;

    /// Get the endgame score for a piece
    /// We always score from White's perspective (i.e., white pieces are 
    /// positive, black pieces are negative) and negate the total score in
    /// case we're interested in Black's score
    fn eg_score(&self, sq: Square) -> Eval;

    /// The amount by which this piece contributes to the game phase
    fn game_phase(&self) -> u8;
}

impl Scorable for Piece {
    fn mg_score(&self, sq: Square) -> Eval {
        let pcolor = self.color();
        let ptype = self.piece_type() as usize;

        // Calculate the piece's scores
        if pcolor.is_white() {
            MG_VALUES[ptype] + MG_TABLES[ptype][sq.flip() as usize]
        } else {
            -(MG_VALUES[ptype] + MG_TABLES[ptype][sq as usize])
        }
    }

    fn eg_score(&self, sq: Square) -> Eval {
        let pcolor = self.color();
        let ptype = self.piece_type() as usize;

        // Calculate the piece's scores
        if pcolor.is_white() {
            EG_VALUES[ptype] + EG_TABLES[ptype][sq.flip() as usize]
        } else {
            -(EG_VALUES[ptype] + EG_TABLES[ptype][sq as usize])
        }
    }

    fn game_phase(&self) -> u8 {
        Score::GAME_PHASE_VALUES[self.piece_type() as usize]
    }
}

#[cfg(test)]
mod tests {
    use chess::board::Board;

    use crate::{tests::TEST_POSITIONS, position::Position};

    #[test]
    fn eval_symmetry() {
        for fen in TEST_POSITIONS {
            println!("Testing symmetry for {fen}");
            let board: Board = fen.parse().unwrap();
            let side = board.current;
            let position = Position::new(board);
            let mirrored_pos = Position::new(board.mirror());

            assert_eq!(position.score.total(side), mirrored_pos.score.total(side.opp()));
        }
    }
}
