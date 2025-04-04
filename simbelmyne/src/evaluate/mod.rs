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
pub mod kp_structure;
pub mod pretty_print;
pub mod terms;
pub mod kp_cache;
pub mod util;

use crate::history_tables::history::HistoryIndex;
use crate::s;
use crate::zobrist::ZHash;

use chess::bitboard::Bitboard;
use chess::board::Board;
use chess::constants::DARK_SQUARES;
use chess::movegen::castling::CastleType;
use chess::movegen::moves::Move;
use chess::piece::Piece;
use chess::square::Square;
use chess::piece::PieceType;
use chess::piece::Color;
use lookups::KINGSIDE;
use lookups::QUEENSIDE;
use params::PARAMS;
use kp_cache::KingPawnCache;
use kp_cache::KingPawnCacheEntry;
use tuner::NullTrace;
use tuner::Trace;
use self::kp_structure::KingPawnStructure;
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

    /// The total king-pawn structure score
    /// See [KingPawnStructure] for implementation
    kp_structure: KingPawnStructure,

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

    /// A bonus for having a knight on an outpost square
    /// See [Board::knight_outposts] for implementation
    knight_outposts: S,

    /// A bonus for having a bishop on an outpost square
    /// See [Board::bishop_outposts] for implementation
    bishop_outposts: S,

    knight_shelter: S,

    bishop_shelter: S,

    bad_bishops: S
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
    pub fn new(board: &Board, trace: &mut impl Trace) -> Self {
        let mut eval = Self::default();

        for (sq_idx, piece) in board.piece_list.into_iter().enumerate() {
            if let Some(piece) = piece {
                let sq = Square::from(sq_idx);
                eval.game_phase += Self::phase_value(piece);
                eval.material += eval.material(piece, trace);
                eval.psqt += eval.psqt(piece, sq, trace);
            }
        }

        eval.kp_structure         = KingPawnStructure::new(board, trace);
        eval.knight_outposts        = eval.knight_outposts::<WHITE>(board, trace);
        eval.knight_outposts       -= eval.knight_outposts::<BLACK>(board, trace);
        eval.bishop_outposts        = eval.bishop_outposts::<WHITE>(board, trace);
        eval.bishop_outposts       -= eval.bishop_outposts::<BLACK>(board, trace);
        eval.bishop_pair            = eval.bishop_pair::<WHITE>(board, trace);
        eval.bishop_pair           -= eval.bishop_pair::<BLACK>(board, trace);
        eval.rook_open_file         = eval.rook_open_file::<WHITE>(board, trace);
        eval.rook_open_file        -= eval.rook_open_file::<BLACK>(board, trace);
        eval.rook_semiopen_file     = eval.rook_semiopen_file::<WHITE>(board, trace);
        eval.rook_semiopen_file    -= eval.rook_semiopen_file::<BLACK>(board, trace);
        eval.queen_open_file        = eval.queen_open_file::<WHITE>(board, trace);
        eval.queen_open_file       -= eval.queen_open_file::<BLACK>(board, trace);
        eval.queen_semiopen_file    = eval.queen_semiopen_file::<WHITE>(board, trace);
        eval.queen_semiopen_file   -= eval.queen_semiopen_file::<BLACK>(board, trace);
        eval.major_on_seventh       = eval.major_on_seventh::<WHITE>(board, trace);
        eval.major_on_seventh      -= eval.major_on_seventh::<BLACK>(board, trace);
        eval.knight_shelter         = eval.knight_shelter::<WHITE>(board, trace);
        eval.knight_shelter        -= eval.knight_shelter::<BLACK>(board, trace);
        eval.bishop_shelter         = eval.bishop_shelter::<WHITE>(board, trace);
        eval.bishop_shelter        -= eval.bishop_shelter::<BLACK>(board, trace);
        eval.bad_bishops            = eval.bad_bishops::<WHITE>(board, trace);
        eval.bad_bishops           -= eval.bad_bishops::<BLACK>(board, trace);

        eval
    }

    /// Return the total (tapered) score for the position as the sum of the
    /// incremental evaluation terms and the volatile terms.
    pub fn total(&mut self, board: &Board, trace: &mut impl Trace) -> Score {
        // We pass around an EvalContext so expensive information gathered in 
        // some evaluation terms can be shared with other eval terms, instead
        // of recomputing them again.
        let mut ctx = EvalContext::new(board);

        // Add up all of the incremental terms stored on the Eval struct
        let mut total = self.material;
        total += self.psqt;
        total += self.kp_structure.score();
        total += self.knight_outposts;
        total += self.bishop_outposts;
        total += self.knight_shelter;
        total += self.bishop_shelter;
        total += self.bishop_pair;
        total += self.rook_open_file;
        total += self.rook_semiopen_file;
        total += self.queen_open_file;
        total += self.queen_semiopen_file;
        total += self.major_on_seventh;
        total += self.bad_bishops;

        // Compute and add up the "volatile" evaluation terms. These are the 
        // terms that need to get recomputed in every node, anyway.
        total += self.connected_rooks::<WHITE>(board, trace);
        total -= self.connected_rooks::<BLACK>(board, trace);
        total += self.mobility::<WHITE>(board, &mut ctx, trace);
        total -= self.mobility::<BLACK>(board, &mut ctx, trace);
        total += self.virtual_mobility::<WHITE>(board, trace);
        total -= self.virtual_mobility::<BLACK>(board, trace);
        total += self.king_zone::<WHITE>(&mut ctx, trace);
        total -= self.king_zone::<BLACK>(&mut ctx, trace);
        total += self.threats::<WHITE>(board, &ctx, trace);
        total -= self.threats::<BLACK>(board, &ctx, trace);
        total += self.checks::<WHITE>(board, &ctx, trace);
        total -= self.checks::<BLACK>(board, &ctx, trace);
        total += self.volatile_passers::<WHITE>(board, &ctx, trace);
        total -= self.volatile_passers::<BLACK>(board, &ctx, trace);
        total += self.push_threats::<WHITE>(board, &ctx, trace);
        total -= self.push_threats::<BLACK>(board, &ctx, trace);

        // Add a side-relative tempo bonus
        // The position should be considered slightly more advantageous for the
        // current side-to-move.
        let perspective = if board.current.is_white() { 1 } else { -1 };
        total += PARAMS.tempo * perspective;
        trace.add(|t| t.tempo += perspective);

        // Downscale the endgame score depending on how drawish the position is
        let eg_scaling = endgame_scaling(board, total.eg());
        let total = S::new(total.mg(), total.eg() * eg_scaling / 128);
        trace.add(|t| t.eg_scaling = eg_scaling);

        // Interpolate between midgame and endgame evals, taking into account
        // the endgame scaling.
        let score = total.lerp(self.game_phase);

        // Return the score relative to the current side-to-move
        perspective * score
    }

    pub fn play_move(
        &self, 
        idx: HistoryIndex, 
        board: &Board, 
        kp_hash: ZHash, 
        kp_cache: &mut KingPawnCache
    ) -> Self {
        let mut new_score = *self;
        let HistoryIndex { moved, captured, mv } = idx;
        let us = moved.color();

        if mv == Move::NULL {
            return new_score;
        }

        // Remove any captured pieces
        if let Some(captured) = captured {
            new_score.remove(captured, mv.get_capture_sq(), &board, kp_hash, kp_cache);
        }

        // Update the moved piece
        if idx.mv.is_promotion() {
            new_score.remove(moved, mv.src(), &board, kp_hash, kp_cache);

            let promo_piece = Piece::new(mv.get_promo_type().unwrap(), us);
            new_score.add(promo_piece, mv.tgt(), &board, kp_hash, kp_cache);
        } else {
            new_score.update(moved, mv.src(), mv.tgt(), &board, kp_hash, kp_cache);
        }

        if mv.is_castle() {
            let ctype = CastleType::from_move(mv).unwrap();
            let rook_move = ctype.rook_move();
            let rook = Piece::new(PieceType::Rook, us);
            new_score.update(rook, rook_move.src(), rook_move.tgt(), &board, kp_hash, kp_cache);
        }

        new_score
    }

    /// Update the Eval by adding a piece to it
    pub fn add(
        &mut self, 
        piece: Piece, 
        sq: Square, 
        board: &Board, 
        kp_hash: ZHash, 
        kp_cache: &mut KingPawnCache
    ) {
        self.game_phase += Self::phase_value(piece);
        self.material += self.material(piece, &mut NullTrace);
        self.psqt += self.psqt(piece, sq, &mut NullTrace);
        self.update_incremental_terms(piece, board, kp_hash, kp_cache);
    }

    /// Update the score by removing a piece from it
    pub fn remove(
        &mut self, 
        piece: Piece, 
        sq: Square, 
        board: &Board, 
        kp_hash: ZHash, 
        kp_cache: &mut KingPawnCache
    ) {
        self.game_phase -= Self::phase_value(piece);
        self.material -= self.material(piece, &mut NullTrace);
        self.psqt -= self.psqt(piece, sq, &mut NullTrace);
        self.update_incremental_terms(piece, board, kp_hash, kp_cache);
    }

    /// Update the score by moving a piece from one square to another
    ///
    /// Slightly more efficient helper that does less work than calling 
    /// `Eval::remove` and `Eval::add` in succession.
    pub fn update(
        &mut self, 
        piece: Piece, 
        from: Square, 
        to: Square, 
        board: &Board,
        kp_hash: ZHash, 
        kp_cache: &mut KingPawnCache
    ) {
        self.psqt -= self.psqt(piece, from, &mut NullTrace);
        self.psqt += self.psqt(piece, to, &mut NullTrace);
        self.update_incremental_terms(piece, board, kp_hash, kp_cache);
    }

    /// Update the incremental eval terms, according to piece that moved.
    ///
    /// This tries to save as much work as possible, by only recomputing eval
    /// terms that depend on the moved piece. No need to update rook-related
    /// terms when a bishop has moved.
    fn update_incremental_terms(
        &mut self, 
        piece: Piece, 
        board: &Board, 
        kp_hash: ZHash, 
        kp_cache: &mut KingPawnCache
    ) {
        use PieceType::*;

        match piece.piece_type() {
            // Pawn moves require almost _all_ terms, save a couple, to be 
            // updated.
            Pawn => {
                self.kp_structure = if let Some(entry) = kp_cache.probe(kp_hash) {
                    entry.into()
                } else {
                    let kp_structure = KingPawnStructure::new(board, &mut NullTrace);
                    kp_cache.insert(KingPawnCacheEntry::new(kp_hash, kp_structure));
                    kp_structure
                };

                self.knight_outposts      = self.knight_outposts::<WHITE>(board, &mut NullTrace);
                self.knight_outposts     -= self.knight_outposts::<BLACK>(board, &mut NullTrace);
                self.bishop_outposts      = self.bishop_outposts::<WHITE>(board, &mut NullTrace);
                self.bishop_outposts     -= self.bishop_outposts::<BLACK>(board, &mut NullTrace);
                self.knight_shelter       = self.knight_shelter::<WHITE>(board, &mut NullTrace);
                self.knight_shelter      -= self.knight_shelter::<BLACK>(board, &mut NullTrace);
                self.bishop_shelter       = self.bishop_shelter::<WHITE>(board, &mut NullTrace);
                self.bishop_shelter      -= self.bishop_shelter::<BLACK>(board, &mut NullTrace);
                self.rook_open_file       = self.rook_open_file::<WHITE>(board, &mut NullTrace);
                self.rook_open_file      -= self.rook_open_file::<BLACK>(board, &mut NullTrace);
                self.rook_semiopen_file   = self.rook_semiopen_file::<WHITE>(board, &mut NullTrace);
                self.rook_semiopen_file  -= self.rook_semiopen_file::<BLACK>(board, &mut NullTrace);
                self.queen_open_file      = self.queen_open_file::<WHITE>(board, &mut NullTrace);
                self.queen_open_file     -= self.queen_open_file::<BLACK>(board, &mut NullTrace);
                self.queen_semiopen_file  = self.queen_semiopen_file::<WHITE>(board, &mut NullTrace);
                self.queen_semiopen_file -= self.queen_semiopen_file::<BLACK>(board, &mut NullTrace);
                self.major_on_seventh     = self.major_on_seventh::<WHITE>(board, &mut NullTrace);
                self.major_on_seventh    -= self.major_on_seventh::<BLACK>(board, &mut NullTrace);
                self.bad_bishops          = self.bad_bishops::<WHITE>(board, &mut NullTrace);
                self.bad_bishops         -= self.bad_bishops::<BLACK>(board, &mut NullTrace);
            },

            Knight => {
                self.knight_outposts      = self.knight_outposts::<WHITE>(board, &mut NullTrace);
                self.knight_outposts     -= self.knight_outposts::<BLACK>(board, &mut NullTrace);
                self.knight_shelter       = self.knight_shelter::<WHITE>(board, &mut NullTrace);
                self.knight_shelter      -= self.knight_shelter::<BLACK>(board, &mut NullTrace);
            },

            Bishop => {
                self.bishop_pair          = self.bishop_pair::<WHITE>(board, &mut NullTrace);
                self.bishop_pair         -= self.bishop_pair::<BLACK>(board, &mut NullTrace);
                self.bishop_outposts      = self.bishop_outposts::<WHITE>(board, &mut NullTrace);
                self.bishop_outposts     -= self.bishop_outposts::<BLACK>(board, &mut NullTrace);
                self.bishop_shelter       = self.bishop_shelter::<WHITE>(board, &mut NullTrace);
                self.bishop_shelter      -= self.bishop_shelter::<BLACK>(board, &mut NullTrace);
                self.bad_bishops          = self.bad_bishops::<WHITE>(board, &mut NullTrace);
                self.bad_bishops         -= self.bad_bishops::<BLACK>(board, &mut NullTrace);
            },

            Rook => {
                self.rook_open_file       = self.rook_open_file::<WHITE>(board, &mut NullTrace);
                self.rook_open_file      -= self.rook_open_file::<BLACK>(board, &mut NullTrace);
                self.rook_semiopen_file   = self.rook_semiopen_file::<WHITE>(board, &mut NullTrace);
                self.rook_semiopen_file  -= self.rook_semiopen_file::<BLACK>(board, &mut NullTrace);
                self.major_on_seventh     = self.major_on_seventh::<WHITE>(board, &mut NullTrace);
                self.major_on_seventh    -= self.major_on_seventh::<BLACK>(board, &mut NullTrace);
            },

            Queen => {
                self.queen_open_file      = self.queen_open_file::<WHITE>(board, &mut NullTrace);
                self.queen_open_file     -= self.queen_open_file::<BLACK>(board, &mut NullTrace);
                self.queen_semiopen_file  = self.queen_semiopen_file::<WHITE>(board, &mut NullTrace);
                self.queen_semiopen_file -= self.queen_semiopen_file::<BLACK>(board, &mut NullTrace);
                self.major_on_seventh     = self.major_on_seventh::<WHITE>(board, &mut NullTrace);
                self.major_on_seventh    -= self.major_on_seventh::<BLACK>(board, &mut NullTrace);
            },

            King => {
                self.kp_structure = if let Some(entry) = kp_cache.probe(kp_hash) {
                    entry.into()
                } else {
                    let kp_structure = KingPawnStructure::new(board, &mut NullTrace);
                    kp_cache.insert(KingPawnCacheEntry::new(kp_hash, kp_structure));
                    kp_structure
                };

                self.major_on_seventh     = self.major_on_seventh::<WHITE>(board, &mut NullTrace);
                self.major_on_seventh    -= self.major_on_seventh::<BLACK>(board, &mut NullTrace);
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

    /// Bitboards of all squares attacked by a given color
    threats: [Bitboard; Color::COUNT],

    /// Bitboards of all squares attacked by a given piece type
    attacked_by: [[Bitboard; PieceType::COUNT]; Color::COUNT],
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
            threats: [Bitboard::EMPTY; Color::COUNT],
            attacked_by: [[Bitboard::EMPTY; PieceType::COUNT]; Color::COUNT],
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
//
// Endgame scaling
//
////////////////////////////////////////////////////////////////////////////////

// Taken from Weiss for now, will expand upon this at some point...
pub fn endgame_scaling(board: &Board, eg_score: i32) -> i32 {
    use Color::*;
    use PieceType::*;

    let strong = if eg_score > 0 { White } else { Black };
    let weak = !strong;

    let strong_pawns = board.pawns(strong);
    let pawns_missing = 8 - strong_pawns.count() as i32;
    let mut pawn_scale = 128 - pawns_missing * pawns_missing;

    let on_one_side = (strong_pawns & QUEENSIDE).is_empty() 
        || (strong_pawns & KINGSIDE).is_empty();

    if  on_one_side {
        pawn_scale -= 20;
    }

    let strong_nonpawn = (board.occupied_by(strong) & !board.pawns(strong)).count();
    let weak_nonpawn = (board.occupied_by(weak) & !board.pawns(weak)).count();

    let opp_bishops = 
        strong_nonpawn <= 2 &&
        weak_nonpawn <= 2 &&
        strong_nonpawn == weak_nonpawn &&
        board.bishops(strong).count() == 1 &&
        board.bishops(weak).count() == 1 &&
        (board.piece_bbs[Bishop] & DARK_SQUARES).count() == 1;

    if opp_bishops {
        let scale = if strong_nonpawn == 1 { 64 } else { 96 };
        pawn_scale = pawn_scale.min(scale);
    }

    pawn_scale
}
