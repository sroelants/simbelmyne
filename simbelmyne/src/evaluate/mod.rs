//! Assign a static score to a gven board position
//!
//! Since it's impractical to search the entire game tree till the end and see
//! who wins, we have to cut the search short at some point and assign a score
//! to the current state of the board.
//!
//! ## Incremental and volatile evaluation terms
//! The evaluation terms fall into two categories:
//!
//! 1. We try to update as much of the evaluation as possible incrementally.
//! To that end, we keep around the individual terms that make up the 
//! (incremental part of the) evaluation. For example, if a bishop is moved,
//! we only recompute the terms that involve bishops, rather than recomputing
//! things like pawn structure terms.
//!
//! 2. Some terms simply can't be updated incrementally very easily. Terms where
//! one piece moving might impact the contribution of all other pieces 
//! (mobility, threats, etc...). These terms are just computed on the fly
//! whenever the total eval is requested.
//!
//! ## Tapered evaluation
//! Each evaluation term actually corresponds to two values: a midgame score and
//! an endgame score. For any given board position, we estimate the progress of
//! the game by the remaining material, and lerp between the two eval scores.

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
use crate::s;

use bytemuck::Pod;
use bytemuck::Zeroable;
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
use tuner::EvalTrace;

use self::lookups::PASSED_PAWN_MASKS;
use self::piece_square_tables::PIECE_SQUARE_TABLES;
use self::params::CONNECTED_ROOKS_BONUS;
use self::params::QUEEN_OPEN_FILE_BONUS;
use self::params::BISHOP_MOBILITY_BONUS;
use self::params::BISHOP_PAIR_BONUS;
use self::params::KNIGHT_MOBILITY_BONUS;
use self::params::QUEEN_MOBILITY_BONUS;
use self::params::ROOK_MOBILITY_BONUS;
use self::params::ROOK_OPEN_FILE_BONUS;
use self::params::PIECE_VALUES;
use self::params::PAWN_SHIELD_BONUS;
use self::params::VIRTUAL_MOBILITY_PENALTY;
use self::params::PAWN_STORM_BONUS;
use self::params::KING_ZONE_ATTACKS;
use self::params::BISHOP_OUTPOSTS;
use self::params::KNIGHT_OUTPOSTS;
use self::params::MINOR_ATTACKS_ON_QUEENS;
use self::params::MINOR_ATTACKS_ON_ROOKS;
use self::params::PASSERS_ENEMY_KING_PENALTY;
use self::params::PASSERS_FRIENDLY_KING_BONUS;
use self::params::MAJOR_ON_SEVENTH_BONUS;
use self::params::PAWN_ATTACKS_ON_MINORS;
use self::params::PAWN_ATTACKS_ON_QUEENS;
use self::params::PAWN_ATTACKS_ON_ROOKS;
use self::params::QUEEN_SEMIOPEN_FILE_BONUS;
use self::params::ROOK_ATTACKS_ON_QUEENS;
use self::params::ROOK_SEMIOPEN_FILE_BONUS;
use self::params::TEMPO_BONUS;
use self::pawn_structure::PawnStructure;

pub type Score = i32;

// Helper consts to make generic parameters more readable.
const WHITE: bool = true;
const BLACK: bool = false;

////////////////////////////////////////////////////////////////////////////////
//
// Evaluation logic
//
////////////////////////////////////////////////////////////////////////////////

/// An `Evaluation` keeps track of the granular score breakdown of incremental
/// terms.
///
/// Keep track of both midgame and endgame scores for a given position, as well
/// as the "game_phase" parameter. Keeping track of these independently
/// means we can incrementally update the score by adding/removing pieces as the
/// game progresses.
///
/// All of the scores are stored as relative to White, and are only converted to
/// the STM-relative value when `Eval::total()` is called.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct Eval {
    /// Value between 0 and 24, keeping track of how far along the game we are.
    /// A score of 0 corresponds to endgame, a score of 24 is in the opening.
    game_phase: u8,

    /// The total material score, based on the piece values. 
    /// See [Board::material] for implementation
    material: S,

    /// The total positional score, based on the piece and occupied square
    /// See [Board::psqt] for implementation
    psqt: S,

    /// The total pawn structure score
    /// See [PawnStructure] for implementation
    pawn_structure: PawnStructure,

    /// A bonus score for having two bishops on the board
    /// See [Board::bishop_pair] for implementation
    bishop_pair: S,

    /// A bonus for having a rook on an open file
    /// See [Board::rook_open_file] for implementation
    rook_open_file: S,

    /// A bonus for having a rook on a semiopen file
    /// See [Board::rook_semiopen_file] for implementation
    rook_semiopen_file: S,

    /// A bonus for rooks on the seventh rank
    /// See [Board::major_on_seventh] for implementation
    major_on_seventh: S,

    /// A bonus for having a queen on an open file
    /// See [Board::queen_open_file] for implementation
    queen_open_file: S,

    /// A bonus for having a rook on a semiopen file
    /// See [Board::queen_semiopen_file] for implementation
    queen_semiopen_file: S,

    /// A bonus for having pawns protecting the king
    /// See [Board::pawn_shield] for implementation
    pawn_shield: S,
    
    /// A bonus for having pawns attacking the enemy king
    /// See [Board::pawn_storm] for implementation
    pawn_storm: S,

    /// A bonus for keeping the king near friendly passed pawns
    /// See [Board::passers_friendly_king] for implementation
    passers_friendly_king: S,

    /// A bonus for keeping the king near enemy passed pawns
    /// See [Board::passers_enemy_king] for implementation
    passers_enemy_king: S,

    /// A bonus for having a knight on an outpost square
    /// See [Board::knight_outposts] for implementation
    knight_outposts: S,

    /// A bonus for having a bishop on an outpost square
    /// See [Board::bishop_outposts] for implementation
    bishop_outposts: S,

    #[cfg(feature = "texel")]
    trace: EvalTrace
}

impl Eval {
    /// A static score that is returned as a draw score.
    /// A positive contempt would make the engine more likely to draw, a 
    /// negative contempt makes it less likely to settle for a draw.
    ///
    /// We don't draw. We go for the kill.
    const CONTEMPT: S = s!(-50, -10);

    /// Create a new score for a board
    /// TODO: Make this more efficient? By running over every single term
    /// exactly once. Then we could re-use this to trace, right?
    pub fn new(board: &Board) -> Self {
        let mut eval = Self::default();
        eval.pawn_structure = PawnStructure::new(board);

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

    /// Return the total (tapered) score for the position as the sum of the
    /// incremental evaluation terms and the volatile terms.
    pub fn total(&self, board: &Board) -> Score {
        // Add up all of the incremental terms stored on the Eval struct
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
            + self.passers_enemy_king
            + self.knight_outposts
            + self.bishop_outposts;

        // We pass around an EvalContext so expensive information gathered in 
        // some evaluation terms can be shared with other eval terms, instead
        // of recomputing them again.
        let mut ctx = EvalContext::new(board);


        // Compute and add up the "volatile" evaluation terms. These are the 
        // terms that need to get recomputed in every node, anyway.
        total += connected_rooks::<WHITE>(board, None)
               - connected_rooks::<BLACK>(board, None)

               + mobility::<WHITE>(board, &self.pawn_structure, &mut ctx, None)
               - mobility::<BLACK>(board, &self.pawn_structure, &mut ctx, None)

               + virtual_mobility::<WHITE>(board, None)
               - virtual_mobility::<BLACK>(board, None)

               + king_zone::<WHITE>(&mut ctx, None) 
               - king_zone::<BLACK>(&mut ctx, None)

               + threats::<WHITE>(&ctx, None)
               - threats::<BLACK>(&ctx, None);

        // Add a side-relative tempo bonus
        // The position should be considered slightly more advantageous for the
        // current side-to-move.
        if board.current.is_white() { 
            total += TEMPO_BONUS 
        } else { 
            total -= TEMPO_BONUS 
        };

        // Interpolate between midgame and endgame evals
        let score = total.lerp(self.game_phase);

        // Return the score relative to the current side-to-move
        if board.current.is_white() { score } else { -score }
    }

    /// Update the Eval by adding a piece to it
    pub fn add(&mut self, piece: Piece, sq: Square, board: &Board) {
        self.game_phase += Self::phase_value(piece);
        let material = material(piece, None);
        let psqt = psqt(piece, sq, None);

        self.material += material;
        self.psqt += psqt;

        self.update_incremental_terms(piece, board)
    }

    /// Update the score by removing a piece from it
    pub fn remove(&mut self, piece: Piece, sq: Square, board: &Board) {
        self.game_phase -= Self::phase_value(piece);
        let material = material(piece, None);
        let psqt = psqt(piece, sq, None);

        self.material -= material;
        self.psqt -= psqt;

        self.update_incremental_terms(piece, board)
    }

    /// Update the score by moving a piece from one square to another
    ///
    /// Slightly more efficient helper that does less work than calling 
    /// `Eval::remove` and `Eval::add` in succession.
    pub fn update(&mut self, piece: Piece, from: Square, to: Square, board: &Board) {
        let from_psqt = psqt(piece, from, None);
        let to_psqt = psqt(piece, to, None);
        // If the piece remains on the board, we only update the PSQT score. 
        // There is no need to update the material score.
        self.psqt -= from_psqt;
        self.psqt += to_psqt;

        self.update_incremental_terms(piece, board)
    }

    /// Update the incremental eval terms, according to piece that moved.
    ///
    /// This tries to save as much work as possible, by only recomputing eval
    /// terms that depend on the moved piece. No need to update rook-related
    /// terms when a bishop has moved.
    fn update_incremental_terms(&mut self, piece: Piece, board: &Board) {
        use PieceType::*;

        match piece.piece_type() {
            // Pawn moves require almost _all_ terms, save a couple, to be 
            // updated.
            Pawn => {
                self.pawn_structure = PawnStructure::new(board);

                self.pawn_shield = pawn_shield::<WHITE>(board, None) 
                    - pawn_shield::<BLACK>(board, None);

                self.pawn_storm = pawn_storm::<WHITE>(board, None)
                    - pawn_storm::<BLACK>(board, None);

                self.rook_open_file = rook_open_file::<WHITE>(board, &self.pawn_structure, None) 
                    - rook_open_file::<BLACK>(board, &self.pawn_structure, None);

                self.rook_semiopen_file = rook_semiopen_file::<WHITE>(board, &self.pawn_structure, None)
                    - rook_semiopen_file::<BLACK>(board, &self.pawn_structure, None);

                self.queen_open_file = queen_open_file::<WHITE>(board, &self.pawn_structure, None) 
                    - queen_open_file::<BLACK>(board, &self.pawn_structure, None);

                self.queen_semiopen_file = queen_semiopen_file::<WHITE>(board, &self.pawn_structure, None)
                    - queen_semiopen_file::<BLACK>(board, &self.pawn_structure, None);

                self.major_on_seventh = major_on_seventh::<WHITE>(board, None)
                    - major_on_seventh::<BLACK>(board, None);

                self.passers_friendly_king = passers_friendly_king::<WHITE>(board, &self.pawn_structure, None)
                    - passers_friendly_king::<BLACK>(board, &self.pawn_structure, None);

                self.passers_enemy_king = passers_enemy_king::<WHITE>(board, &self.pawn_structure, None)
                    - passers_enemy_king::<BLACK>(board, &self.pawn_structure, None);

                self.knight_outposts = knight_outposts::<WHITE>(board, &self.pawn_structure, None)
                    - knight_outposts::<BLACK>(board, &self.pawn_structure, None);

                self.bishop_outposts = bishop_outposts::<WHITE>(board, &self.pawn_structure, None)
                    - bishop_outposts::<BLACK>(board, &self.pawn_structure, None);
            },

            Knight => {
                self.knight_outposts = knight_outposts::<WHITE>(board, &self.pawn_structure, None)
                    - knight_outposts::<BLACK>(board, &self.pawn_structure, None);
            },

            Bishop => {
                self.bishop_pair = bishop_pair::<WHITE>(board, None)
                    - bishop_pair::<BLACK>(board, None);

                self.bishop_outposts = bishop_outposts::<WHITE>(board, &self.pawn_structure, None)
                    - bishop_outposts::<BLACK>(board, &self.pawn_structure, None);
            },

            Rook => {
                self.rook_open_file = rook_open_file::<WHITE>(board, &self.pawn_structure, None)
                    - rook_open_file::<BLACK>(board, &self.pawn_structure, None);

                self.rook_semiopen_file = rook_semiopen_file::<WHITE>(board, &self.pawn_structure, None)
                    - rook_semiopen_file::<BLACK>(board, &self.pawn_structure, None);

                self.major_on_seventh = major_on_seventh::<WHITE>(board, None)
                    - major_on_seventh::<BLACK>(board, None);
            },

            Queen => {
                self.queen_open_file = queen_open_file::<WHITE>(board, &self.pawn_structure, None)
                    - queen_open_file::<BLACK>(board, &self.pawn_structure, None);

                self.queen_semiopen_file = queen_semiopen_file::<WHITE>(board, &self.pawn_structure, None)
                    - queen_semiopen_file::<BLACK>(board, &self.pawn_structure, None);

                self.major_on_seventh = major_on_seventh::<WHITE>(board, None)
                    - major_on_seventh::<BLACK>(board, None);
            },

            King => {
                self.pawn_shield = pawn_shield::<WHITE>(board, None)
                    - pawn_shield::<BLACK>(board, None);

                self.pawn_storm = pawn_storm::<WHITE>(board, None)
                    - pawn_storm::<BLACK>(board, None);

                self.passers_friendly_king = passers_friendly_king::<WHITE>(board, &self.pawn_structure, None)
                    - passers_friendly_king::<BLACK>(board, &self.pawn_structure, None);

                self.passers_enemy_king = passers_enemy_king::<WHITE>(board, &self.pawn_structure, None)
                    - passers_enemy_king::<BLACK>(board, &self.pawn_structure, None);

                self.major_on_seventh = major_on_seventh::<WHITE>(board, None)
                    - major_on_seventh::<BLACK>(board, None);
            },
        }
    }

    /// Values assignd to each piece type to calculate the approximate stage 
    /// of the game
    const GAME_PHASE_VALUES: [u8; PieceType::COUNT] = [0, 1, 1, 2, 4, 0];

    /// Return the game phase as a value between 0 and 24. 
    ///
    /// 0 corresponds to endgame, 24 corresponds to midgame
    fn phase_value(piece: Piece) -> u8 {
        Self::GAME_PHASE_VALUES[piece.piece_type()]
    }

    /// Return the draw score, taking into account the global contempt factor
    pub fn draw_score(self, ply: usize, nodes: u32) -> Score {
        let random = nodes as Score & 0b11 - 2;

        // Make sure to make the returned contempt relative to the side-to-move
        // at root.
        //
        // We add a small random contribution to help with repetitions
        if ply % 2 == 0 {
            Self::CONTEMPT.lerp(self.game_phase) + random
        } else {
            -(Self::CONTEMPT.lerp(self.game_phase) + random)
        }
    }
}

/// Helper struct that we use to share gathered information between eval terms, 
/// in order to save us from having to recompute them again.
///
/// (Yes, we could avoid this by throwing everything into one big function. No,
/// I don't want to do that.)
struct EvalContext {
    /// The 9x9 area surrounding each king, indexed by the king's color
    king_zones: [Bitboard; Color::COUNT],

    /// The number of attacks on each side's king zone, indexed by the side
    /// whose king zone is attacked.
    king_attacks: [u32; Color::COUNT],

    /// The number of attacks by pawns on minor pieces (bishops and knights),
    /// indexed by the side doing the attacking.
    pawn_attacks_on_minors: [u8; Color::COUNT],

    /// The number of attacks by pawns on rooks, indexed by the side doing the
    /// attacking
    pawn_attacks_on_rooks: [u8; Color::COUNT],

    /// The number of attacks by pawns on queens, indexed by the side doing the
    /// attacking
    pawn_attacks_on_queens: [u8; Color::COUNT],

    /// The number of attacks by minor pieces (bishops and knights) on rooks,
    /// indexed by the side  doing the attacking
    minor_attacks_on_rooks: [u8; Color::COUNT],

    /// The number of attacks by minor pieces (bishops and knights) on queens,
    /// indexed by the side  doing the attacking
    minor_attacks_on_queens: [u8; Color::COUNT],

    /// The number of attacks by rooks on queens, indexed by the side doing
    /// the attacking
    rook_attacks_on_queens: [u8; Color::COUNT],
}

impl EvalContext {
    /// Create a new EvalContext
    pub fn new(board: &Board) -> Self {
        let white_king = board.kings(Color::White).first();
        let black_king = board.kings(Color::Black).first();

        let white_king_zone = white_king.king_squares();
        let black_king_zone = black_king.king_squares();

        Self {
            king_zones: [white_king_zone, black_king_zone],
            king_attacks: [0, 0],
            pawn_attacks_on_minors: [0, 0],
            pawn_attacks_on_rooks: [0, 0],
            pawn_attacks_on_queens: [0, 0],
            minor_attacks_on_rooks: [0, 0],
            minor_attacks_on_queens: [0, 0],
            rook_attacks_on_queens: [0, 0],
        }
    }
}

impl Eval {
}

////////////////////////////////////////////////////////////////////////////////
//
// Packed scores
//
/// Scores are made sure to fit within an i16, and we pack both of them into an
/// 132. This means we can do a poor man's version of SIMD and perform all of 
/// the operations on midgame/endgame scores in single instructions.
///
////////////////////////////////////////////////////////////////////////////////

/// A wrapper that stores a midgame and endgame score
///
/// Scores are made sure to fit within an i16, and we pack both of them into an
/// 132. This means we can do a poor man's version of SIMD and perform all of 
/// the operations on midgame/endgame scores in single instructions.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Pod, Zeroable)]
#[repr(C)]
pub struct S(i32);

// Utility macro that saves us some space when working with many scores at once
// (see [./params.rs]).
#[macro_export]
macro_rules! s {
    ($mg:literal, $eg:literal) => {
        S::new($mg, $eg)
    };
}

impl S {
    /// Create a new packed score.
    pub const fn new(mg: Score, eg: Score) -> Self {
        Self((eg << 16).wrapping_add(mg))
    }

    /// Extract the midgame score from the packed score
    pub fn mg(&self) -> Score {
        self.0 as i16 as Score
    }

    /// Extract the endgame score from the packed score.
    pub fn eg(&self) -> Score {
        ((self.0 + 0x8000) >> 16 as i16) as Score
    }

    /// Interpolate between the midgame and endgame score according to a
    /// given `phase` which is a value between 0 and 24.
    pub fn lerp(&self, phase: u8) -> Score {
        (phase as Score * self.mg() + (24 - phase as Score) * self.eg()) / 24 
    }
}

// Utility traits for the packed score, that allow us to use arithmetic
// operations transparently.

impl Add for S {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for S {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Sub for S {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl SubAssign for S {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs
    }
}

impl Mul<Score> for S {
    type Output = Self;

    fn mul(self, rhs: Score) -> Self::Output {
        Self(self.0 * rhs)
    }
}

impl Neg for S {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new(-self.mg(), -self.eg())
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
// A `Score` is just a type alias for an i32. This means we can't  really add
// any methods on `Score`s. (because of Rust's orphan rules)
//
// Instead, we define an extension trait that allows us to put some additional 
// helper methods on the Score type alias.
//
////////////////////////////////////////////////////////////////////////////////

pub trait ScoreExt {
    const MINUS_INF: Self;
    const PLUS_INF: Self;
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
// Eval terms
//
////////////////////////////////////////////////////////////////////////////////

/// An evaluation score for having a specific piece on the board.
///
/// This more or less corresponds to the classic heuristic values of
/// 100 (Pawn), 300 (Knight), 300 (Bishop), 500 (Rook), 900 (Queen),
/// but the values are tuned. 
///
/// The distinction between midgame and engame values means we can be more 
/// granular. E.g., a bishop is worth more in the endgame than a knight, 
/// rooks become more valuable in the endgame, etc...
fn material(piece: Piece, trace: Option<&mut EvalTrace>) -> S {
    if let Some(trace) = trace {
        if piece.color().is_white() {
            trace.piece_values[piece.piece_type()] += 1;
        } else {
            trace.piece_values[piece.piece_type()] -= 1;
        }
    }

    if piece.color().is_white() {
        PIECE_VALUES[piece.piece_type()]
    } else {
        -PIECE_VALUES[piece.piece_type()]
    }
}

/// A positional score for each piece and the square it resides on,
/// as determined by piece-specific "piece-square tables" (PSQTs).
///
/// This captures a ton of different heuristics
/// - The king should hide in the midgame, but come out in the endgame
/// - Pawns should be pushed, especially in the endgame
/// - Controlling the center
/// - ...
///
/// The tables are stored from black's perspective (so they read easier
/// in text), so in order to get the correct value for White, we need to
/// artificially mirror the square vertically.
fn psqt(piece: Piece, sq: Square, trace: Option<&mut EvalTrace>) -> S {
    let sq = if piece.color().is_white() { sq.flip() } else { sq };

    if let Some(trace) = trace {
        use PieceType::*;
        let term = if piece.color().is_white() { 1 } else { -1 };
        match piece.piece_type() {
            Pawn   => trace.pawn_psqt[sq]   += term,
            Knight => trace.knight_psqt[sq] += term,
            Bishop => trace.bishop_psqt[sq] += term,
            Rook   => trace.rook_psqt[sq]   += term,
            Queen  => trace.queen_psqt[sq]  += term,
            King   => trace.king_psqt[sq]   += term,
        };
    }

    if piece.color().is_white() {
        PIECE_SQUARE_TABLES[piece.piece_type()][sq.flip()]
    } else {
        -PIECE_SQUARE_TABLES[piece.piece_type()][sq]
    }
}

/// A score for pawns protecting the squares directly in front of the 
/// friendly king.
///
/// Assign a flat bonus for every pawn that is
/// - on the three files surrounding the king,
/// - 1 or 2 ranks in front of the king
///
/// We assign different bonuses depending on how far the shield pawn is 
/// removed from the king.
fn pawn_shield<const WHITE: bool>(board: &Board, mut trace: Option<&mut EvalTrace>) -> S {
    let mut total = S::default();

    let us = if WHITE { White } else { Black };
    let our_king = board.kings(us).first();
    let our_pawns = board.pawns(us);

    // Use the passed pawn masks to mask the squares in front of the king.
    let shield_mask = PASSED_PAWN_MASKS[us][our_king];
    let shield_pawns = shield_mask & our_pawns;

    for pawn in shield_pawns {
        // Get the (vertical) distance from the king, clamped to [1, 2],
        // and use it to assign the associated bonus.
        let distance = pawn.vdistance(our_king).min(3) - 1;
        total += PAWN_SHIELD_BONUS[distance];

        if let Some(ref mut trace) = trace {
            trace.pawn_shield[distance] += if WHITE { 1 } else { -1 };
        }
    }

    total
}

// A score for pawns approaching the squares directly in front of the enemy
// king.
//
/// Assign a flat bonus for every pawn that is
/// - on the three files surrounding the king,
/// - 1 or 2 ranks in front of the king
///
/// We assign different bonuses depending on how far the shield pawn is 
/// removed from the king.
fn pawn_storm<const WHITE: bool>(board: &Board, mut trace: Option<&mut EvalTrace>) -> S {
    let mut total = S::default();

    let us = if WHITE { White } else { Black };
    let them = !us;
    let their_king = board.kings(them).first();
    let our_pawns = board.pawns(us);

    // Use the passed pawn masks to mask the squares in front of the king.
    let storm_mask = PASSED_PAWN_MASKS[them][their_king];
    let storm_pawns = storm_mask & our_pawns;

    for pawn in storm_pawns {
        // Get the (vertical) distance from the king, clamped to [1, 2],
        // and use it to assign the associated bonus.
        let distance = pawn.vdistance(their_king).min(3) - 1;
        total += PAWN_STORM_BONUS[distance];

        if let Some(ref mut trace) = trace  {
            trace.pawn_storm[distance] += if WHITE { 1 } else { -1 };
        }
    }

    total
}

/// A score for keeping the king close to friendly passed powns, in order to
/// protect them.
///
/// For every passed pawn, we assign a bonus dependent on how far away they
/// are from the friendly king. The distance is measured using the Chebyshev
/// (infinity-, or max-) norm.
fn passers_friendly_king<const WHITE: bool>(board: &Board, pawn_structure: &PawnStructure, mut trace: Option<&mut EvalTrace>) -> S {
    let mut total = S::default();

    let us = if WHITE { White } else { Black };
    let our_king = board.kings(us).first();

    for passer in pawn_structure.passed_pawns(us) {
        // Get the L_inf distance from the king, and use it to assign the 
        // associated bonus
        let distance = passer.max_dist(our_king);
        total += PASSERS_FRIENDLY_KING_BONUS[distance - 1];

        if let Some(ref mut trace) = trace  {
            trace.passers_friendly_king[distance - 1] += if WHITE { 1 } else { -1 };
        }
    }

    total
}

/// A penalty for having passers too close to the enemy king.
///
/// For every passed pawn, we assign a penalty dependent on how close they
/// are from the enemy king. The distance is measured using the Chebyshev
/// (infinity-, or max-) norm.
fn passers_enemy_king<const WHITE: bool>(board: &Board, pawn_structure: &PawnStructure, mut trace: Option<&mut EvalTrace>) -> S {
    let mut total = S::default();

    let us = if WHITE { White } else { Black };
    let their_king = board.kings(!us).first();

    for passer in pawn_structure.passed_pawns(us) {
        // Get the L_inf distance from the king, and use it to assign the 
        // associated bonus
        let distance = passer.max_dist(their_king);
        total += PASSERS_ENEMY_KING_PENALTY[distance - 1];

        if let Some(ref mut trace) = trace  {
            trace.passers_enemy_king[distance - 1] += if WHITE { 1 } else { -1 };
        }
    }

    total
}

/// A bonus for knights that are positioned on outpost squares.
///
/// Outpost squares are squares that cannot easily be attacked by pawns,
/// and are defended by one of our own pawns.
///
/// For the implementation of outpost squares, see [PawnStructure::new].
fn knight_outposts<const WHITE: bool>(board: &Board, pawn_structure: &PawnStructure, trace: Option<&mut EvalTrace>) -> S {
    let us = if WHITE { White } else { Black };
    let outpost_knights = board.knights(us) & pawn_structure.outposts(us);
    let count = outpost_knights.count() as i32;

    if let Some(trace) = trace  {
        trace.knight_outposts += if WHITE { count } else { -count };
    }

    KNIGHT_OUTPOSTS * count
}

/// A bonus for bishops that are positioned on outpost squares.
///
/// Outpost squares are squares that cannot easily be attacked by pawns,
/// and are defended by one of our own pawns.
///
/// For the implementation of outpost squares, see [PawnStructure::new].
fn bishop_outposts<const WHITE: bool>(board: &Board, pawn_structure: &PawnStructure, trace: Option<&mut EvalTrace>) -> S {
    let us = if WHITE { White } else { Black };
    let outpost_bishops = board.bishops(us) & pawn_structure.outposts(us);
    let count = outpost_bishops.count() as i32;

    if let Some(trace) = trace  {
        trace.bishop_outposts += if WHITE { count } else { -count };
    }

    BISHOP_OUTPOSTS * count
}

/// A bonus for having a bishop pair on opposite colored squares.
///
/// This does not actually check the square colors, and just assumes that if
/// the player has two bishops, they are opposite colored (rather than, say,
/// two same-color bishops through a promotion)
fn bishop_pair<const WHITE: bool>(board: &Board, trace: Option<&mut EvalTrace>) -> S {
    let us = if WHITE { White } else { Black };

    if board.bishops(us).count() == 2 {
        if let Some(trace) = trace  {
            trace.bishop_pair += if WHITE { 1 } else { -1 };
        }

        BISHOP_PAIR_BONUS
    } else {
        S::default()
    }
}

/// A bonus for having a rook on an open file
///
/// Open files are files that have no pawns on them, and allow the rook to
/// move freely along them without pawns blocking them in.
///
/// For the implementation of open files, see [PawnStructure].
fn rook_open_file<const WHITE: bool>(board: &Board, pawn_structure: &PawnStructure, trace: Option<&mut EvalTrace>) -> S {
    let us = if WHITE { White } else { Black };
    let rooks_on_open = board.rooks(us) & pawn_structure.open_files();
    let count = rooks_on_open.count() as i32;

    if let Some(trace) = trace  {
        trace.rook_open_file += if WHITE { count } else { -count };
    }

    ROOK_OPEN_FILE_BONUS * count
}

/// A bonus for having a rook on a semi-open file
///
/// Semi-open files are files that have no friendly pawns on them, but do
/// have enemy pawns on them. They allow rooks to move somewhat freely,
/// since they aren't blocked by any friendly pawns.
///
/// For the implementation of semi-open files, see [PawnStructure].
fn rook_semiopen_file<const WHITE: bool>(board: &Board, pawn_structure: &PawnStructure, trace: Option<&mut EvalTrace>) -> S {
    let us = if WHITE { White } else { Black };
    let rooks_on_semi = board.rooks(us) & pawn_structure.semi_open_files(us);
    let count = rooks_on_semi.count() as i32;

    if let Some(trace) = trace  {
        trace.rook_semiopen_file += if WHITE { count } else { -count };
    }

    ROOK_SEMIOPEN_FILE_BONUS * count
}

/// A bonus for having connected rooks on the back rank.
///
/// Two rooks count as connected when they are withing direct line-of-sight
/// of each other and are protecting one another.
fn connected_rooks<const WHITE: bool>(board: &Board, trace: Option<&mut EvalTrace>) -> S {
    let mut total = S::default();
    let us = if WHITE { White } else { Black };

    let mut rooks = board.rooks(us);
    let back_rank = if WHITE { 0 } else { 7 };

    if let Some(first) = rooks.next() {
        if let Some(second) = rooks.next() {
            let on_back_rank = first.rank() == back_rank && second.rank() == back_rank;
            let connected = BETWEEN[first][second] & board.all_occupied() == Bitboard::EMPTY;

            if on_back_rank && connected {
                if let Some(trace) = trace  {
                    trace.connected_rooks += if WHITE { 1 } else { -1 };
                }

                total += CONNECTED_ROOKS_BONUS;
            }
        }
    }

    total
}

/// A bonus for having a major piece (rook or queen) on the 7th/2nd rank.
///
/// The idea is that these are powerful pieces on the 7th rank, because 
/// they can trap the king on the 8th rank, and attack weak pawns on the 7th
/// rank.
///
/// As such, the terms assigns a bonus _only if_ the king is on the 8th rank
/// or there are powns on the 7th.
fn major_on_seventh<const WHITE: bool>(board: &Board, trace: Option<&mut EvalTrace>) -> S {
    let mut total = S::default();
    let us = if WHITE { White } else { Black };
    let seventh_rank = if WHITE { RANKS[6] } else { RANKS[1] };
    let eighth_rank = if WHITE { RANKS[7] } else { RANKS[0] };
    let pawns_on_seventh = !(board.pawns(!us) & seventh_rank).is_empty();
    let king_on_eighth = !(board.kings(!us) & eighth_rank).is_empty();
    let majors = board.rooks(us) | board.queens(us);

    if pawns_on_seventh || king_on_eighth {
        let count = (majors & seventh_rank).count() as i32;

        if let Some(trace) = trace  {
            trace.major_on_seventh += if WHITE { count } else { -count };
        }

        total += MAJOR_ON_SEVENTH_BONUS * count;
    }

    total
}

/// A bonus for having a queen on an open file.
///
/// Identical in spirit and implementation to [Board::rook_open_file]
fn queen_open_file<const WHITE: bool>(board: &Board, pawn_structure: &PawnStructure, trace: Option<&mut EvalTrace>) -> S {
    let us = if WHITE { White } else { Black };
    let queens_on_open = board.queens(us) & pawn_structure.open_files();
    let count = queens_on_open.count() as i32;

    if let Some(trace) = trace  {
        trace.queen_open_file += if WHITE { count } else { -count };
    }

    QUEEN_OPEN_FILE_BONUS * count
}

/// A bonus for having a queen on a semi-open file.
///
/// Identical in spirit and implementation to [Board::rook_semiopen_file]
fn queen_semiopen_file<const WHITE: bool>(board: &Board, pawn_structure: &PawnStructure, trace: Option<&mut EvalTrace>) -> S {
    let us = if WHITE { White } else { Black };
    let queens_on_semi = board.queens(us) 
        & pawn_structure.semi_open_files(us)
        & !pawn_structure.open_files();
    let count = queens_on_semi.count() as i32;

    if let Some(trace) = trace  {
        trace.queen_semiopen_file += if WHITE { count } else { -count };
    }

    QUEEN_SEMIOPEN_FILE_BONUS * count
}

/// A score associated with how many squares a piece can move to.
/// 
/// This tries to take into account some extra considerations:
/// 1. Disregard squares attacked by pawns
/// 2. Disregard squares occupied by blocked pawns
/// 3. Disregard squares not on the pinray when the piece is pinned
///
/// NOTE: Because this function relies on generating attacked squares for 
/// every single piece on the board, it is rather expensive. That's why we 
/// also make it responsible for gathering relevant information derived from 
/// the attacks to share with other evaluation terms.
/// I kinda hate this, and it makes the order in which we evaluate the 
/// individual eval terms important, which feels gross.
/// FIXME: I'm pretty sure the blocked pawns thing is irrelevant?
/// It's only relevant if I were to consider xray attacks, but then a lot 
/// of the other calculated stuff (threats, king zone) would be invalid?
fn mobility<const WHITE: bool>(board: &Board, pawn_structure: &PawnStructure, ctx: &mut EvalContext, mut trace: Option<&mut EvalTrace>) -> S {
    let mut total = S::default();

    let us = if WHITE { White } else { Black };

    let their_minors = board.knights(!us) | board.bishops(!us);
    let their_rooks = board.rooks(!us);
    let their_queens = board.queens(!us);

    // Pawn threats
    let pawn_attacks = pawn_structure.pawn_attacks(us);
    ctx.pawn_attacks_on_minors[us] += (pawn_attacks & their_minors).count() as u8;
    ctx.pawn_attacks_on_rooks[us] += (pawn_attacks & their_rooks).count() as u8;
    ctx.pawn_attacks_on_queens[us] += (pawn_attacks & their_queens).count() as u8;

    // King safety, threats and mobility
    let blockers = board.all_occupied();
    let enemy_king_zone = ctx.king_zones[!us];

    let pawn_attacks = pawn_structure.pawn_attacks(!us);
    let blocked_pawns = pawn_structure.blocked_pawns(us);

    let mobility_squares = !pawn_attacks & !blocked_pawns;

    for sq in board.knights(us) {
        let attacks = sq.knight_squares();

        // King safety
        let king_attacks = enemy_king_zone & attacks;
        ctx.king_attacks[!us] += king_attacks.count();

        // Threats
        ctx.minor_attacks_on_rooks[us] += (attacks & their_rooks).count() as u8;
        ctx.minor_attacks_on_queens[us] += (attacks & their_queens).count() as u8;

        // Mobility
        let mut available_squares = attacks & mobility_squares;

        if board.get_pinrays(us).contains(sq) {
            available_squares &= board.get_pinrays(us);
        }

        let sq_count = available_squares.count() as usize;

        if let Some(ref mut trace) = trace  {
            trace.knight_mobility[sq_count] += if WHITE { 1 } else { -1 };
        }

        total += KNIGHT_MOBILITY_BONUS[sq_count];

    }

    for sq in board.bishops(us) {
        let attacks = sq.bishop_squares(blockers);

        // King safety
        let king_attacks = enemy_king_zone & attacks;
        ctx.king_attacks[!us] += king_attacks.count();

        // Threats
        ctx.minor_attacks_on_rooks[us] += (attacks & their_rooks).count() as u8;
        ctx.minor_attacks_on_queens[us] += (attacks & their_queens).count() as u8;

        // Mobility
        let mut available_squares = attacks & mobility_squares;

        if board.get_pinrays(us).contains(sq) {
            available_squares &= board.get_pinrays(us);
        }

        let sq_count = available_squares.count() as usize;

        if let Some(ref mut trace) = trace  {
            trace.bishop_mobility[sq_count] += if WHITE { 1 } else { -1 };
        }

        total += BISHOP_MOBILITY_BONUS[sq_count];
    }

    for sq in board.rooks(us) {
        let attacks = sq.rook_squares(blockers);

        // King safety
        let king_attacks = enemy_king_zone & attacks;
        ctx.king_attacks[!us] += king_attacks.count();

        // Threats
        ctx.rook_attacks_on_queens[us] += (attacks & their_queens).count() as u8;

        // Mobility
        let mut available_squares = attacks & mobility_squares;

        if board.get_pinrays(us).contains(sq) {
            available_squares &= board.get_pinrays(us);
        }

        let sq_count = available_squares.count() as usize;

        if let Some(ref mut trace) = trace  {
            trace.rook_mobility[sq_count] += if WHITE { 1 } else { -1 };
        }

        total += ROOK_MOBILITY_BONUS[sq_count];
    }

    for sq in board.queens(us) {
        let attacks = sq.queen_squares(blockers);

        // King safety
        let king_attacks = enemy_king_zone & attacks;
        ctx.king_attacks[!us] += king_attacks.count();

        // Mobility
        let mut available_squares = attacks & mobility_squares;

        if board.get_pinrays(us).contains(sq) {
            available_squares &= board.get_pinrays(us);
        }

        let sq_count = available_squares.count() as usize;

        if let Some(ref mut trace) = trace  {
            trace.queen_mobility[sq_count] += if WHITE { 1 } else { -1 };
        }

        total += QUEEN_MOBILITY_BONUS[sq_count];
    }

    total
}

/// A penalty for the amount of freedom the friendly king has.
///
/// We quantify the "freedom" by placing a hypothetical queen on the king
/// square, and seeing how many available squares she would have.
///
/// The idea is that having many available queen squares correlates to 
/// having many slider attack vectors.
fn virtual_mobility<const WHITE: bool>(board: &Board, trace: Option<&mut EvalTrace>) -> S {
    let us = if WHITE { White } else { Black };
    let king_sq = board.kings(us).first();
    let blockers = board.all_occupied();
    let ours = board.occupied_by(us);
    let available_squares = king_sq.queen_squares(blockers) & !ours;
    let mobility = available_squares.count() as usize;

    if let Some(trace) = trace  {
        trace.virtual_mobility[mobility] += if WHITE { 1 } else { -1 };
    }

    VIRTUAL_MOBILITY_PENALTY[mobility]
}

/// A penalty for having many squares in the direct vicinity of the king 
/// under attack.
///
/// This uses the values that have been aggregated into an [EvalContext]
/// The heavy lifting has been done in populating the [EvalContext] inside 
/// [Board::mobility].
fn king_zone<const WHITE: bool>(ctx: &EvalContext, trace: Option<&mut EvalTrace>) -> S {
    let us = if WHITE { White } else { Black };
    let attacks = ctx.king_attacks[us];
    let attacks = usize::min(attacks as usize, 15);

    if let Some(trace) = trace  {
        trace.king_zone[attacks] += if WHITE { 1 } else { -1 };
    }

    KING_ZONE_ATTACKS[attacks]
}

/// A penalty for having pieces attacked by less valuable pieces.
///
/// There are many levels of granularity possible here, but we distinguish
/// between:
/// 
/// 1. Pawn attacks on minor pieces
/// 2. Pawn attacks on rooks
/// 3. Pawn attacks on queens
/// 4. Minor piece attacks on rooks
/// 5. Minor piece attacks on queens
/// 6. Rook attacks on queens
///
/// This uses the values that have been aggregated into an [EvalContext]
/// The heavy lifting has been done in populating the [EvalContext] inside 
/// [Board::mobility].
fn threats<const WHITE: bool>(ctx: &EvalContext, trace: Option<&mut EvalTrace>) -> S {
    let us = if WHITE { White } else { Black };

    if let Some(trace) = trace  {
        if WHITE {
            trace.pawn_attacks_on_minors  += ctx.pawn_attacks_on_minors[us] as i32;
            trace.pawn_attacks_on_rooks   += ctx.pawn_attacks_on_rooks[us] as i32; 
            trace.pawn_attacks_on_queens  += ctx.pawn_attacks_on_queens[us] as i32; 
            trace.minor_attacks_on_rooks  += ctx.minor_attacks_on_rooks[us] as i32; 
            trace.minor_attacks_on_queens += ctx.minor_attacks_on_queens[us] as i32; 
            trace.rook_attacks_on_queens  += ctx.rook_attacks_on_queens[us] as i32; 
        } else {
            trace.pawn_attacks_on_minors  -= ctx.pawn_attacks_on_minors[us] as i32;
            trace.pawn_attacks_on_rooks   -= ctx.pawn_attacks_on_rooks[us] as i32; 
            trace.pawn_attacks_on_queens  -= ctx.pawn_attacks_on_queens[us] as i32; 
            trace.minor_attacks_on_rooks  -= ctx.minor_attacks_on_rooks[us] as i32; 
            trace.minor_attacks_on_queens -= ctx.minor_attacks_on_queens[us] as i32; 
            trace.rook_attacks_on_queens  -= ctx.rook_attacks_on_queens[us] as i32; 
        }
    }

      PAWN_ATTACKS_ON_MINORS * ctx.pawn_attacks_on_minors[us] as i32
    + PAWN_ATTACKS_ON_ROOKS * ctx.pawn_attacks_on_rooks[us] as i32
    + PAWN_ATTACKS_ON_QUEENS * ctx.pawn_attacks_on_queens[us] as i32
    + MINOR_ATTACKS_ON_ROOKS * ctx.minor_attacks_on_rooks[us] as i32
    + MINOR_ATTACKS_ON_QUEENS * ctx.minor_attacks_on_queens[us] as i32
    + ROOK_ATTACKS_ON_QUEENS * ctx.rook_attacks_on_queens[us] as i32
}

