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

pub mod kp_cache;
pub mod kp_structure;
mod lookups;
pub mod params;
pub mod terms;
pub mod tuner;
pub mod util;

use crate::position::Position;
use crate::s;

use self::kp_structure::KingPawnStructure;
use self::terms::*;
use Color::*;
use chess::attacks::king_squares;
use chess::bitboard::Bitboard;
use chess::board::Board;
use chess::constants::DARK_SQUARES;
use chess::piece::Color;
use chess::piece::PieceType;
use chess::square::Square;
use kp_cache::KingPawnCache;
use kp_cache::KingPawnCacheEntry;
use lookups::KINGSIDE;
use lookups::QUEENSIDE;
use params::PARAMS;
use tuner::EvalTrace;
use tuner::NullTracer;
use tuner::Tracer;
pub use util::*;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum BoardSide {
  Queenside = 0,
  Kingside = 1,
}

#[inline(always)]
pub fn board_side(sq: Square) -> BoardSide {
  if sq.file() < 4 {
    BoardSide::Queenside
  } else {
    BoardSide::Kingside
  }
}

#[inline(always)]
pub fn hm(sq: Square, side: BoardSide) -> Square {
  if side == BoardSide::Kingside {
    sq.mirror()
  } else {
    sq
  }
}

// TODO: Check whether it's faster to look up through the mailbox
fn refresh_psqt(board: &Board, stm: Color, king_side: BoardSide) -> S {
  use PieceType::*;
  let mut total = S::default();

  for ptype in [Pawn, Knight, Bishop, Rook, Queen, King] {
    let bb = board.get_bb(ptype, stm);

    for sq in bb {
      let sq = hm(sq, king_side);
      total += material(ptype, stm, &mut NullTracer);
      total += psqt(ptype, stm, sq, &mut NullTracer);
    }
  }

  total
}

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
  game_phase: u8,
  psqt: [S; 2],
  kp_structure: KingPawnStructure,
  knights: S,
  bishops: S,
  bishop_pair: S,
  rooks: S,
  queens: S,
  major_on_seventh: S,
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
    eval.init(board);
    eval
  }

  pub fn init(&mut self, board: &Board) {
    self.init_traced(board, &mut NullTracer);
  }

  pub fn init_traced(
    &mut self,
    board: &Board,
    trace: &mut impl Tracer<EvalTrace>,
  ) {
    let king_sides = [
      board_side(board.kings(White).first()),
      board_side(board.kings(Black).first()),
    ];

    for (sq_idx, piece) in board.piece_list.into_iter().enumerate() {
      if let Some(piece) = piece {
        let color = piece.color();
        let ptype = piece.piece_type();
        let sq = hm(Square::from(sq_idx), king_sides[color]);
        self.game_phase += Self::phase_value(ptype);
        self.psqt[color] += material(ptype, color, trace);
        self.psqt[color] += psqt(ptype, color, sq, trace);
      }
    }

    self.kp_structure = KingPawnStructure::new(board, trace);

    self.knights +=
      knight_outposts::<{ White }>(board, &self.kp_structure, trace);
    self.knights -=
      knight_outposts::<{ Black }>(board, &self.kp_structure, trace);
    self.knights += knight_shelter::<{ White }>(board, trace);
    self.knights -= knight_shelter::<{ Black }>(board, trace);

    self.bishops +=
      bishop_outposts::<{ White }>(board, &self.kp_structure, trace);
    self.bishops -=
      bishop_outposts::<{ Black }>(board, &self.kp_structure, trace);
    self.bishops += bishop_shelter::<{ White }>(board, trace);
    self.bishops -= bishop_shelter::<{ Black }>(board, trace);
    self.bishops += bad_bishops::<{ White }>(board, trace);
    self.bishops -= bad_bishops::<{ Black }>(board, trace);

    self.bishop_pair +=
      bishop_pair::<{ White }>(board, &self.kp_structure, trace);
    self.bishop_pair -=
      bishop_pair::<{ Black }>(board, &self.kp_structure, trace);

    self.rooks += rook_open_file::<{ White }>(board, &self.kp_structure, trace);
    self.rooks -= rook_open_file::<{ Black }>(board, &self.kp_structure, trace);
    self.rooks +=
      rook_semiopen_file::<{ White }>(board, &self.kp_structure, trace);
    self.rooks -=
      rook_semiopen_file::<{ Black }>(board, &self.kp_structure, trace);

    self.queens +=
      queen_open_file::<{ White }>(board, &self.kp_structure, trace);
    self.queens -=
      queen_open_file::<{ Black }>(board, &self.kp_structure, trace);
    self.queens +=
      queen_semiopen_file::<{ White }>(board, &self.kp_structure, trace);
    self.queens -=
      queen_semiopen_file::<{ Black }>(board, &self.kp_structure, trace);

    self.major_on_seventh += major_on_seventh::<{ White }>(board, trace);
    self.major_on_seventh -= major_on_seventh::<{ Black }>(board, trace);
  }

  pub fn evaluate(&mut self, board: &Board) -> Score {
    self.evaluate_traced(board, &mut NullTracer)
  }

  /// Return the total (tapered) score for the position as the sum of the
  /// incremental evaluation terms and the volatile terms.
  pub fn evaluate_traced(
    &mut self,
    board: &Board,
    trace: &mut impl Tracer<EvalTrace>,
  ) -> Score {
    // We pass around an EvalContext so expensive information gathered in
    // some evaluation terms can be shared with other eval terms, instead
    // of recomputing them again.
    let mut ctx = EvalContext::new(board);

    // Add up all of the incremental terms stored on the Eval struct
    let mut total = S::default();
    total += self.psqt[White];
    total -= self.psqt[Black];

    total += self.kp_structure.score();

    total += self.knights;
    total += self.bishops;
    total += self.bishop_pair;
    total += self.rooks;
    total += self.queens;
    total += self.major_on_seventh;

    // Compute and add up the "volatile" evaluation terms. These are the
    // terms that need to get recomputed in every node, anyway.
    total += connected_rooks::<{ White }>(board, trace);
    total -= connected_rooks::<{ Black }>(board, trace);
    total += mobility::<{ White }>(board, &mut ctx, trace);
    total -= mobility::<{ Black }>(board, &mut ctx, trace);
    total += virtual_mobility::<{ White }>(board, trace);
    total -= virtual_mobility::<{ Black }>(board, trace);
    total += king_zone::<{ White }>(&mut ctx, trace);
    total -= king_zone::<{ Black }>(&mut ctx, trace);
    total += threats::<{ White }>(board, &ctx, trace);
    total -= threats::<{ Black }>(board, &ctx, trace);
    total += checks::<{ White }>(board, &ctx, trace);
    total -= checks::<{ Black }>(board, &ctx, trace);
    total +=
      volatile_passers::<{ White }>(board, &self.kp_structure, &ctx, trace);
    total -=
      volatile_passers::<{ Black }>(board, &self.kp_structure, &ctx, trace);
    total += push_threats::<{ White }>(board, &ctx, trace);
    total -= push_threats::<{ Black }>(board, &ctx, trace);

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

  pub fn apply(
    &self,
    update: EvalUpdate,
    pos: &Position,
    cache: &mut KingPawnCache,
  ) -> Self {
    let mut new_eval = *self;
    let mut dirty = PieceSet::new();

    let mut needs_refresh = false;

    let king_sides = [
      board_side(pos.board.kings(White).first()),
      board_side(pos.board.kings(Black).first()),
    ];

    for &PieceUpdate { piece, mut sq } in update.added() {
      let color = piece.color();
      let ptype = piece.piece_type();

      new_eval.add(ptype, color, hm(sq, king_sides[color]));
      dirty.add(ptype);
    }

    for &PieceUpdate { piece, mut sq } in update.removed() {
      let color = piece.color();
      let ptype = piece.piece_type();

      if ptype == PieceType::King {
        needs_refresh = board_side(sq) != king_sides[color];
      }

      new_eval.remove(ptype, color, hm(sq, king_sides[color]));
      dirty.add(ptype);
    }

    // If king changed sides, refresh psqt
    if needs_refresh {
      let stm = !pos.board.current; // previous stm
      new_eval.psqt[stm] = refresh_psqt(&pos.board, stm, king_sides[stm]);
    }

    new_eval.update_incremental_terms(dirty, pos, cache);

    new_eval
  }

  /// Update the Eval by adding a piece to it
  pub fn add(&mut self, ptype: PieceType, color: Color, sq: Square) {
    self.game_phase += Self::phase_value(ptype);
    self.psqt[color] += material(ptype, color, &mut NullTracer);
    self.psqt[color] += psqt(ptype, color, sq, &mut NullTracer);
  }

  /// Update the score by removing a piece from it
  pub fn remove(&mut self, ptype: PieceType, color: Color, sq: Square) {
    self.game_phase -= Self::phase_value(ptype);
    self.psqt[color] -= material(ptype, color, &mut NullTracer);
    self.psqt[color] -= psqt(ptype, color, sq, &mut NullTracer);
  }

  fn update_incremental_terms(
    &mut self,
    dirty: PieceSet,
    pos: &Position,
    kp_cache: &mut KingPawnCache,
  ) {
    let hash = pos.kp_hash;
    let board = &pos.board;

    if (PieceSet::PK & dirty).nempty() {
      self.kp_structure = if let Some(entry) = kp_cache.probe(hash) {
        entry.into()
      } else {
        let kp_structure = KingPawnStructure::new(board, &mut NullTracer);
        kp_cache.insert(KingPawnCacheEntry::new(hash, kp_structure));
        kp_structure
      };
    }

    if (PieceSet::PN & dirty).nempty() {
      self.knights = knight_outposts::<{ White }>(
        board,
        &self.kp_structure,
        &mut NullTracer,
      );
      self.knights -= knight_outposts::<{ Black }>(
        board,
        &self.kp_structure,
        &mut NullTracer,
      );
      self.knights += knight_shelter::<{ White }>(board, &mut NullTracer);
      self.knights -= knight_shelter::<{ Black }>(board, &mut NullTracer);
    }

    if (PieceSet::B & dirty).nempty() {
      self.bishop_pair =
        bishop_pair::<{ White }>(board, &self.kp_structure, &mut NullTracer);
      self.bishop_pair -=
        bishop_pair::<{ Black }>(board, &self.kp_structure, &mut NullTracer);
    }

    if (PieceSet::PB & dirty).nempty() {
      self.bishops = bishop_outposts::<{ White }>(
        board,
        &self.kp_structure,
        &mut NullTracer,
      );
      self.bishops -= bishop_outposts::<{ Black }>(
        board,
        &self.kp_structure,
        &mut NullTracer,
      );
      self.bishops += bishop_shelter::<{ White }>(board, &mut NullTracer);
      self.bishops -= bishop_shelter::<{ Black }>(board, &mut NullTracer);
      self.bishops += bad_bishops::<{ White }>(board, &mut NullTracer);
      self.bishops -= bad_bishops::<{ Black }>(board, &mut NullTracer);
    }

    if (PieceSet::PR & dirty).nempty() {
      self.rooks =
        rook_open_file::<{ White }>(board, &self.kp_structure, &mut NullTracer);
      self.rooks -=
        rook_open_file::<{ Black }>(board, &self.kp_structure, &mut NullTracer);
      self.rooks += rook_semiopen_file::<{ White }>(
        board,
        &self.kp_structure,
        &mut NullTracer,
      );
      self.rooks -= rook_semiopen_file::<{ Black }>(
        board,
        &self.kp_structure,
        &mut NullTracer,
      );
    }

    if (PieceSet::PQ & dirty).nempty() {
      self.queens = queen_open_file::<{ White }>(
        board,
        &self.kp_structure,
        &mut NullTracer,
      );
      self.queens -= queen_open_file::<{ Black }>(
        board,
        &self.kp_structure,
        &mut NullTracer,
      );
      self.queens += queen_semiopen_file::<{ White }>(
        board,
        &self.kp_structure,
        &mut NullTracer,
      );
      self.queens -= queen_semiopen_file::<{ Black }>(
        board,
        &self.kp_structure,
        &mut NullTracer,
      );
    }

    if (PieceSet::PRQK & dirty).nempty() {
      self.major_on_seventh =
        major_on_seventh::<{ White }>(board, &mut NullTracer);
      self.major_on_seventh -=
        major_on_seventh::<{ Black }>(board, &mut NullTracer);
    }
  }

  /// Return the game phase as a value between 0 and 24.
  ///
  /// 0 corresponds to endgame, 24 corresponds to midgame
  fn phase_value(ptype: PieceType) -> u8 {
    const GAME_PHASE_VALUES: [u8; PieceType::COUNT] = [0, 1, 1, 2, 4, 0];
    GAME_PHASE_VALUES[ptype]
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

    let white_king_zone = get_king_zone::<{ Color::White }>(white_king);
    let black_king_zone = get_king_zone::<{ Color::Black }>(black_king);

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
  let weak_pawns = board.pawns(weak);
  let pawns_missing = 8 - strong_pawns.count() as i32;
  let mut pawn_scale = 128 - pawns_missing * pawns_missing;

  let on_one_side = (strong_pawns & QUEENSIDE).is_empty()
    || (strong_pawns & KINGSIDE).is_empty();

  if on_one_side {
    pawn_scale -= 20;
  }

  let strong_nonpawn = (board.occupied_by(strong) & !strong_pawns).count();
  let weak_nonpawn = (board.occupied_by(weak) & !weak_pawns).count();
  let bishops = board.piece_bbs[Bishop];

  let opp_bishops = strong_nonpawn == 2
    && weak_nonpawn == 2
    && bishops.count() == 2
    && (bishops & DARK_SQUARES).count() == 1;

  if opp_bishops {
    let scale = if strong_nonpawn == 1 { 64 } else { 96 };
    pawn_scale = pawn_scale.min(scale);
  }

  pawn_scale
}

fn get_king_zone<const US: Color>(sq: Square) -> Bitboard {
  let ring = king_squares(sq);
  let zone = ring | ring.forward(US);
  zone & !Bitboard::from(sq)
}
