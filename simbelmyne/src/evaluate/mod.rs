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

mod lookups;
pub mod params;
pub mod tuner;
mod piece_square_tables;

use std::iter::Sum;
use std::ops::Add;
use std::ops::AddAssign;
use std::ops::Mul;
use std::ops::Neg;
use std::ops::Sub;
use std::ops::SubAssign;

use chess::bitboard::Bitboard;
use chess::board::Board;
use chess::movegen::legal_moves::MAX_MOVES;
use chess::piece::Piece;
use chess::square::Square;
use chess::piece::PieceType;
use chess::piece::Color;
use chess::piece::Color::*;

use crate::evaluate::lookups::FILES;
use crate::evaluate::lookups::DOUBLED_PAWN_MASKS;
use crate::evaluate::lookups::ISOLATED_PAWN_MASKS;
use crate::evaluate::lookups::PASSED_PAWN_MASKS;
use crate::evaluate::params::BISHOP_MOBILITY_BONUS;
use crate::evaluate::params::BISHOP_PAIR_BONUS;
use crate::evaluate::params::DOUBLED_PAWN_PENALTY;
use crate::evaluate::params::ISOLATED_PAWN_PENALTY;
use crate::evaluate::params::KNIGHT_MOBILITY_BONUS;
use crate::evaluate::params::PASSED_PAWN_TABLE;
use crate::evaluate::params::QUEEN_MOBILITY_BONUS;
use crate::evaluate::params::ROOK_MOBILITY_BONUS;
use crate::evaluate::params::ROOK_OPEN_FILE_BONUS;
use crate::evaluate::params::PIECE_VALUES;
use crate::evaluate::params::PAWN_SHIELD_BONUS;
use crate::evaluate::params::VIRTUAL_MOBILITY_PENALTY;
use crate::evaluate::piece_square_tables::PIECE_SQUARE_TABLES;

pub type Score = i32;

////////////////////////////////////////////////////////////////////////////////
//
// Evaluation logic
//
////////////////////////////////////////////////////////////////////////////////

/// An `Evaluation` keeps track of the granular score breakdown
///
/// Keep track of both midgame and endgame scores for a given position, as well
/// as the "game_phase" parameter. Keeping track of all of these independently
/// means we can incrementally update the score by adding/removing pieces as the
/// game progresses.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct Eval {
    /// Value between 0 and 24, keeping track of how far along the game we are.
    /// A score of 0 corresponds to endgame, a score of 24 is in the opening.
    game_phase: u8,

    material: S,

    psqt: S,

    pawn_structure: S,

    bishop_pair: S,

    rook_open_file: S,

    pawn_shield: S,

    mobility: S,

    virtual_mobility: S,
}

impl Eval {
    pub const MIN: Score = Score::MIN + 1;
    pub const MAX: Score = Score::MAX;
    pub const MATE: Score = 20_000;
    pub const DRAW: Score = 0;

    /// Create a new score for a board
    pub fn new(board: &Board) -> Self {
        let mut eval = Self::default();

        // Walk through all the pieces on the board, and add update the Score
        // counter for each one.
        for (sq_idx, piece) in board.piece_list.into_iter().enumerate() {
            if let Some(piece) = piece {
                let square = Square::from(sq_idx);

                eval.add(piece, square, board);
            }
        }

        eval
    }

    /// Return the total (weighted) score for the position
    pub fn total(&self, side: Color) -> Score {
        let total = self.material
            + self.psqt
            + self.pawn_structure
            + self.bishop_pair
            + self.rook_open_file
            + self.pawn_shield
            + self.mobility
            + self.virtual_mobility;

        let score = total.lerp(self.game_phase);

        if side.is_white() { score } else { -score }
    }

    /// Update the score by adding a piece to it
    pub fn add(&mut self, piece: Piece, sq: Square, board: &Board) {
        self.game_phase += Self::phase_value(piece);

        self.material += material(piece);
        self.psqt += psqt(piece, sq);

        if piece.is_pawn() {
            self.pawn_structure = S::default();
            self.pawn_structure += passed_pawns(board, White) - passed_pawns(board, Black);
            self.pawn_structure += isolated_pawns(board, White) - isolated_pawns(board, Black);
            self.pawn_structure += doubled_pawns(board, White) - doubled_pawns(board, Black);

            self.pawn_shield = pawn_shield(board, White) - pawn_shield(board, Black);
            self.rook_open_file = rook_open_file(board, White) - rook_open_file(board, Black);
        }

        if piece.is_bishop() {
            self.bishop_pair = bishop_pair(board, White) - bishop_pair(board, Black);
        }

        if piece.is_rook() {
            self.rook_open_file = rook_open_file(board, White) - rook_open_file(board, Black);
        }

        if piece.is_king() {
            self.pawn_shield = pawn_shield(board, White) - pawn_shield(board, Black);
        }

        self.mobility = mobility(board, White) - mobility(board, Black);

        self.virtual_mobility = virtual_mobility(board, White) - virtual_mobility(board, Black);
    }

    /// Update the score by removing a piece from it
    pub fn remove(&mut self, piece: Piece, sq: Square, board: &Board) {
        self.game_phase -= Self::phase_value(piece);

        self.material -= material(piece);
        self.psqt -= psqt(piece, sq);

        if piece.is_pawn() {
            self.pawn_structure = S::default();
            self.pawn_structure += passed_pawns(board, White) - passed_pawns(board, Black);
            self.pawn_structure += isolated_pawns(board, White) - isolated_pawns(board, Black);
            self.pawn_structure += doubled_pawns(board, White) - doubled_pawns(board, Black);

            self.pawn_shield = pawn_shield(board, White) - pawn_shield(board, Black);
            self.rook_open_file = rook_open_file(board, White) - rook_open_file(board, Black);
        }

        if piece.is_bishop() {
            self.bishop_pair = bishop_pair(board, White) - bishop_pair(board, Black);
        }

        if piece.is_rook() {
            self.rook_open_file = rook_open_file(board, White) - rook_open_file(board, Black);
        }

        if piece.is_king() {
            self.pawn_shield = pawn_shield(board, White) - pawn_shield(board, Black);
        }

        self.mobility = mobility(board, White) - mobility(board, Black);

        self.virtual_mobility = virtual_mobility(board, White) - virtual_mobility(board, Black);
    }

    /// Update the score by moving a piece from one square to another
    pub fn update(&mut self, piece: Piece, from: Square, to: Square, board: &Board) {
        self.psqt -= psqt(piece, from);
        self.psqt += psqt(piece, to);

        if piece.is_pawn() {
            self.pawn_structure = S::default();
            self.pawn_structure += passed_pawns(board, White) - passed_pawns(board, Black);
            self.pawn_structure += isolated_pawns(board, White) - isolated_pawns(board, Black);
            self.pawn_structure += doubled_pawns(board, White) - doubled_pawns(board, Black);

            self.pawn_shield = pawn_shield(board, White) - pawn_shield(board, Black);
            self.rook_open_file = rook_open_file(board, White) - rook_open_file(board, Black);
        }

        if piece.is_bishop() {
            self.bishop_pair = bishop_pair(board, White) - bishop_pair(board, Black);
        }

        if piece.is_rook() {
            self.rook_open_file = rook_open_file(board, White) - rook_open_file(board, Black);
        }

        if piece.is_king() {
            self.pawn_shield = pawn_shield(board, White) - pawn_shield(board, Black);
        }

        self.mobility = mobility(board, White) - mobility(board, Black);

        self.virtual_mobility = virtual_mobility(board, White) - virtual_mobility(board, Black);
    }


    /// Values assignd to each piece type to calculate the approximate stage 
    /// of the game
    const GAME_PHASE_VALUES: [u8; PieceType::COUNT] = [0, 1, 1, 2, 4, 0];

    /// Return the game phase as a value between 0 and 24. 
    ///
    /// 0 corresponds to endgame, 24 corresponds to midgame
    fn phase_value(piece: Piece) -> u8 {
        Self::GAME_PHASE_VALUES[piece.piece_type() as usize]
    }
}

/// Return the material contribution to the total evaluation for a particular 
/// piece
fn material(piece: Piece) -> S {
    if piece.color().is_white() {
        PIECE_VALUES[piece.piece_type() as usize]
    } else {
        -PIECE_VALUES[piece.piece_type() as usize]
    }
}

/// Return the positional contribution to the total evaluation for a particular 
/// piece.
fn psqt(piece: Piece, sq: Square) -> S {
    if piece.color().is_white() {
        PIECE_SQUARE_TABLES[piece.piece_type() as usize][sq.flip() as usize]
    } else {
        -PIECE_SQUARE_TABLES[piece.piece_type() as usize][sq as usize]
    }
}

/// Return the bonus due to passed pawns for a given position and color
fn passed_pawns(board: &Board, us: Color) -> S {
    let our_pawns = board.pawns(us);
    let their_pawns = board.pawns(!us);
    let mut total = S::default();

    for sq in our_pawns {
        let mask = PASSED_PAWN_MASKS[us as usize][sq as usize];

        if their_pawns & mask == Bitboard::EMPTY {
            let sq = if us.is_white() { sq.flip() } else { sq };
            total += PASSED_PAWN_TABLE[sq as usize];
        }
    }

    total
}

/// Return the penalty due to isolated pawns for a given position and color
fn isolated_pawns(board: &Board, us: Color) -> S {
    let our_pawns = board.pawns(us);
    let mut total = S::default();

    for sq in our_pawns {
        let mask = ISOLATED_PAWN_MASKS[sq as usize];

        if our_pawns & mask == Bitboard::EMPTY {
            total += ISOLATED_PAWN_PENALTY;
        }
    }

    total
}

/// Return the penalty due to doubled pawns for a given position and color
fn doubled_pawns(board: &Board, us: Color) -> S {
    let our_pawns = board.pawns(us);
    let mut total = S::default();

    for mask in DOUBLED_PAWN_MASKS {
        let doubled_white = (our_pawns & mask).count().saturating_sub(1) as Score;
        total += DOUBLED_PAWN_PENALTY * doubled_white;
    }

    total
}

/// Return the bonus for having a bishop pair for a given position and color.
fn bishop_pair(board: &Board, us: Color) -> S {
    if board.bishops(us).count() == 2 {
        BISHOP_PAIR_BONUS
    } else {
        S::default()
    }
}

/// Return the bonus for having a rook on an open file, for a given position and 
/// color.
fn rook_open_file(board: &Board, us: Color) -> S {
    use PieceType::*;
    let pawns = board.piece_bbs[Pawn as usize];
    let mut total = S::default();

    for sq in board.rooks(us) {
        if (FILES[sq as usize] & pawns).is_empty() {
            total += ROOK_OPEN_FILE_BONUS;
        }
    }

    total
}

/// Return the bonus/penalty due to the mobility of each piece, for a given
/// position and color.
fn mobility(board: &Board, us: Color) -> S {
    let blockers = board.all_occupied();
    let our_pieces = board.occupied_by(us);
    let mut total = S::default();

    for sq in board.knights(us) {
        let available_squares = sq.knight_squares() & !our_pieces;
        let sq_count = available_squares.count();

        total += KNIGHT_MOBILITY_BONUS[sq_count as usize];
    }

    for sq in board.bishops(us) {
        let available_squares = sq.bishop_squares(blockers) & !our_pieces;
        let sq_count = available_squares.count();
        
        total += BISHOP_MOBILITY_BONUS[sq_count as usize];
    }
    
    for sq in board.rooks(us) {
        let available_squares = sq.rook_squares(blockers) & !our_pieces;
        let sq_count = available_squares.count();
    
        total += ROOK_MOBILITY_BONUS[sq_count as usize];
    }
    
    for sq in board.queens(us) {
        let available_squares = sq.queen_squares(blockers) & !our_pieces;
        let sq_count = available_squares.count();
    
        total += QUEEN_MOBILITY_BONUS[sq_count as usize];
    }
    
    total
}

fn pawn_shield(board: &Board, us: Color) -> S {
    let king_sq = board.kings(us).first();
    let shield_squares = king_sq.forward(us).into_iter()
        .chain(king_sq.forward(us).and_then(|sq| sq.left()))
        .chain(king_sq.forward(us).and_then(|sq| sq.right()))
        .collect::<Bitboard>();

    let pawn_shield = board.pawns(us) & shield_squares;

    PAWN_SHIELD_BONUS * pawn_shield.count() as Score
}

fn virtual_mobility(board: &Board, us: Color) -> S {
    let king_sq = board.kings(us).first();
    let blockers = board.all_occupied();
    let ours = board.occupied_by(us);
    let available_squares = king_sq.queen_squares(blockers) & !ours;
    let mobility = available_squares.count();

    VIRTUAL_MOBILITY_PENALTY[mobility as usize]
}

/// Return whether or not a score is a mate score
pub fn is_mate_score(score: Score) -> bool {
    Score::abs(score) >= Eval::MATE - MAX_MOVES as i32
}

/// Normalize the score such that mate scores are considered relative to
/// the _provided ply_.
pub fn get_relative_score(score: Score, ply: usize) -> Score {
    if is_mate_score(score) {
        score + ply as Score
    } else {
        score
    }
}

/// Denormalize a score such that any mate scores are considered relative 
/// to the _root_.
pub fn get_absolute_score(score: Score, ply: usize) -> Score {
    if is_mate_score(score) {
        score - ply as Score
    } else {
        score
    }
}

////////////////////////////////////////////////////////////////////////////////
//
// Weights
//
////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct S(pub Score, pub Score);

impl S {
    /// Interpolate between the midgame and endgame score according to a
    /// given `phase` which is a value between 0 and 24.
    pub fn lerp(&self, phase: u8) -> Score {
        phase as Score * self.0 / 24 + (24 - phase as Score) * self.1 / 24
    }
}

impl Add for S {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl AddAssign for S {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
        self.1 += rhs.1;
    }
}

impl Sub for S {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0, self.1 - rhs.1)
    }
}

impl SubAssign for S {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
        self.1 -= rhs.1;
    }
}

impl Mul<Score> for S {
    type Output = Self;

    fn mul(self, rhs: Score) -> Self::Output {
        Self(self.0 * rhs, self.1 * rhs)
    }
}

impl Neg for S {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(-self.0, -self.1)
    }
}

impl Sum for S {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::default(), Self::add)
    }
}
