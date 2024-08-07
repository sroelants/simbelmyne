#![allow(unused_variables, unused_mut)]

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
pub mod terms;
pub mod util;
mod piece_square_tables;

use crate::s;

use chess::bitboard::Bitboard;
use chess::board::Board;
use chess::piece::Piece;
use chess::square::Square;
use chess::piece::PieceType;
use chess::piece::Color;
use params::TEMPO_BONUS;
use self::terms::*;
use self::pawn_structure::PawnStructure;
pub use util::*;


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
        // We pass around an EvalContext so expensive information gathered in 
        // some evaluation terms can be shared with other eval terms, instead
        // of recomputing them again.
        let mut ctx = EvalContext::new(board);

        // Add up all of the incremental terms stored on the Eval struct
        let mut total = self.material;
        total += self.psqt;
        total += self.pawn_structure.score();
        total += self.bishop_pair;
        total += self.rook_open_file;
        total += self.rook_semiopen_file;
        total += self.major_on_seventh;
        total += self.queen_open_file;
        total += self.queen_semiopen_file;
        total += self.pawn_shield;
        total += self.pawn_storm;
        total += self.passers_friendly_king;
        total += self.passers_enemy_king;
        total += self.knight_outposts;
        total += self.bishop_outposts;

        // Compute and add up the "volatile" evaluation terms. These are the 
        // terms that need to get recomputed in every node, anyway.
        total += connected_rooks::<WHITE>(board, None);
        total -= connected_rooks::<BLACK>(board, None);
        total += mobility::<WHITE>(board, &self.pawn_structure, &mut ctx, None);
        total -= mobility::<BLACK>(board, &self.pawn_structure, &mut ctx, None);
        total += virtual_mobility::<WHITE>(board, None);
        total -= virtual_mobility::<BLACK>(board, None);
        total += king_zone::<WHITE>(&mut ctx, None);
        total -= king_zone::<BLACK>(&mut ctx, None);
        total += threats::<WHITE>(&ctx, None);
        total -= threats::<BLACK>(&ctx, None);

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
        self.material += material(piece, None);
        self.psqt += psqt(piece, sq, None);
        self.update_incremental_terms(piece, board);
    }

    /// Update the score by removing a piece from it
    pub fn remove(&mut self, piece: Piece, sq: Square, board: &Board) {
        self.game_phase -= Self::phase_value(piece);
        self.material -= material(piece, None);
        self.psqt -= psqt(piece, sq, None);
        self.update_incremental_terms(piece, board);
    }

    /// Update the score by moving a piece from one square to another
    ///
    /// Slightly more efficient helper that does less work than calling 
    /// `Eval::remove` and `Eval::add` in succession.
    pub fn update(&mut self, piece: Piece, from: Square, to: Square, board: &Board) {
        let from_psqt = psqt(piece, from, None);
        let to_psqt = psqt(piece, to, None);
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

                self.rook_open_file  = rook_open_file::<WHITE>(board, &self.pawn_structure, None);
                self.rook_open_file -= rook_open_file::<BLACK>(board, &self.pawn_structure, None);
                self.rook_semiopen_file  = rook_semiopen_file::<WHITE>(board, &self.pawn_structure, None);
                self.rook_semiopen_file -= rook_semiopen_file::<BLACK>(board, &self.pawn_structure, None);
                self.queen_open_file  = queen_open_file::<WHITE>(board, &self.pawn_structure, None);
                self.queen_open_file -= queen_open_file::<BLACK>(board, &self.pawn_structure, None);
                self.queen_semiopen_file  = queen_semiopen_file::<WHITE>(board, &self.pawn_structure, None);
                self.queen_semiopen_file -= queen_semiopen_file::<BLACK>(board, &self.pawn_structure, None);
                self.major_on_seventh  = major_on_seventh::<WHITE>(board, None);
                self.major_on_seventh -= major_on_seventh::<BLACK>(board, None);
                self.knight_outposts  = knight_outposts::<WHITE>(board, &self.pawn_structure, None);
                self.knight_outposts -= knight_outposts::<BLACK>(board, &self.pawn_structure, None);
                self.bishop_outposts  = bishop_outposts::<WHITE>(board, &self.pawn_structure, None);
                self.bishop_outposts -= bishop_outposts::<BLACK>(board, &self.pawn_structure, None);
            },

            Knight => {
                self.knight_outposts  = knight_outposts::<WHITE>(board, &self.pawn_structure, None);
                self.knight_outposts -= knight_outposts::<BLACK>(board, &self.pawn_structure, None);
            },

            Bishop => {
                self.bishop_pair  = bishop_pair::<WHITE>(board, None);
                self.bishop_pair -= bishop_pair::<BLACK>(board, None);
                self.bishop_outposts  = bishop_outposts::<WHITE>(board, &self.pawn_structure, None);
                self.bishop_outposts -= bishop_outposts::<BLACK>(board, &self.pawn_structure, None);
            },

            Rook => {
                self.rook_open_file  = rook_open_file::<WHITE>(board, &self.pawn_structure, None);
                self.rook_open_file -= rook_open_file::<BLACK>(board, &self.pawn_structure, None);
                self.rook_semiopen_file  = rook_semiopen_file::<WHITE>(board, &self.pawn_structure, None);
                self.rook_semiopen_file -= rook_semiopen_file::<BLACK>(board, &self.pawn_structure, None);
                self.major_on_seventh  = major_on_seventh::<WHITE>(board, None);
                self.major_on_seventh -= major_on_seventh::<BLACK>(board, None);
            },

            Queen => {
                self.queen_open_file  = queen_open_file::<WHITE>(board, &self.pawn_structure, None);
                self.queen_open_file -= queen_open_file::<BLACK>(board, &self.pawn_structure, None);
                self.queen_semiopen_file  = queen_semiopen_file::<WHITE>(board, &self.pawn_structure, None);
                self.queen_semiopen_file -= queen_semiopen_file::<BLACK>(board, &self.pawn_structure, None);
                self.major_on_seventh  = major_on_seventh::<WHITE>(board, None);
                self.major_on_seventh -= major_on_seventh::<BLACK>(board, None);
            },

            King => {
                self.pawn_shield  = pawn_shield::<WHITE>(board, None);
                self.pawn_shield -= pawn_shield::<BLACK>(board, None);
                self.pawn_storm  = pawn_storm::<WHITE>(board, None);
                self.pawn_storm -= pawn_storm::<BLACK>(board, None);
                self.passers_friendly_king  = passers_friendly_king::<WHITE>(board, &self.pawn_structure, None);
                self.passers_friendly_king -= passers_friendly_king::<BLACK>(board, &self.pawn_structure, None);
                self.passers_enemy_king  = passers_enemy_king::<WHITE>(board, &self.pawn_structure, None);
                self.passers_enemy_king -= passers_enemy_king::<BLACK>(board, &self.pawn_structure, None);
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
pub struct EvalContext {
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
