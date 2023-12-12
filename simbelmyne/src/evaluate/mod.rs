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

use piece_square_tables::{MG_TABLES, EG_TABLES};
use chess::board::Board;
use chess::piece::Piece;
use chess::square::Square;
use chess::piece::PieceType;
use chess::piece::Color;

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
}

impl Score {
    /// Values assignd to each piece type to calculate the approximate stage 
    /// of the game
    const GAME_PHASE_VALUES: [u8; PieceType::COUNT] = [0, 1, 1, 2, 4, 0];

    pub const MIN: Eval = Eval::MIN + 1;
    pub const MAX: Eval = Eval::MAX;
    pub const MATE: Eval = 20_000;
    pub const DRAW: Eval = 0;

    /// Create a new score for a board
    pub fn new(board: &Board) -> Self {
        let us = board.current;
        let mut score = Self { game_phase: 0, mg_score: 0, eg_score: 0 };

        // Walk through all the pieces on the board, and add update the Score
        // counter for each one.
        for (sq_idx, piece) in board.piece_list.into_iter().enumerate() {
            if let Some(piece) = piece {
                let square = Square::from(sq_idx);

                score.add(us, piece, square);
            }
        }

        score
    }

    /// Convert the individual scores to the opponent's POV.
    pub fn flipped(&self) -> Self {
        Self {
            game_phase: self.game_phase,
            mg_score: -self.mg_score,
            eg_score: -self.eg_score
        }
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
    pub fn total(&self) -> Eval {
        (self.mg_score * self.mg_weight() as Eval
            + self.eg_score * self.eg_weight() as Eval) / 24
    }

    /// Update the score by adding a piece to it
    pub fn add(&mut self, us: Color, piece: Piece, sq: Square) {
        let color = piece.color();

        // The PSTs aren't symmetric, so mirroring the square effectively
        // mirrors the table
        let sq = if color.is_white() { sq } else { sq.flip() };

        let sq_idx = sq as usize;
        let ptype_idx = piece.piece_type() as usize;

        // Calculate the piece's scores
        let mg_score = MG_VALUES[ptype_idx] + MG_TABLES[ptype_idx][sq_idx];
        let eg_score = EG_VALUES[ptype_idx] + EG_TABLES[ptype_idx][sq_idx];

        // Update the scores by either adding or removing the score, depending
        // on the color of the player.
        if us == color {
            self.mg_score += mg_score;
            self.eg_score += eg_score;
        } else {
            self.mg_score -= mg_score; 
            self.eg_score -= eg_score;
        }

        // Update the game phase
        self.game_phase += Self::GAME_PHASE_VALUES[ptype_idx];
    }

    /// Update the score by removing a piece from it
    pub fn remove(&mut self, us: Color, piece: Piece, sq: Square) {
        let color = piece.color();

        // The PSTs aren't symmetric, so flipping the square effectively
        // flips the table
        let sq = if color.is_white() { sq } else { sq.flip() };

        let ptype_idx = piece.piece_type() as usize;
        let sq_idx = sq as usize;

        // Calculate the piece's scores
        let mg_score = MG_VALUES[ptype_idx] + MG_TABLES[ptype_idx][sq_idx];
        let eg_score = EG_VALUES[ptype_idx] + EG_TABLES[ptype_idx][sq_idx];

        // Update the scores by either adding or removing the score, depending
        // on the color of the player.
        if us == color {
            self.mg_score -= mg_score;
            self.eg_score -= eg_score;
        } else {
            self.mg_score += mg_score; 
            self.eg_score += eg_score;
        }

        // Update the game phase
        self.game_phase -= Self::GAME_PHASE_VALUES[ptype_idx];
    }
}
