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
pub mod tuner;
mod params;

use chess::bitboard::Bitboard;
use piece_square_tables::{MG_TABLES, EG_TABLES};
use chess::board::Board;
use chess::piece::Piece;
use chess::square::Square;
use chess::piece::PieceType;
use chess::piece::Color;

use crate::evaluate::lookups::FILES;
use crate::evaluate::lookups::DOUBLED_PAWN_MASKS;
use crate::evaluate::lookups::ISOLATED_PAWN_MASKS;
use crate::evaluate::lookups::PASSED_PAWN_MASKS;
use crate::evaluate::params::EG_BISHOP_MOBILITY_BONUS;
use crate::evaluate::params::EG_BISHOP_PAIR_BONUS;
use crate::evaluate::params::EG_DOUBLED_PAWN_PENALTY;
use crate::evaluate::params::EG_ISOLATED_PAWN_PENALTY;
use crate::evaluate::params::EG_KNIGHT_MOBILITY_BONUS;
use crate::evaluate::params::EG_QUEEN_MOBILITY_BONUS;
use crate::evaluate::params::EG_ROOK_MOBILITY_BONUS;
use crate::evaluate::params::EG_ROOK_OPEN_FILE_BONUS;
use crate::evaluate::params::MG_BISHOP_MOBILITY_BONUS;
use crate::evaluate::params::MG_BISHOP_PAIR_BONUS;
use crate::evaluate::params::MG_DOUBLED_PAWN_PENALTY;
use crate::evaluate::params::MG_ISOLATED_PAWN_PENALTY;
use crate::evaluate::params::MG_KNIGHT_MOBILITY_BONUS;
use crate::evaluate::params::MG_PASSED_PAWN_TABLE;
use crate::evaluate::params::EG_PASSED_PAWN_TABLE;
use crate::evaluate::params::MG_QUEEN_MOBILITY_BONUS;
use crate::evaluate::params::MG_ROOK_MOBILITY_BONUS;
use crate::evaluate::params::MG_ROOK_OPEN_FILE_BONUS;
use crate::evaluate::params::EG_PIECE_VALUES;
use crate::evaluate::params::MG_PIECE_VALUES;
use crate::search::params::MAX_DEPTH;


pub type Eval = i32;

////////////////////////////////////////////////////////////////////////////////
//
// Evaluation logic
//
////////////////////////////////////////////////////////////////////////////////

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

    /// Midgame penalty for isolated pawns
    mg_isolated_pawns: Eval,

    /// Endgame penalty for isolated pawns
    eg_isolated_pawns: Eval,

    /// Midgame penalty for doubled pawns
    mg_doubled_pawns: Eval,

    /// Endgame penalty for doubled pawns
    eg_doubled_pawns: Eval,

    /// Midgame bonus for bishop pair
    mg_bishop_pair: Eval,

    /// Endgame bonus for bishop pair
    eg_bishop_pair: Eval,

    /// Midgame bonus for a rook on an open file
    mg_rook_open_file: Eval,

    /// Endgame bonus for a rook on an open file
    eg_rook_open_file: Eval,

    /// Midgame mobility score
    mg_mobility_score: Eval,

    /// Endgame mobility score
    eg_mobility_score: Eval,
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
        let mut score = Self { 
            game_phase: 0, 
            mg_score: 0, 
            eg_score: 0, 
            mg_passed_pawns: 0,
            eg_passed_pawns: 0,
            mg_isolated_pawns: 0,
            eg_isolated_pawns: 0,
            mg_doubled_pawns: 0,
            eg_doubled_pawns: 0,
            mg_bishop_pair: 0,
            eg_bishop_pair: 0,
            mg_rook_open_file: 0,
            eg_rook_open_file: 0,
            mg_mobility_score: 0,
            eg_mobility_score: 0,
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
        let mg_total = self.mg_score 
            + self.mg_passed_pawns 
            + self.mg_isolated_pawns 
            + self.mg_doubled_pawns
            + self.mg_bishop_pair
            + self.mg_rook_open_file
            + self.mg_mobility_score;

        let eg_total = self.eg_score 
            + self.eg_passed_pawns 
            + self.eg_isolated_pawns 
            + self.eg_doubled_pawns
            + self.eg_bishop_pair
            + self.eg_rook_open_file
            + self.eg_mobility_score;

        let score = mg_total * self.mg_weight() as Eval / 24
            + eg_total * self.eg_weight() as Eval / 24;

        if side.is_white() { score } else { -score }
    }

    /// Update the score by adding a piece to it
    pub fn add(&mut self, piece: Piece, sq: Square, board: &Board) {
        self.mg_score += piece.mg_score(sq);
        self.eg_score += piece.eg_score(sq);

        if piece.is_pawn() {
            self.update_passed_pawns(board);
            self.update_isolated_pawns(board);
            self.update_doubled_pawns(board);
        }

        if piece.is_bishop() {
            self.update_bishop_pair(board);
        }

        if piece.is_rook() {
            self.update_rook_open_file(board);
        }

        self.update_mobility(board);

        self.game_phase += piece.game_phase();
    }

    /// Update the score by removing a piece from it
    pub fn remove(&mut self, piece: Piece, sq: Square, board: &Board) {
        self.mg_score -= piece.mg_score(sq);
        self.eg_score -= piece.eg_score(sq);

        if piece.is_pawn() {
            self.update_passed_pawns(board);
            self.update_isolated_pawns(board);
            self.update_doubled_pawns(board);
        }

        if piece.is_bishop() {
            self.update_bishop_pair(board);
        }

        // if piece.is_rook() {
        //     self.update_rook_open_file(board);
        // }

        self.update_mobility(board);

        self.game_phase -= piece.game_phase();
    }

    fn update_passed_pawns(&mut self, board: &Board) {
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

    fn update_isolated_pawns(&mut self, board: &Board) {
        use Color::*;
        let white_pawns = board.pawns(White);
        let black_pawns = board.pawns(Black);

        // Clear the previous passed-pawn scores
        self.mg_isolated_pawns = 0;
        self.eg_isolated_pawns = 0;

        for sq in white_pawns {
            let mask = ISOLATED_PAWN_MASKS[sq as usize];

            if white_pawns & mask == Bitboard::EMPTY {
                self.mg_isolated_pawns += MG_ISOLATED_PAWN_PENALTY;
                self.eg_isolated_pawns += EG_ISOLATED_PAWN_PENALTY;
            }
        }

        for sq in black_pawns {
            let mask = ISOLATED_PAWN_MASKS[sq as usize];

            if black_pawns & mask == Bitboard::EMPTY {
                self.mg_isolated_pawns -= MG_ISOLATED_PAWN_PENALTY;
                self.eg_isolated_pawns -= EG_ISOLATED_PAWN_PENALTY;
            }
        }
    }

    fn update_doubled_pawns(&mut self, board: &Board) {
        use Color::*;
        let white_pawns = board.pawns(White);
        let black_pawns = board.pawns(Black);

        // Clear the previous passed-pawn scores
        self.mg_doubled_pawns = 0;
        self.eg_doubled_pawns = 0;

        for mask in DOUBLED_PAWN_MASKS {
            let doubled_white = (white_pawns & mask).count().saturating_sub(1) as i32;
            self.mg_doubled_pawns += doubled_white * MG_DOUBLED_PAWN_PENALTY;
            self.eg_doubled_pawns += doubled_white * EG_DOUBLED_PAWN_PENALTY;

            let doubled_black = (black_pawns & mask).count().saturating_sub(1) as i32;
            self.mg_doubled_pawns -= doubled_black * MG_DOUBLED_PAWN_PENALTY;
            self.eg_doubled_pawns -= doubled_black * EG_DOUBLED_PAWN_PENALTY;
        }
    }

    fn update_bishop_pair(&mut self, board: &Board) {
        use Color::*;
        self.mg_bishop_pair = 0;
        self.eg_bishop_pair = 0;

        if board.bishops(White).count() == 2 {
            self.mg_bishop_pair += MG_BISHOP_PAIR_BONUS;
            self.eg_bishop_pair += EG_BISHOP_PAIR_BONUS;
        }

        if board.bishops(Black).count() == 2 {
            self.mg_bishop_pair -= MG_BISHOP_PAIR_BONUS;
            self.eg_bishop_pair -= EG_BISHOP_PAIR_BONUS;
        }
    }

    fn update_rook_open_file(&mut self, board: &Board) {
        use Color::*;
        use PieceType::*;
        let pawns = board.piece_bbs[Pawn as usize];

        self.mg_rook_open_file = 0;
        self.eg_rook_open_file = 0;

        for sq in board.rooks(White) {
            if (FILES[sq as usize] & pawns).is_empty() {
                self.mg_rook_open_file += MG_ROOK_OPEN_FILE_BONUS;
                self.eg_rook_open_file += EG_ROOK_OPEN_FILE_BONUS;
            }
        }

        for sq in board.rooks(Black) {
            if (FILES[sq as usize] & pawns).is_empty() {
                self.mg_rook_open_file -= MG_ROOK_OPEN_FILE_BONUS;
                self.eg_rook_open_file -= EG_ROOK_OPEN_FILE_BONUS;
            }
        }
    }

    fn update_mobility(&mut self, board: &Board) {
        use Color::*;

        self.mg_mobility_score = 0;
        self.eg_mobility_score = 0;
        let blockers = board.all_occupied();
        let white_pieces = board.occupied_by(White);
        let black_pieces = board.occupied_by(Black);

        for sq in board.knights(White) {
            let available_squares = sq.knight_squares() 
                & !white_pieces;

            let sq_count = available_squares.count();
            self.mg_mobility_score += MG_KNIGHT_MOBILITY_BONUS[sq_count as usize];
            self.eg_mobility_score += EG_KNIGHT_MOBILITY_BONUS[sq_count as usize];
        }

        for sq in board.knights(Black) {
            let available_squares = sq.knight_squares() 
                & !black_pieces;

            let sq_count = available_squares.count();

            self.mg_mobility_score -= MG_KNIGHT_MOBILITY_BONUS[sq_count as usize];
            self.eg_mobility_score -= EG_KNIGHT_MOBILITY_BONUS[sq_count as usize];
        }

        for sq in board.bishops(White) {
            let available_squares = sq.bishop_squares(blockers) 
                & !white_pieces;

            let sq_count = available_squares.count();
            self.mg_mobility_score += MG_BISHOP_MOBILITY_BONUS[sq_count as usize];
            self.eg_mobility_score += EG_BISHOP_MOBILITY_BONUS[sq_count as usize];
        }

        for sq in board.bishops(Black) {
            let available_squares = sq.bishop_squares(blockers) 
                & !black_pieces;

            let sq_count = available_squares.count();
            self.mg_mobility_score -= MG_BISHOP_MOBILITY_BONUS[sq_count as usize];
            self.eg_mobility_score -= EG_BISHOP_MOBILITY_BONUS[sq_count as usize];
        }

        for sq in board.rooks(White) {
            let available_squares = sq.rook_squares(blockers) 
                & !white_pieces;

            let sq_count = available_squares.count();
            self.mg_mobility_score += MG_ROOK_MOBILITY_BONUS[sq_count as usize];
            self.eg_mobility_score += EG_ROOK_MOBILITY_BONUS[sq_count as usize];
        }

        for sq in board.rooks(Black) {
            let available_squares = sq.rook_squares(blockers) 
                & !black_pieces;

            let sq_count = available_squares.count();
            self.mg_mobility_score -= MG_ROOK_MOBILITY_BONUS[sq_count as usize];
            self.eg_mobility_score -= EG_ROOK_MOBILITY_BONUS[sq_count as usize];
        }

        for sq in board.queens(White) {
            let available_squares = sq.queen_squares(blockers) 
                & !white_pieces;

            let sq_count = available_squares.count();
            self.mg_mobility_score += MG_QUEEN_MOBILITY_BONUS[sq_count as usize];
            self.eg_mobility_score += EG_QUEEN_MOBILITY_BONUS[sq_count as usize];
        }

        for sq in board.queens(Black) {
            let available_squares = sq.queen_squares(blockers) 
                // & black_safe_squares
                & !black_pieces;

            let sq_count = available_squares.count();
            self.mg_mobility_score -= MG_QUEEN_MOBILITY_BONUS[sq_count as usize];
            self.eg_mobility_score -= EG_QUEEN_MOBILITY_BONUS[sq_count as usize];
        }
    }

    pub fn is_mate_score(eval: Eval) -> bool {
        Eval::abs(eval) >= Self::MATE - MAX_DEPTH as Eval
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
            MG_PIECE_VALUES[ptype] + MG_TABLES[ptype][sq.flip() as usize]
        } else {
            -MG_PIECE_VALUES[ptype] - MG_TABLES[ptype][sq as usize]
        }
    }

    fn eg_score(&self, sq: Square) -> Eval {
        let pcolor = self.color();
        let ptype = self.piece_type() as usize;

        // Calculate the piece's scores
        if pcolor.is_white() {
            EG_PIECE_VALUES[ptype] + EG_TABLES[ptype][sq.flip() as usize]
        } else {
            -EG_PIECE_VALUES[ptype] - EG_TABLES[ptype][sq as usize]
        }
    }

    fn game_phase(&self) -> u8 {
        Score::GAME_PHASE_VALUES[self.piece_type() as usize]
    }
}

////////////////////////////////////////////////////////////////////////////////
//
// Tests
//
////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use chess::{board::Board, bitboard::Bitboard};

    use crate::{tests::TEST_POSITIONS, evaluate::{Score, lookups::ISOLATED_PAWN_MASKS}};

    #[test]
    fn eval_symmetry() {
        for fen in TEST_POSITIONS {
            let board: Board = fen.parse().unwrap();
            let score = Score::new(&board);
            let score = score.total(board.current);

            let mirrored = board.mirror();
            let mirrored_score = Score::new(&mirrored);
            let mirrored_score = mirrored_score.total(mirrored.current);

            assert_eq!(score, mirrored_score);
        }
    }

    #[test]
    fn isolated_pawns() {
        use super::Color::*;

        let mut isolated_count = 0;
        let board: Board = "r3k2r/2pb1ppp/2pp1q2/p7/1nP1B3/1P2P3/P2N1PPP/R2QK2R w KQkq a6 0 14"
            .parse()
            .unwrap();

        for sq in board.pawns(White) {
            let mask = ISOLATED_PAWN_MASKS[sq as usize];

            if board.pawns(White) & mask == Bitboard::EMPTY {
                isolated_count += 1;
            }
        }

        for sq in board.pawns(Black) {
            let mask = ISOLATED_PAWN_MASKS[sq as usize];

            if board.pawns(Black) & mask == Bitboard::EMPTY {
                isolated_count += 1;
            }
        }

        assert_eq!(isolated_count, 1);
    }
}
