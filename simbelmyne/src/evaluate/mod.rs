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

use self::params::PASSERS_ENEMY_KING_PENALTY;
use self::params::PASSERS_FRIENDLY_KING_BONUS;

pub type Score = i32;

const WHITE: bool = true;
const BLACK: bool = false;

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

    /// The total material score, based on the piece values
    material: S,

    /// The total positional score, based on the piece and occupied square
    psqt: S,

    /// The total pawn structure score
    pawn_structure: S,

    /// A bonus score for having two bishops on the board
    bishop_pair: S,

    /// A bonus for having a rook on an open file
    rook_open_file: S,

    /// A bonus for having pawns protecting the king
    pawn_shield: S,
    
    /// A bonus for having pawns attacking the enemy king
    pawn_storm: S,

    /// A bonus for having adequate squares to move to
    mobility: S,

    /// A penalty for having to many attack vectors for checking the king
    virtual_mobility: S,

    /// A penalty for how many of the king's surrounding squares are under 
    /// attack
    king_zone: S,

    /// A bonus for keeping the king near friendly passed pawns
    passers_friendly_king: S,

    /// A bonus for keeping the king near enemy passed pawns
    passers_enemy_king: S,
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
    pub fn total(&self, board: &Board) -> Score {
        let mut total = self.material
            + self.psqt
            + self.pawn_structure
            + self.bishop_pair
            + self.rook_open_file
            + self.pawn_shield
            + self.pawn_storm
            + self.passers_friendly_king
            + self.passers_enemy_king;

        let mut ctx = EvalContext::new(board);
        total += board.mobility::<WHITE>(&mut ctx)  - board.mobility::<BLACK>(&mut ctx);
        total += board.virtual_mobility::<WHITE>()  - board.virtual_mobility::<BLACK>();
        total += board.king_zone::<WHITE>(&mut ctx) - board.king_zone::<BLACK>(&mut ctx);

        let score = total.lerp(self.game_phase);

        if board.current.is_white() { score } else { -score }
    }

    /// Update the score by adding a piece to it
    pub fn add(&mut self, piece: Piece, sq: Square, board: &Board) {
        self.game_phase += Self::phase_value(piece);

        self.material += board.material(piece);
        self.psqt += board.psqt(piece, sq);

        if piece.is_pawn() {
            self.pawn_structure = board.pawn_structure::<WHITE>()    - board.pawn_structure::<BLACK>();
            self.pawn_shield    = board.pawn_shield::<WHITE>()       - board.pawn_shield::<BLACK>();
            self.pawn_storm     = board.pawn_storm::<WHITE>()        - board.pawn_storm::<BLACK>();
            self.rook_open_file = board.rook_open_file::<WHITE>()    - board.rook_open_file::<BLACK>();
            self.passers_friendly_king = board.passers_friendly_king::<WHITE>() - board.passers_friendly_king::<BLACK>();
            self.passers_enemy_king = board.passers_enemy_king::<WHITE>() - board.passers_enemy_king::<BLACK>();
        }

        if piece.is_bishop() {
            self.bishop_pair    = board.bishop_pair::<WHITE>()       - board.bishop_pair::<BLACK>();
        }

        if piece.is_rook() {
            self.rook_open_file = board.rook_open_file::<WHITE>()    - board.rook_open_file::<BLACK>();
        }

        if piece.is_king() {
            self.pawn_shield    = board.pawn_shield::<WHITE>()       - board.pawn_shield::<BLACK>();
            self.pawn_storm     = board.pawn_storm::<WHITE>()        - board.pawn_storm::<BLACK>();
            self.passers_friendly_king = board.passers_friendly_king::<WHITE>() - board.passers_friendly_king::<BLACK>();
            self.passers_enemy_king = board.passers_enemy_king::<WHITE>() - board.passers_enemy_king::<BLACK>();
        }
    }

    /// Update the score by removing a piece from it
    pub fn remove(&mut self, piece: Piece, sq: Square, board: &Board) {
        self.game_phase -= Self::phase_value(piece);

        self.material -= board.material(piece);
        self.psqt -= board.psqt(piece, sq);

        if piece.is_pawn() {
            self.pawn_structure = board.pawn_structure::<WHITE>()    - board.pawn_structure::<BLACK>();
            self.pawn_shield    = board.pawn_shield::<WHITE>()       - board.pawn_shield::<BLACK>();
            self.pawn_storm     = board.pawn_storm::<WHITE>()        - board.pawn_storm::<BLACK>();
            self.rook_open_file = board.rook_open_file::<WHITE>()    - board.rook_open_file::<BLACK>();
            self.passers_friendly_king = board.passers_friendly_king::<WHITE>() - board.passers_friendly_king::<BLACK>();
            self.passers_enemy_king = board.passers_enemy_king::<WHITE>() - board.passers_enemy_king::<BLACK>();
        }

        if piece.is_bishop() {
            self.bishop_pair    = board.bishop_pair::<WHITE>()       - board.bishop_pair::<BLACK>();
        }

        if piece.is_rook() {
            self.rook_open_file = board.rook_open_file::<WHITE>()    - board.rook_open_file::<BLACK>();
        }

        if piece.is_king() {
            self.pawn_shield    = board.pawn_shield::<WHITE>()       - board.pawn_shield::<BLACK>();
            self.pawn_storm     = board.pawn_storm::<WHITE>()        - board.pawn_storm::<BLACK>();
            self.passers_friendly_king = board.passers_friendly_king::<WHITE>() - board.passers_friendly_king::<BLACK>();
            self.passers_enemy_king = board.passers_enemy_king::<WHITE>() - board.passers_enemy_king::<BLACK>();
        }
    }

    /// Update the score by moving a piece from one square to another
    pub fn update(&mut self, piece: Piece, from: Square, to: Square, board: &Board) {
        self.psqt -= board.psqt(piece, from);
        self.psqt += board.psqt(piece, to);

        if piece.is_pawn() {
            self.pawn_structure = board.pawn_structure::<WHITE>()    - board.pawn_structure::<BLACK>();
            self.pawn_shield    = board.pawn_shield::<WHITE>()       - board.pawn_shield::<BLACK>();
            self.pawn_storm     = board.pawn_storm::<WHITE>()        - board.pawn_storm::<BLACK>();
            self.rook_open_file = board.rook_open_file::<WHITE>()    - board.rook_open_file::<BLACK>();
            self.passers_friendly_king = board.passers_friendly_king::<WHITE>() - board.passers_friendly_king::<BLACK>();
            self.passers_enemy_king = board.passers_enemy_king::<WHITE>() - board.passers_enemy_king::<BLACK>();
        }

        if piece.is_bishop() {
            self.bishop_pair    = board.bishop_pair::<WHITE>()       - board.bishop_pair::<BLACK>();
        }

        if piece.is_rook() {
            self.rook_open_file = board.rook_open_file::<WHITE>()    - board.rook_open_file::<BLACK>();
        }

        if piece.is_king() {
            self.pawn_shield    = board.pawn_shield::<WHITE>()       - board.pawn_shield::<BLACK>();
            self.pawn_storm     = board.pawn_storm::<WHITE>()        - board.pawn_storm::<BLACK>();
            self.passers_friendly_king = board.passers_friendly_king::<WHITE>() - board.passers_friendly_king::<BLACK>();
            self.passers_enemy_king = board.passers_enemy_king::<WHITE>() - board.passers_enemy_king::<BLACK>();
        }
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

struct EvalContext {
    king_zones: [Bitboard; Color::COUNT],
    king_attacks: [u32; Color::COUNT],
}

impl EvalContext {
    pub fn new(board: &Board) -> Self {
        let white_king = board.kings(Color::White).first();
        let black_king = board.kings(Color::Black).first();

        let white_king_zone = white_king.king_squares();
        let black_king_zone = black_king.king_squares();

        Self {
            king_zones: [white_king_zone, black_king_zone],
            king_attacks: [0, 0]
        }
    }
}

trait Evaluate {
    fn material(&self, piece: Piece) -> S;
    fn psqt(&self, piece: Piece, sq: Square) -> S;

    fn pawn_structure<const WHITE: bool>(&self) -> S;
    fn pawn_shield<const WHITE: bool>(&self) -> S;
    fn pawn_storm<const WHITE: bool>(&self) -> S;

    fn bishop_pair<const WHITE: bool>(&self) -> S;
    fn rook_open_file<const WHITE: bool>(&self) -> S;
    fn mobility<const WHITE: bool>(&self, ctx: &mut EvalContext) -> S;
    fn virtual_mobility<const WHITE: bool>(&self) -> S;
    fn king_zone<const WHITE: bool>(&self, ctx: &mut EvalContext) -> S;
    fn passers_friendly_king<const WHITE: bool>(&self) -> S;
    fn passers_enemy_king<const WHITE: bool>(&self) -> S;
}

impl Evaluate for Board {
    fn material(&self, piece: Piece) -> S {
        if piece.color().is_white() {
            PIECE_VALUES[piece.piece_type() as usize]
        } else {
            -PIECE_VALUES[piece.piece_type() as usize]
        }
    }

    fn psqt(&self, piece: Piece, sq: Square) -> S {
        if piece.color().is_white() {
            PIECE_SQUARE_TABLES[piece.piece_type() as usize][sq.flip() as usize]
        } else {
            -PIECE_SQUARE_TABLES[piece.piece_type() as usize][sq as usize]
        }
    }

    fn pawn_structure<const WHITE: bool>(&self) -> S {
        let mut total = S::default();

        let us = if WHITE { White } else { Black };
        let our_pawns = self.pawns(us);
        let their_pawns = self.pawns(!us);

        for sq in our_pawns {
            // Passed pawns
            let passed_mask = PASSED_PAWN_MASKS[us as usize][sq as usize];
            if their_pawns & passed_mask == Bitboard::EMPTY {
                let sq = if us.is_white() { sq.flip() } else { sq };
                total += PASSED_PAWN_TABLE[sq as usize];
            }

            // Connected pawns
            let connected = (our_pawns & sq.pawn_attacks(us)).count();
            total += CONNECTED_PAWN_BONUS[connected as usize];

            // Phalanx pawns
            let neighbors = Bitboard::from(sq.left()) | Bitboard::from(sq.right());
            let phalanx_pawns = our_pawns & neighbors;
            let phalanx_count = phalanx_pawns.count();
            total += PHALANX_PAWN_BONUS[phalanx_count as usize];

            // Isolated pawns
            let isolated_mask = ISOLATED_PAWN_MASKS[sq as usize];
            if our_pawns & isolated_mask == Bitboard::EMPTY {
                total += ISOLATED_PAWN_PENALTY;
            }

            // Doubled pawns
            // FIXME: Doesn't seem to be correct?
            // let is_doubled = (our_pawns & FILES[sq as usize]).count() > 1;
            // if is_doubled {
            //     total += DOUBLED_PAWN_PENALTY;
            // }
        }

        // Doubled pawns
        for mask in DOUBLED_PAWN_MASKS {
            let doubled = (our_pawns & mask).count().saturating_sub(1) as Score;
            total += DOUBLED_PAWN_PENALTY * doubled;
        }

        total
    }

    fn pawn_shield<const WHITE: bool>(&self) -> S {
        let mut total = S::default();

        let us = if WHITE { White } else { Black };
        let our_king = self.kings(us).first();
        let our_pawns = self.pawns(us);
        let shield_mask = PASSED_PAWN_MASKS[us as usize][our_king as usize];
        let shield_pawns = shield_mask & our_pawns;

        for pawn in shield_pawns {
            let distance = pawn.vdistance(our_king).min(3) - 1;
            total += PAWN_SHIELD_BONUS[distance];
        }

        total
    }

    fn pawn_storm<const WHITE: bool>(&self) -> S {
        let mut total = S::default();

        let us = if WHITE { White } else { Black };
        let them = !us;
        let their_king = self.kings(them).first();
        let our_pawns = self.pawns(us);
        let storm_mask = PASSED_PAWN_MASKS[them as usize][their_king as usize];

        let storm_pawns = storm_mask & our_pawns;

        for pawn in storm_pawns {
            let distance = pawn.vdistance(their_king).min(3) - 1;
            total += PAWN_STORM_BONUS[distance];
        }

        total
    }

    fn passers_friendly_king<const WHITE: bool>(&self) -> S {
        let mut total = S::default();

        let us = if WHITE { White } else { Black };
        let our_king = self.kings(us).first();
        let our_pawns = self.pawns(us);
        let their_pawns = self.pawns(!us);

        for pawn in our_pawns {
            if (PASSED_PAWN_MASKS[us as usize][pawn as usize] & their_pawns).is_empty() {
                let distance = pawn.max_dist(our_king);
                total += PASSERS_FRIENDLY_KING_BONUS[distance - 1];
            }
        }

        total
    }

    fn passers_enemy_king<const WHITE: bool>(&self) -> S {
        let mut total = S::default();

        let us = if WHITE { White } else { Black };
        let our_pawns = self.pawns(us);
        let their_pawns = self.pawns(!us);
        let their_king = self.kings(!us).first();

        for pawn in our_pawns {
            if (PASSED_PAWN_MASKS[us as usize][pawn as usize] & their_pawns).is_empty() {
                let distance = pawn.max_dist(their_king);
                total += PASSERS_ENEMY_KING_PENALTY[distance - 1];
            }
        }

        total
    }


    fn bishop_pair<const WHITE: bool>(&self) -> S {
        let us = if WHITE { White } else { Black };

        if self.bishops(us).count() == 2 {
            BISHOP_PAIR_BONUS
        } else {
            S::default()
        }
    }

    fn rook_open_file<const WHITE: bool>(&self) -> S {
        use PieceType::*;
        let mut total = S::default();

        let us = if WHITE { White } else { Black };
        let pawns = self.piece_bbs[Pawn as usize];

        for sq in self.rooks(us) {
            if (FILES[sq as usize] & pawns).is_empty() {
                total += ROOK_OPEN_FILE_BONUS;
            }
        }

        total
    }

    fn mobility<const WHITE: bool>(&self, ctx: &mut EvalContext) -> S {
        let mut total = S::default();

        let us = if WHITE { White } else { Black };
        let blockers = self.all_occupied();
        let enemy_king_zone = ctx.king_zones[!us as usize];

        let pawn_attacks: Bitboard = self.pawns(!us)
            .map(|sq| sq.pawn_attacks(!us))
            .collect();

        let blocked_pawns = if WHITE {
            self.pawns(us) & (self.pawns(!us) >> 8)
        } else {
            self.pawns(us) & (self.pawns(!us) << 8)
        };

        let mobility_squares = !pawn_attacks & !blocked_pawns;

        // let mobility_squares = !pawn_attacks;

        for sq in self.knights(us) {
            // King safety
            let available_squares = sq.knight_squares();
            let king_attacks = enemy_king_zone & available_squares;
            ctx.king_attacks[!us as usize] += king_attacks.count();

            // Mobility
            let mut available_squares = available_squares & mobility_squares;

            if self.get_pinrays(us).contains(sq) {
                available_squares &= self.get_pinrays(us);
            }

            let sq_count = available_squares.count();

            total += KNIGHT_MOBILITY_BONUS[sq_count as usize];
        }

        for sq in self.bishops(us) {
            // King safety
            let available_squares = sq.bishop_squares(blockers);
            let king_attacks = enemy_king_zone & available_squares;
            ctx.king_attacks[!us as usize] += king_attacks.count();

            // Mobility
            let mut available_squares = available_squares & mobility_squares;

            if self.get_pinrays(us).contains(sq) {
                available_squares &= self.get_pinrays(us);
            }

            let sq_count = available_squares.count();

            total += BISHOP_MOBILITY_BONUS[sq_count as usize];
        }

        for sq in self.rooks(us) {
            // King safety
            let available_squares = sq.rook_squares(blockers);
            let king_attacks = enemy_king_zone & available_squares;
            ctx.king_attacks[!us as usize] += king_attacks.count();

            // Mobility
            let mut available_squares = available_squares & mobility_squares;

            if self.get_pinrays(us).contains(sq) {
                available_squares &= self.get_pinrays(us);
            }

            let sq_count = available_squares.count();

            total += ROOK_MOBILITY_BONUS[sq_count as usize];
        }

        for sq in self.queens(us) {
            // King safety
            let available_squares = sq.queen_squares(blockers);
            let king_attacks = enemy_king_zone & available_squares;
            ctx.king_attacks[!us as usize] += king_attacks.count();

            // Mobility
            let mut available_squares = available_squares & mobility_squares;

            if self.get_pinrays(us).contains(sq) {
                available_squares &= self.get_pinrays(us);
            }

            let sq_count = available_squares.count();

            total += QUEEN_MOBILITY_BONUS[sq_count as usize];
        }

        total
    }

    fn virtual_mobility<const WHITE: bool>(&self) -> S {
        let us = if WHITE { White } else { Black };
        let king_sq = self.kings(us).first();
        let blockers = self.all_occupied();
        let ours = self.occupied_by(us);
        let available_squares = king_sq.queen_squares(blockers) & !ours;
        let mobility = available_squares.count();

        VIRTUAL_MOBILITY_PENALTY[mobility as usize]
    }

    fn king_zone<const WHITE: bool>(&self, ctx: &mut EvalContext) -> S {
        let us = if WHITE { White } else { Black };
        let attacks = ctx.king_attacks[us as usize];
        let attacks = usize::min(attacks as usize, 15);

        KING_ZONE_ATTACKS[attacks]
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
        (phase as Score * self.0 + (24 - phase as Score) * self.1) / 24 
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
                let score = board.material(piece) + board.psqt(piece, sq);
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

    let mut ctx = EvalContext::new(board);

    let white_pawn_structure =  board.pawn_structure::<WHITE>().lerp(eval.game_phase) as f32 / 100.0;
    let black_pawn_structure = -board.pawn_structure::<BLACK>().lerp(eval.game_phase) as f32 / 100.0;
    lines.push(format!("{:<20} {:>7.2} {:>7.2}", "Pawn structure:", white_pawn_structure, black_pawn_structure));

    let white_bishop_pair =  board.bishop_pair::<WHITE>().lerp(eval.game_phase) as f32 / 100.0;
    let black_bishop_pair = -board.bishop_pair::<BLACK>().lerp(eval.game_phase) as f32 / 100.0;
    lines.push(format!("{:<20} {:>7.2} {:>7.2}", "Bishop pair", white_bishop_pair, black_bishop_pair));

    let white_rook_open_file =  board.rook_open_file::<WHITE>().lerp(eval.game_phase) as f32 / 100.0;
    let black_rook_open_file = -board.rook_open_file::<BLACK>().lerp(eval.game_phase) as f32 / 100.0;
    lines.push(format!("{:<20} {:>7.2} {:>7.2}", "Rook on open file:", white_rook_open_file, black_rook_open_file));

    let white_pawn_shield =  board.pawn_shield::<WHITE>().lerp(eval.game_phase) as f32 / 100.0;
    let black_pawn_shield = -board.pawn_shield::<BLACK>().lerp(eval.game_phase) as f32 / 100.0;
    lines.push(format!("{:<20} {:>7.2} {:>7.2}", "Pawn shield:", white_pawn_shield, black_pawn_shield));

    let white_pawn_storm =  board.pawn_storm::<WHITE>().lerp(eval.game_phase) as f32 / 100.0;
    let black_pawn_storm = -board.pawn_storm::<BLACK>().lerp(eval.game_phase) as f32 / 100.0;
    lines.push(format!("{:<20} {:>7.2} {:>7.2}", "Pawn storm:", white_pawn_storm, black_pawn_storm));

    let white_mobility =  board.mobility::<WHITE>(&mut ctx).lerp(eval.game_phase) as f32 / 100.0;
    let black_mobility = -board.mobility::<BLACK>(&mut ctx).lerp(eval.game_phase) as f32 / 100.0;
    lines.push(format!("{:<20} {:>7.2} {:>7.2}", "Mobility:", white_mobility, black_mobility));

    let white_virtual_mobility =  board.virtual_mobility::<WHITE>().lerp(eval.game_phase) as f32 / 100.0;
    let black_virtual_mobility = -board.virtual_mobility::<BLACK>().lerp(eval.game_phase) as f32 / 100.0;
    lines.push(format!("{:<20} {:>7.2} {:>7.2}", "Virtual mobility:", white_virtual_mobility, black_virtual_mobility));

    let white_king_zone =  board.king_zone::<WHITE>(&mut ctx).lerp(eval.game_phase) as f32 / 100.0;
    let black_king_zone = -board.king_zone::<BLACK>(&mut ctx).lerp(eval.game_phase) as f32 / 100.0;
    lines.push(format!("{:<20} {:>7.2} {:>7.2}", "King zone:", white_king_zone, black_king_zone));

    lines.push("".to_string());

    lines.push(format!("Total: {}", eval.total(&board)));

    lines.join("\n")
}
