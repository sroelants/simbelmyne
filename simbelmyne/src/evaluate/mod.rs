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
pub mod pawn_structure;
pub mod pretty_print;
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
use chess::constants::RANKS;
use chess::movegen::legal_moves::MAX_MOVES;
use chess::movegen::lookups::BETWEEN;
use chess::piece::Piece;
use chess::square::Square;
use chess::piece::PieceType;
use chess::piece::Color;
use chess::piece::Color::*;

use crate::evaluate::lookups::PASSED_PAWN_MASKS;
use crate::evaluate::params::CONNECTED_ROOKS_BONUS;
use crate::evaluate::params::QUEEN_OPEN_FILE_BONUS;
use crate::evaluate::piece_square_tables::PIECE_SQUARE_TABLES;
use crate::evaluate::params::BISHOP_MOBILITY_BONUS;
use crate::evaluate::params::BISHOP_PAIR_BONUS;
use crate::evaluate::params::KNIGHT_MOBILITY_BONUS;
use crate::evaluate::params::QUEEN_MOBILITY_BONUS;
use crate::evaluate::params::ROOK_MOBILITY_BONUS;
use crate::evaluate::params::ROOK_OPEN_FILE_BONUS;
use crate::evaluate::params::PIECE_VALUES;
use crate::evaluate::params::PAWN_SHIELD_BONUS;
use crate::evaluate::params::VIRTUAL_MOBILITY_PENALTY;
use crate::evaluate::params::PAWN_STORM_BONUS;
use crate::evaluate::params::KING_ZONE_ATTACKS;

use self::params::PASSERS_ENEMY_KING_PENALTY;
use self::params::PASSERS_FRIENDLY_KING_BONUS;
use self::params::MAJOR_ON_SEVENTH_BONUS;
use self::params::QUEEN_SEMIOPEN_FILE_BONUS;
use self::params::ROOK_SEMIOPEN_FILE_BONUS;
use self::pawn_structure::PawnStructure;

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
    pawn_structure: PawnStructure,

    /// A bonus score for having two bishops on the board
    bishop_pair: S,

    /// A bonus for having a rook on an open file
    rook_open_file: S,

    /// A bonus for having a rook on a semiopen file
    rook_semiopen_file: S,

    /// A bonus for having connected rooks
    connected_rooks: S,

    /// A bonus for rooks on the seventh rank
    major_on_seventh: S,

    /// A bonus for having a queen on an open file
    queen_open_file: S,

    /// A bonus for having a rook on a semiopen file
    queen_semiopen_file: S,

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
            + self.pawn_structure.score()
            + self.bishop_pair
            + self.rook_open_file
            + self.rook_semiopen_file
            + self.major_on_seventh
            + self.queen_open_file
            + self.queen_semiopen_file
            + self.pawn_shield
            + self.pawn_storm
            + self.passers_friendly_king
            + self.passers_enemy_king;

        let mut ctx = EvalContext::new(board);

        total += board.connected_rooks::<WHITE>()
               - board.connected_rooks::<BLACK>()

               + board.mobility::<WHITE>(&mut ctx, &self.pawn_structure)
               - board.mobility::<BLACK>(&mut ctx, &self.pawn_structure)

               + board.virtual_mobility::<WHITE>()
               - board.virtual_mobility::<BLACK>()

               + board.king_zone::<WHITE>(&mut ctx) 
               - board.king_zone::<BLACK>(&mut ctx);

        let score = total.lerp(self.game_phase);

        if board.current.is_white() { score } else { -score }
    }

    /// Update the score by adding a piece to it
    pub fn add(&mut self, piece: Piece, sq: Square, board: &Board) {
        self.game_phase += Self::phase_value(piece);

        self.material += board.material(piece);
        self.psqt += board.psqt(piece, sq);

        self.update_incremental_terms(piece, board)
    }

    /// Update the score by removing a piece from it
    pub fn remove(&mut self, piece: Piece, sq: Square, board: &Board) {
        self.game_phase -= Self::phase_value(piece);

        self.material -= board.material(piece);
        self.psqt -= board.psqt(piece, sq);

        self.update_incremental_terms(piece, board)
    }

    /// Update the score by moving a piece from one square to another
    pub fn update(&mut self, piece: Piece, from: Square, to: Square, board: &Board) {
        self.psqt -= board.psqt(piece, from);
        self.psqt += board.psqt(piece, to);

        self.update_incremental_terms(piece, board)
    }

    fn update_incremental_terms(&mut self, piece: Piece, board: &Board) {
        if piece.is_pawn() {
            self.pawn_structure = PawnStructure::new(board);

            self.pawn_shield = board.pawn_shield::<WHITE>() 
                - board.pawn_shield::<BLACK>();

            self.pawn_storm = board.pawn_storm::<WHITE>()
                - board.pawn_storm::<BLACK>();

            self.rook_open_file = board.rook_open_file::<WHITE>(&self.pawn_structure) 
                - board.rook_open_file::<BLACK>(&self.pawn_structure);

            self.rook_semiopen_file = board.rook_semiopen_file::<WHITE>(&self.pawn_structure)
                - board.rook_semiopen_file::<BLACK>(&self.pawn_structure);

            self.queen_open_file = board.queen_open_file::<WHITE>(&self.pawn_structure) 
                - board.queen_open_file::<BLACK>(&self.pawn_structure);

            self.queen_semiopen_file = board.queen_semiopen_file::<WHITE>(&self.pawn_structure)
                - board.queen_semiopen_file::<BLACK>(&self.pawn_structure);

            self.major_on_seventh = board.major_on_seventh::<WHITE>()
                - board.major_on_seventh::<BLACK>();

            self.passers_friendly_king = board.passers_friendly_king::<WHITE>(&self.pawn_structure)
                - board.passers_friendly_king::<BLACK>(&self.pawn_structure);

            self.passers_enemy_king = board.passers_enemy_king::<WHITE>(&self.pawn_structure)
                - board.passers_enemy_king::<BLACK>(&self.pawn_structure);
        }

        if piece.is_bishop() {
            self.bishop_pair = board.bishop_pair::<WHITE>()
                - board.bishop_pair::<BLACK>();
        }

        if piece.is_rook() {
            self.rook_open_file = board.rook_open_file::<WHITE>(&self.pawn_structure)
                - board.rook_open_file::<BLACK>(&self.pawn_structure);

            self.rook_semiopen_file = board.rook_semiopen_file::<WHITE>(&self.pawn_structure)
                - board.rook_semiopen_file::<BLACK>(&self.pawn_structure);

            self.major_on_seventh = board.major_on_seventh::<WHITE>()
                - board.major_on_seventh::<BLACK>();
        }

        if piece.is_queen() {
            self.queen_open_file = board.queen_open_file::<WHITE>(&self.pawn_structure)
                - board.queen_open_file::<BLACK>(&self.pawn_structure);

            self.queen_semiopen_file = board.queen_semiopen_file::<WHITE>(&self.pawn_structure)
                - board.queen_semiopen_file::<BLACK>(&self.pawn_structure);

            self.major_on_seventh = board.major_on_seventh::<WHITE>()
                - board.major_on_seventh::<BLACK>();
        }

        if piece.is_king() {
            self.pawn_shield = board.pawn_shield::<WHITE>()
                - board.pawn_shield::<BLACK>();

            self.pawn_storm = board.pawn_storm::<WHITE>()
                - board.pawn_storm::<BLACK>();

            self.passers_friendly_king = board.passers_friendly_king::<WHITE>(&self.pawn_structure)
                - board.passers_friendly_king::<BLACK>(&self.pawn_structure);

            self.passers_enemy_king = board.passers_enemy_king::<WHITE>(&self.pawn_structure)
                - board.passers_enemy_king::<BLACK>(&self.pawn_structure);

            self.major_on_seventh = board.major_on_seventh::<WHITE>()
                - board.major_on_seventh::<BLACK>();
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

    fn pawn_shield<const WHITE: bool>(&self) -> S;
    fn pawn_storm<const WHITE: bool>(&self) -> S;
    fn passers_friendly_king<const WHITE: bool>(&self, pawn_structure: &PawnStructure) -> S;
    fn passers_enemy_king<const WHITE: bool>(&self, pawn_structure: &PawnStructure) -> S;

    fn bishop_pair<const WHITE: bool>(&self) -> S;

    fn rook_open_file<const WHITE: bool>(&self, pawn_structure: &PawnStructure) -> S;
    fn rook_semiopen_file<const WHITE: bool>(&self, pawn_structure: &PawnStructure) -> S;
    fn connected_rooks<const WHITE: bool>(&self) -> S;

    fn queen_open_file<const WHITE: bool>(&self, pawn_structure: &PawnStructure) -> S;
    fn queen_semiopen_file<const WHITE: bool>(&self, pawn_structure: &PawnStructure) -> S;

    fn major_on_seventh<const WHITE: bool>(&self) -> S;

    fn virtual_mobility<const WHITE: bool>(&self) -> S;
    fn king_zone<const WHITE: bool>(&self, ctx: &mut EvalContext) -> S;

    fn mobility<const WHITE: bool>(&self, ctx: &mut EvalContext, pawn_structure: &PawnStructure) -> S;
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

    fn passers_friendly_king<const WHITE: bool>(&self, pawn_structure: &PawnStructure) -> S {
        let mut total = S::default();

        let us = if WHITE { White } else { Black };
        let our_king = self.kings(us).first();

        for passer in pawn_structure.passed_pawns(us) {
            let distance = passer.max_dist(our_king);
            total += PASSERS_FRIENDLY_KING_BONUS[distance - 1];
        }

        total
    }

    fn passers_enemy_king<const WHITE: bool>(&self, pawn_structure: &PawnStructure) -> S {
        let mut total = S::default();

        let us = if WHITE { White } else { Black };
        let their_king = self.kings(!us).first();

        for passer in pawn_structure.passed_pawns(us) {
            let distance = passer.max_dist(their_king);
            total += PASSERS_ENEMY_KING_PENALTY[distance - 1];
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

    fn rook_open_file<const WHITE: bool>(&self, pawn_structure: &PawnStructure) -> S {
        let us = if WHITE { White } else { Black };
        let rooks_on_open = self.rooks(us) & pawn_structure.open_files();
        ROOK_OPEN_FILE_BONUS * rooks_on_open.count() as i32
    }

    fn rook_semiopen_file<const WHITE: bool>(&self, pawn_structure: &PawnStructure) -> S {
        let us = if WHITE { White } else { Black };
        let rooks_on_semi = self.rooks(us) & pawn_structure.semi_open_files(us);
        ROOK_SEMIOPEN_FILE_BONUS * rooks_on_semi.count() as i32
    }

    fn connected_rooks<const WHITE: bool>(&self) -> S {
        let mut total = S::default();
        let us = if WHITE { White } else { Black };

        let mut rooks = self.rooks(us);
        let back_rank = if WHITE { 0 } else { 7 };

        if let Some(first) = rooks.next() {
            if let Some(second) = rooks.next() {
                let on_back_rank = first.rank() == back_rank && second.rank() == back_rank;
                let connected = BETWEEN[first as usize][second as usize] & self.all_occupied() == Bitboard::EMPTY;

                if on_back_rank && connected {
                    total += CONNECTED_ROOKS_BONUS;
                }
            }
        }

        total
    }

    fn major_on_seventh<const WHITE: bool>(&self) -> S {
        let mut total = S::default();
        let us = if WHITE { White } else { Black };
        let seventh_rank = if WHITE { RANKS[6] } else { RANKS[1] };
        let eighth_rank = if WHITE { RANKS[7] } else { RANKS[0] };

        let pawns_on_seventh = !(self.pawns(!us) & seventh_rank).is_empty();
        let king_on_eighth = !(self.kings(!us) & eighth_rank).is_empty();
        let majors = self.rooks(us) | self.queens(us);

        if pawns_on_seventh || king_on_eighth {
            total += MAJOR_ON_SEVENTH_BONUS * (majors & seventh_rank).count() as i32;

        }

        total
    }

    fn queen_open_file<const WHITE: bool>(&self, pawn_structure: &PawnStructure) -> S {
        let us = if WHITE { White } else { Black };
        let queens_on_open = self.queens(us) & pawn_structure.open_files();
        QUEEN_OPEN_FILE_BONUS * queens_on_open.count() as i32
    }

    fn queen_semiopen_file<const WHITE: bool>(&self, pawn_structure: &PawnStructure) -> S {
        let us = if WHITE { White } else { Black };
        let queens_on_semi = self.queens(us) 
            & pawn_structure.semi_open_files(us)
            & !pawn_structure.open_files();
        QUEEN_SEMIOPEN_FILE_BONUS * queens_on_semi.count() as i32
    }

    fn mobility<const WHITE: bool>(&self, ctx: &mut EvalContext, pawn_structure: &PawnStructure) -> S {
        let mut total = S::default();

        let us = if WHITE { White } else { Black };
        let blockers = self.all_occupied();
        let enemy_king_zone = ctx.king_zones[!us as usize];

        let pawn_attacks = pawn_structure.pawn_attacks(!us);
        let blocked_pawns = pawn_structure.blocked_pawns(us);

        let mobility_squares = !pawn_attacks & !blocked_pawns;

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
// Utility helpers for our custom tapered scores
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
// Extension trait that allows us to put some additional helper methods on 
// the Score type alias (recall, Score is just an alias for i32).
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
