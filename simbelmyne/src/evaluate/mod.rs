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
use crate::evaluate::piece_square_tables::PIECE_SQUARE_TABLES;
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
use crate::evaluate::params::CONNECTED_PAWN_BONUS;
use crate::evaluate::params::PAWN_STORM_BONUS;
use crate::evaluate::params::KING_ZONE_ATTACKS;
use crate::evaluate::params::PHALANX_PAWN_BONUS;

use colored::Colorize;

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
    
    pawn_storm: S,

    mobility: S,

    virtual_mobility: S,

    king_zone: S,
}

impl Eval {
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
            + self.pawn_storm
            + self.mobility
            + self.virtual_mobility
            + self.king_zone;

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
            self.pawn_structure += connected_pawns(board, White) - connected_pawns(board, Black);
            self.pawn_structure += phalanx_pawns(board, White) - phalanx_pawns(board, Black);

            self.pawn_shield = pawn_shield(board, White) - pawn_shield(board, Black);
            self.pawn_storm = pawn_storm(board, White) - pawn_storm(board, Black);
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
            self.pawn_storm = pawn_storm(board, White) - pawn_storm(board, Black);
        }

        self.mobility = mobility(board, White) - mobility(board, Black);

        self.virtual_mobility = virtual_mobility(board, White) - virtual_mobility(board, Black);

        self.king_zone = king_zone(board, White) - king_zone(board, Black);
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
            self.pawn_structure += connected_pawns(board, White) - connected_pawns(board, Black);
            self.pawn_structure += phalanx_pawns(board, White) - phalanx_pawns(board, Black);

            self.pawn_shield = pawn_shield(board, White) - pawn_shield(board, Black);
            self.pawn_storm = pawn_storm(board, White) - pawn_storm(board, Black);

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
            self.pawn_storm = pawn_storm(board, White) - pawn_storm(board, Black);
        }

        self.mobility = mobility(board, White) - mobility(board, Black);

        self.virtual_mobility = virtual_mobility(board, White) - virtual_mobility(board, Black);

        self.king_zone = king_zone(board, White) - king_zone(board, Black);
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
            self.pawn_structure += connected_pawns(board, White) - connected_pawns(board, Black);
            self.pawn_structure += phalanx_pawns(board, White) - phalanx_pawns(board, Black);

            self.pawn_shield = pawn_shield(board, White) - pawn_shield(board, Black);
            self.pawn_storm = pawn_storm(board, White) - pawn_storm(board, Black);

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
            self.pawn_storm = pawn_storm(board, White) - pawn_storm(board, Black);
        }

        self.mobility = mobility(board, White) - mobility(board, Black);

        self.virtual_mobility = virtual_mobility(board, White) - virtual_mobility(board, Black);

        self.king_zone = king_zone(board, White) - king_zone(board, Black);
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


fn pawn_structure(board: &Board, us: Color) -> S {
    passed_pawns(board, us) 
        + connected_pawns(board, us) 
        + phalanx_pawns(board, us) 
        + isolated_pawns(board, us)
        + doubled_pawns(board, us)
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

fn connected_pawns(board: &Board, us: Color) -> S {
    let our_pawns = board.pawns(us);
    let mut total = S::default();

    for sq in our_pawns {
        let connected = (our_pawns & sq.pawn_attacks(us)).count();
        total += CONNECTED_PAWN_BONUS[connected as usize];
    }

    total
}

fn phalanx_pawns(board: &Board, us: Color) -> S {
    let our_pawns = board.pawns(us);
    let mut total = S::default();

    for sq in our_pawns {
        let adjacent_squares = Bitboard::from(sq.left()) | Bitboard::from(sq.right());
        let phalanx_pawns = our_pawns & adjacent_squares;
        let phalanx_count = phalanx_pawns.count();
        total += PHALANX_PAWN_BONUS[phalanx_count as usize];
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
    let mut total = S::default();
    let our_king = board.kings(us).first();
    let our_pawns = board.pawns(us);
    let shield_mask = PASSED_PAWN_MASKS[us as usize][our_king as usize];

    let shield_pawns = shield_mask & our_pawns;

    for pawn in shield_pawns {
        let distance = pawn.vdistance(our_king).min(3) - 1;
        total += PAWN_SHIELD_BONUS[distance];
    }

    total
}

fn pawn_storm(board: &Board, us: Color) -> S {
    let mut total = S::default();
    let them = !us;
    let their_king = board.kings(them).first();
    let our_pawns = board.pawns(us);
    let storm_mask = PASSED_PAWN_MASKS[them as usize][their_king as usize];

    let storm_pawns = storm_mask & our_pawns;

    for pawn in storm_pawns {
        let distance = pawn.vdistance(their_king).min(3) - 1;
        total += PAWN_STORM_BONUS[distance];
    }

    total
}

fn virtual_mobility(board: &Board, us: Color) -> S {
    let king_sq = board.kings(us).first();
    let blockers = board.all_occupied();
    let ours = board.occupied_by(us);
    let available_squares = king_sq.queen_squares(blockers) & !ours;
    let mobility = available_squares.count();

    VIRTUAL_MOBILITY_PENALTY[mobility as usize]
}

fn king_zone(board: &Board, us: Color) -> S {
    let mut attacks = 0;
    let blockers = board.all_occupied();

    let king_sq = board.kings(us).first();
    let king_zone = king_sq.king_squares();


    for knight in board.knights(!us) {
        attacks += (king_zone & knight.knight_squares()).count();
    }

    for bishop in board.bishops(!us) {
        attacks += (king_zone & bishop.bishop_squares(blockers)).count();
    }

    for rook in board.rooks(!us) {
        attacks += (king_zone & rook.rook_squares(blockers)).count();
    }

    for queen in board.queens(!us) {
        attacks += (king_zone & queen.queen_squares(blockers)).count();
    }

    let attacks = usize::min(attacks as usize, 15);

    KING_ZONE_ATTACKS[attacks]
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

////////////////////////////////////////////////////////////////////////////////
//
// Score
//
////////////////////////////////////////////////////////////////////////////////

pub trait ScoreExt {
    const MINUS_INF: Self;
    const PLUS_INF: Self;
    const DRAW: Self;
    const MATE: Self;

    /// Return whether or not a score is a mate score
    fn is_mate(self) -> bool;

    /// Return the number of plies until mate.
    fn mate_distance(self) -> i32;

    /// Normalize the score such that mate scores are considered relative to
    /// the _provided ply_.
    fn relative(self, ply: usize) -> Self;

    /// Denormalize a score such that any mate scores are considered relative 
    /// to the _root_.
    fn absolute(self, ply: usize) -> Self;
}

impl ScoreExt for Score {
    const MINUS_INF: Self = Self::MIN + 1;
    const PLUS_INF: Self = Self::MAX;
    const DRAW: Self = 0;
    const MATE: Self = 20_000;

    fn is_mate(self) -> bool {
        Self::abs(self) >= Self::MATE - MAX_MOVES as i32
    }

    fn mate_distance(self) -> i32 {
        (Self::MATE - self.abs()) as i32
    }

    fn relative(self, ply: usize) -> Self {
        if self.is_mate() {
            self + ply as Self
        } else {
            self
        }
    }

    fn absolute(self, ply: usize) -> Self {
        if self.is_mate() {
            self - ply as Self
        } else {
            self
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
//
// Print evaluation
//
////////////////////////////////////////////////////////////////////////////////

fn blank_line(rank: usize) -> String {
        let mut line: Vec<String> = Vec::new();
        line.push("  ║".to_string());
    if rank % 2 == 0 {
        line.push("       ".on_white().to_string());
        line.push("       ".on_black().to_string());
        line.push("       ".on_white().to_string());
        line.push("       ".on_black().to_string());
        line.push("       ".on_white().to_string());
        line.push("       ".on_black().to_string());
        line.push("       ".on_white().to_string());
        line.push("       ".on_black().to_string());
    } else {
        line.push("       ".on_black().to_string());
        line.push("       ".on_white().to_string());
        line.push("       ".on_black().to_string());
        line.push("       ".on_white().to_string());
        line.push("       ".on_black().to_string());
        line.push("       ".on_white().to_string());
        line.push("       ".on_black().to_string());
        line.push("       ".on_white().to_string());
    }

    line.push("║ ".to_string());
    line.join("")
}

pub fn print_eval(board: &Board) -> String {
    let eval = Eval::new(board);

    let mut lines: Vec<String> = vec![];
    lines.push("      a      b      c      d      e      f      g      h    ".to_string());
    lines.push("  ╔════════════════════════════════════════════════════════╗".to_string());

    for (rank, squares) in Square::RANKS.into_iter().enumerate() {
        lines.push(blank_line(rank));

        // Piece character
        let mut line: Vec<String> = vec![];
        line.push((8 - rank).to_string());
        line.push(" ║".to_string());
        for (file, sq) in squares.into_iter().enumerate() {
            let bg = if (rank + file) % 2 == 0 { "white" } else { "black" };
            let fg = if (rank + file) % 2 == 0 { "black" } else { "white" };

            let square = match board.get_at(sq) {
                Some(piece) => format!("   {}   ", piece).color(fg).on_color(bg),
                None => "       ".to_string().on_color(bg),
            };

            line.push(square.to_string());
        }
        line.push("║ ".to_string());
        line.push((8 - rank).to_string());
        lines.push(line.join(""));

        lines.push(blank_line(rank));

        // Piece score
        let mut line: Vec<String> = vec![];
        line.push("  ║".to_string());
        for (file, sq) in squares.into_iter().enumerate() {
            let bg = if (rank + file) % 2 == 0 { "white" } else { "black" };
            let fg = if (rank + file) % 2 == 0 { "black" } else { "white" };
            let score = if let Some(piece) = board.get_at(sq) {
                // Get score for piece
                let score = material(piece) 
                    + psqt(piece, sq);
                let pawn_score = score.lerp(eval.game_phase) as f32 / 100.0;

                format!("{:.2}", pawn_score)
            } else {
                "".to_string()
            };

            line.push(format!("{:^7}", score.color(fg).on_color(bg)));
            
        }
        line.push("║  ".to_string());
        let line = line.join("");

        lines.push(line);


    }
    lines.push("  ╚════════════════════════════════════════════════════════╝".to_string());
    lines.push("      a      b      c      d      e      f      g      h    ".to_string());

    lines.push("\n".to_string());
    lines.push("Evaluation features:".blue().to_string());
    lines.push("--------------------".blue().to_string());

    let white_pawn_structure =  pawn_structure(board, White).lerp(eval.game_phase) as f32 / 100.0;
    let black_pawn_structure = -pawn_structure(board, Black).lerp(eval.game_phase) as f32 / 100.0;
    lines.push(format!("{:<20} {:>7.2} {:>7.2}", "Pawn structure:", white_pawn_structure, black_pawn_structure));

    let white_bishop_pair =  bishop_pair(board, White).lerp(eval.game_phase) as f32 / 100.0;
    let black_bishop_pair = -bishop_pair(board, Black).lerp(eval.game_phase) as f32 / 100.0;
    lines.push(format!("{:<20} {:>7.2} {:>7.2}", "Bishop pair", white_bishop_pair, black_bishop_pair));

    let white_rook_open_file =  rook_open_file(board, White).lerp(eval.game_phase) as f32 / 100.0;
    let black_rook_open_file = -rook_open_file(board, Black).lerp(eval.game_phase) as f32 / 100.0;
    lines.push(format!("{:<20} {:>7.2} {:>7.2}", "Rook on open file:", white_rook_open_file, black_rook_open_file));

    let white_pawn_shield =  pawn_shield(board, White).lerp(eval.game_phase) as f32 / 100.0;
    let black_pawn_shield = -pawn_shield(board, Black).lerp(eval.game_phase) as f32 / 100.0;
    lines.push(format!("{:<20} {:>7.2} {:>7.2}", "Pawn shield:", white_pawn_shield, black_pawn_shield));

    let white_pawn_storm =  pawn_storm(board, White).lerp(eval.game_phase) as f32 / 100.0;
    let black_pawn_storm = -pawn_storm(board, Black).lerp(eval.game_phase) as f32 / 100.0;
    lines.push(format!("{:<20} {:>7.2} {:>7.2}", "Pawn storm:", white_pawn_storm, black_pawn_storm));

    let white_mobility =  mobility(board, White).lerp(eval.game_phase) as f32 / 100.0;
    let black_mobility = -mobility(board, Black).lerp(eval.game_phase) as f32 / 100.0;
    lines.push(format!("{:<20} {:>7.2} {:>7.2}", "Mobility:", white_mobility, black_mobility));

    let white_virtual_mobility =  virtual_mobility(board, White).lerp(eval.game_phase) as f32 / 100.0;
    let black_virtual_mobility = -virtual_mobility(board, Black).lerp(eval.game_phase) as f32 / 100.0;
    lines.push(format!("{:<20} {:>7.2} {:>7.2}", "Virtual mobility:", white_virtual_mobility, black_virtual_mobility));

    let white_king_zone =  king_zone(board, White).lerp(eval.game_phase) as f32 / 100.0;
    let black_king_zone = -king_zone(board, Black).lerp(eval.game_phase) as f32 / 100.0;
    lines.push(format!("{:<20} {:>7.2} {:>7.2}", "King zone:", white_king_zone, black_king_zone));

    lines.push("".to_string());

    lines.push(format!("Total: {}", eval.total(board.current)));

    lines.push(format!("{eval:?}"));

    lines.join("\n")
}
