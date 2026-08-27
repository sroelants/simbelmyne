use super::Eval;
use super::EvalContext;
use super::S;
use super::params::*;
use super::tuner::EvalTrace;
use super::tuner::Tracer;
use crate::evaluate::lookups::CENTER_SQUARES;
use chess::attacks::between;
use chess::attacks::*;
use chess::bitboard::Bitboard;
use chess::board::Board;
use chess::constants::DARK_SQUARES;
use chess::constants::LIGHT_SQUARES;
use chess::constants::RANKS;
use chess::piece::Color;
use chess::piece::Piece;
use chess::piece::PieceType;
use chess::square::Square;

pub const PIECE_SQUARE_TABLES: [[S; Square::COUNT]; PieceType::COUNT] = [
  PARAMS.pawn_psqt,
  PARAMS.knight_psqt,
  PARAMS.bishop_psqt,
  PARAMS.rook_psqt,
  PARAMS.queen_psqt,
  PARAMS.king_psqt,
];

impl Eval {
  /// An evaluation score for having a specific piece on the board.
  ///
  /// This more or less corresponds to the classic heuristic values of
  /// 100 (Pawn), 300 (Knight), 300 (Bishop), 500 (Rook), 900 (Queen),
  /// but the values are tuned.
  ///
  /// The distinction between midgame and engame values means we can be more
  /// granular. E.g., a bishop is worth more in the endgame than a knight,
  /// rooks become more valuable in the endgame, etc...
  #[inline(always)]
  pub fn material(
    &self,
    piece: Piece,
    trace: &mut impl Tracer<EvalTrace>,
  ) -> S {
    trace.add(|t| {
      if piece.color().is_white() {
        t.piece_values[piece.piece_type()] += 1;
      } else {
        t.piece_values[piece.piece_type()] -= 1;
      }
    });

    if piece.color().is_white() {
      PARAMS.piece_values[piece.piece_type()]
    } else {
      -PARAMS.piece_values[piece.piece_type()]
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
  #[inline(always)]
  pub fn psqt(
    &self,
    piece: Piece,
    sq: Square,
    trace: &mut impl Tracer<EvalTrace>,
  ) -> S {
    let sq = if piece.color().is_white() {
      sq.flip()
    } else {
      sq
    };

    trace.add(|t| {
      use PieceType::*;
      let term = if piece.color().is_white() { 1 } else { -1 };
      match piece.piece_type() {
        Pawn => t.pawn_psqt[sq] += term,
        Knight => t.knight_psqt[sq] += term,
        Bishop => t.bishop_psqt[sq] += term,
        Rook => t.rook_psqt[sq] += term,
        Queen => t.queen_psqt[sq] += term,
        King => t.king_psqt[sq] += term,
      };
    });

    if piece.color().is_white() {
      PIECE_SQUARE_TABLES[piece.piece_type()][sq]
    } else {
      -PIECE_SQUARE_TABLES[piece.piece_type()][sq]
    }
  }

  /// A bonus for knights that are positioned on outpost squares.
  ///
  /// Outpost squares are squares that cannot easily be attacked by pawns,
  /// and are defended by one of our own pawns.
  ///
  /// For the implementation of outpost squares, see [PawnStructure::new].
  #[inline(always)]
  pub fn knight_outposts<const US: Color>(
    &self,
    board: &Board,
    trace: &mut impl Tracer<EvalTrace>,
  ) -> S {
    let perspective = if US.is_white() { 1 } else { -1 };
    let outpost_knights = board.knights(US) & self.kp_structure.outposts(US);
    let count = outpost_knights.count() as i32;

    trace.add(|t| t.knight_outposts += perspective * count);
    PARAMS.knight_outposts * count
  }

  /// A bonus for bishops that are positioned on outpost squares.
  ///
  /// Outpost squares are squares that cannot easily be attacked by pawns,
  /// and are defended by one of our own pawns.
  ///
  /// For the implementation of outpost squares, see [PawnStructure::new].
  #[inline(always)]
  pub fn bishop_outposts<const US: Color>(
    &self,
    board: &Board,
    trace: &mut impl Tracer<EvalTrace>,
  ) -> S {
    let perspective = if US.is_white() { 1 } else { -1 };
    let outpost_bishops = board.bishops(US) & self.kp_structure.outposts(US);
    let count = outpost_bishops.count() as i32;

    trace.add(|t| t.bishop_outposts += perspective * count);
    PARAMS.bishop_outposts * count
  }

  /// A bonus for having a bishop pair on opposite colored squares.
  ///
  /// This does not actually check the square colors, and just assumes that if
  /// the player has two bishops, they are opposite colored (rather than, say,
  /// two same-color bishops through a promotion)
  #[inline(always)]
  pub fn bishop_pair<const US: Color>(
    &self,
    board: &Board,
    trace: &mut impl Tracer<EvalTrace>,
  ) -> S {
    let perspective = if US.is_white() { 1 } else { -1 };
    let both_bishops = board.bishops(US).count() == 2;

    trace.add(|t| t.bishop_pair += perspective * both_bishops as i32);
    PARAMS.bishop_pair * both_bishops as i32
  }

  /// A bonus for having a rook on an open file
  ///
  /// Open files are files that have no pawns on them, and allow the rook to
  /// move freely along them without pawns blocking them in.
  ///
  /// For the implementation of open files, see [PawnStructure].
  #[inline(always)]
  pub fn rook_open_file<const US: Color>(
    &self,
    board: &Board,
    trace: &mut impl Tracer<EvalTrace>,
  ) -> S {
    let perspective = if US.is_white() { 1 } else { -1 };
    let rooks_on_open = board.rooks(US) & self.kp_structure.open_files();
    let count = rooks_on_open.count() as i32;

    trace.add(|t| t.rook_open_file += perspective * count);
    PARAMS.rook_open_file * count
  }

  /// A bonus for having a rook on a semi-open file
  ///
  /// Semi-open files are files that have no friendly pawns on them, but do
  /// have enemy pawns on them. They allow rooks to move somewhat freely,
  /// since they aren't blocked by any friendly pawns.
  ///
  /// For the implementation of semi-open files, see [PawnStructure].
  #[inline(always)]
  pub fn rook_semiopen_file<const US: Color>(
    &self,
    board: &Board,
    trace: &mut impl Tracer<EvalTrace>,
  ) -> S {
    let perspective = if US.is_white() { 1 } else { -1 };
    let rooks_on_semi = board.rooks(US) & self.kp_structure.semi_open_files(US);
    let count = rooks_on_semi.count() as i32;

    trace.add(|t| t.rook_semiopen_file += perspective * count);
    PARAMS.rook_semiopen_file * count
  }

  /// A bonus for having connected rooks on the back rank.
  ///
  /// Two rooks count as connected when they are withing direct line-of-sight
  /// of each other and are protecting one another.
  #[inline(always)]
  pub fn connected_rooks<const US: Color>(
    &self,
    board: &Board,
    trace: &mut impl Tracer<EvalTrace>,
  ) -> S {
    let perspective = if US.is_white() { 1 } else { -1 };
    let back_rank = if US.is_white() { 0 } else { 7 };
    let mut rooks = board.rooks(US);

    let Some(fst) = rooks.next() else {
      return S::default();
    };
    let Some(snd) = rooks.next() else {
      return S::default();
    };

    let on_back_rank = fst.rank() == back_rank && snd.rank() == back_rank;
    let unobstructed = (between(fst, snd) & board.all_occupied()).is_empty();
    let connected = on_back_rank && unobstructed;

    trace.add(|t| t.connected_rooks += perspective * connected as i32);
    PARAMS.connected_rooks * connected as i32
  }

  /// A bonus for having a major piece (rook or queen) on the 7th/2nd rank.
  ///
  /// The idea is that these are powerful pieces on the 7th rank, because
  /// they can trap the king on the 8th rank, and attack weak pawns on the 7th
  /// rank.
  ///
  /// As such, the terms assigns a bonus _only if_ the king is on the 8th rank
  /// or there are powns on the 7th.
  #[inline(always)]
  pub fn major_on_seventh<const US: Color>(
    &self,
    board: &Board,
    trace: &mut impl Tracer<EvalTrace>,
  ) -> S {
    let perspective = if US.is_white() { 1 } else { -1 };
    let seventh_rank = if US.is_white() { RANKS[6] } else { RANKS[1] };
    let eighth_rank = if US.is_white() { RANKS[7] } else { RANKS[0] };
    let pawns_on_seventh = !(board.pawns(!US) & seventh_rank).is_empty();
    let king_on_eighth = !(board.kings(!US) & eighth_rank).is_empty();
    let majors = board.rooks(US) | board.queens(US);

    let relevant = pawns_on_seventh || king_on_eighth;
    let count = (majors & seventh_rank).count() as i32;

    trace.add(|t| t.major_on_seventh += perspective * count * relevant as i32);
    PARAMS.major_on_seventh * count * relevant as i32
  }

  /// A bonus for having a queen on an open file.
  ///
  /// Identical in spirit and implementation to [Board::rook_open_file]
  #[inline(always)]
  pub fn queen_open_file<const US: Color>(
    &self,
    board: &Board,
    trace: &mut impl Tracer<EvalTrace>,
  ) -> S {
    let perspective = if US.is_white() { 1 } else { -1 };
    let queens_on_open = board.queens(US) & self.kp_structure.open_files();
    let count = queens_on_open.count() as i32;

    trace.add(|t| t.queen_open_file += perspective * count);
    PARAMS.queen_open_file * count
  }

  /// A bonus for having a queen on a semi-open file.
  ///
  /// Identical in spirit and implementation to [Board::rook_semiopen_file]
  #[inline(always)]
  pub fn queen_semiopen_file<const US: Color>(
    &self,
    board: &Board,
    trace: &mut impl Tracer<EvalTrace>,
  ) -> S {
    let perspective = if US.is_white() { 1 } else { -1 };
    let queens_on_semi = board.queens(US)
      & self.kp_structure.semi_open_files(US)
      & !self.kp_structure.open_files();
    let count = queens_on_semi.count() as i32;

    trace.add(|t| t.queen_semiopen_file += perspective * count);
    PARAMS.queen_semiopen_file * count
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
  #[inline]
  pub fn mobility<const US: Color>(
    &self,
    board: &Board,
    ctx: &mut EvalContext,
    trace: &mut impl Tracer<EvalTrace>,
  ) -> S {
    use PieceType::*;
    let mut total = S::default();

    let perspective = if US.is_white() { 1 } else { -1 };
    let our_pawns = board.pawns(US);
    let their_pawns = board.pawns(!US);
    let their_minors = board.knights(!US) | board.bishops(!US);
    let their_rooks = board.rooks(!US);
    let their_queens = board.queens(!US);

    // Pawn threats
    let pawn_attacks = board.pawn_attacks(US);
    ctx.threats[US] |= pawn_attacks;
    ctx.attacked_by[US][Pawn] |= pawn_attacks;
    ctx.king_attacks[!US] += (pawn_attacks & ctx.king_zones[!US]).count();

    // King safety, threats and mobility
    let blockers = board.all_occupied();
    let enemy_king_zone = ctx.king_zones[!US];

    let pawn_attacks = board.pawn_attacks(!US);
    let blocked_pawns = our_pawns & their_pawns.backward(US);

    let mobility_squares = !pawn_attacks & !blocked_pawns;

    let their_king = board.kings(!US).first();

    for sq in board.knights(US) {
      let attacks = knight_squares(sq);

      ctx.threats[US] |= attacks;
      ctx.attacked_by[US][Knight] |= attacks;

      // King safety
      let king_attacks = enemy_king_zone & attacks;
      ctx.king_attacks[!US] += king_attacks.count();

      // Mobility
      let mut available_squares = attacks & mobility_squares;

      if board.get_pinrays(US).contains(sq) {
        available_squares &= board.get_pinrays(US);
      }

      let sq_count = available_squares.count() as usize;

      total += PARAMS.knight_mobility[sq_count];
      trace.add(|t| t.knight_mobility[sq_count] += perspective);
    }

    for sq in board.bishops(US) {
      let attacks = bishop_squares(sq, blockers);

      ctx.threats[US] |= attacks;
      ctx.attacked_by[US][Bishop] |= attacks;

      // King safety
      let king_attacks = enemy_king_zone & attacks;
      ctx.king_attacks[!US] += king_attacks.count();

      // Long diagonal
      let long_diagonal = (attacks & CENTER_SQUARES).count() > 1;
      total += PARAMS.bishop_long_diagonal * long_diagonal as i32;
      trace
        .add(|t| t.bishop_long_diagonal += perspective * long_diagonal as i32);

      // Mobility
      let mut available_squares = attacks & mobility_squares;

      if board.get_pinrays(US).contains(sq) {
        available_squares &= board.get_pinrays(US);
      }

      let sq_count = available_squares.count() as usize;
      total += PARAMS.bishop_mobility[sq_count];
      trace.add(|t| t.bishop_mobility[sq_count] += perspective);
    }

    for sq in board.rooks(US) {
      let attacks = rook_squares(sq, blockers);

      ctx.threats[US] |= attacks;
      ctx.attacked_by[US][Rook] |= attacks;

      // King safety
      let king_attacks = enemy_king_zone & attacks;
      ctx.king_attacks[!US] += king_attacks.count();

      // Mobility
      let mut available_squares = attacks & mobility_squares;

      if board.get_pinrays(US).contains(sq) {
        available_squares &= board.get_pinrays(US);
      }

      let sq_count = available_squares.count() as usize;

      total += PARAMS.rook_mobility[sq_count];
      trace.add(|t| t.rook_mobility[sq_count] += perspective);
    }

    for sq in board.queens(US) {
      let attacks = queen_squares(sq, blockers);

      ctx.threats[US] |= attacks;
      ctx.attacked_by[US][Queen] |= attacks;

      // King safety
      let king_attacks = enemy_king_zone & attacks;
      ctx.king_attacks[!US] += king_attacks.count();

      // Mobility
      let mut available_squares = attacks & mobility_squares;

      if board.get_pinrays(US).contains(sq) {
        available_squares &= board.get_pinrays(US);
      }

      let sq_count = available_squares.count() as usize;

      total += PARAMS.queen_mobility[sq_count];
      trace.add(|t| t.queen_mobility[sq_count] += perspective);
    }

    let king_attacks = ctx.king_zones[US];
    ctx.threats[US] |= king_attacks;
    ctx.attacked_by[US][King] |= king_attacks;

    total
  }

  /// A penalty for the amount of freedom the friendly king has.
  ///
  /// We quantify the "freedom" by placing a hypothetical queen on the king
  /// square, and seeing how many available squares she would have.
  ///
  /// The idea is that having many available queen squares correlates to
  /// having many slider attack vectors.
  #[inline(always)]
  pub fn virtual_mobility<const US: Color>(
    &self,
    board: &Board,
    trace: &mut impl Tracer<EvalTrace>,
  ) -> S {
    let perspective = if US.is_white() { 1 } else { -1 };
    let king_sq = board.kings(US).first();
    let blockers = board.all_occupied();
    let ours = board.occupied_by(US);
    let available_squares = queen_squares(king_sq, blockers) & !ours;
    let mobility = available_squares.count() as usize;

    trace.add(|t| t.virtual_mobility[mobility] += perspective);
    PARAMS.virtual_mobility[mobility]
  }

  /// A penalty for having many squares in the direct vicinity of the king
  /// under attack.
  ///
  /// This uses the values that have been aggregated into an [EvalContext]
  /// The heavy lifting has been done in populating the [EvalContext] inside
  /// [Board::mobility].
  #[inline(always)]
  pub fn king_zone<const US: Color>(
    &self,
    ctx: &EvalContext,
    trace: &mut impl Tracer<EvalTrace>,
  ) -> S {
    let perspective = if US.is_white() { 1 } else { -1 };
    let attacks = ctx.king_attacks[US];
    let attacks = usize::min(attacks as usize, 15);

    trace.add(|t| t.king_zone[attacks] += perspective);
    PARAMS.king_zone[attacks]
  }

  /// A penalty for pieces under attack.
  ///
  /// Assigns a different weight for every (attacker/victim) pair.
  ///
  /// This uses the values that have been aggregated into an [EvalContext]
  /// The heavy lifting has been done in populating the [EvalContext] inside
  /// [Board::mobility].
  #[inline(always)]
  pub fn threats<const US: Color>(
    &self,
    board: &Board,
    ctx: &EvalContext,
    trace: &mut impl Tracer<EvalTrace>,
  ) -> S {
    use PieceType::*;
    let perspective = if US.is_white() { 1 } else { -1 };
    let mut total = S::default();

    trace.add(|t| {
      for victim in [Pawn, Knight, Bishop, Rook, Queen] {
        t.pawn_attacks[victim] += perspective
          * (board.get_bb(victim, !US) & ctx.attacked_by[US][Pawn]).count()
            as i32;
        t.knight_attacks[victim] += perspective
          * (board.get_bb(victim, !US) & ctx.attacked_by[US][Knight]).count()
            as i32;
        t.bishop_attacks[victim] += perspective
          * (board.get_bb(victim, !US) & ctx.attacked_by[US][Bishop]).count()
            as i32;
        t.rook_attacks[victim] += perspective
          * (board.get_bb(victim, !US) & ctx.attacked_by[US][Rook]).count()
            as i32;
        t.queen_attacks[victim] += perspective
          * (board.get_bb(victim, !US) & ctx.attacked_by[US][Queen]).count()
            as i32;
      }
    });

    for victim in [Pawn, Knight, Bishop, Rook, Queen] {
      total += PARAMS.pawn_attacks[victim]
        * (board.get_bb(victim, !US) & ctx.attacked_by[US][Pawn]).count()
          as i32;
      total += PARAMS.knight_attacks[victim]
        * (board.get_bb(victim, !US) & ctx.attacked_by[US][Knight]).count()
          as i32;
      total += PARAMS.bishop_attacks[victim]
        * (board.get_bb(victim, !US) & ctx.attacked_by[US][Bishop]).count()
          as i32;
      total += PARAMS.rook_attacks[victim]
        * (board.get_bb(victim, !US) & ctx.attacked_by[US][Rook]).count()
          as i32;
      total += PARAMS.queen_attacks[victim]
        * (board.get_bb(victim, !US) & ctx.attacked_by[US][Queen]).count()
          as i32;
    }

    total
  }

  /// Add bonuses for available checking moves (distinguishing between
  /// safe and unsafe)
  #[inline(always)]
  pub fn checks<const US: Color>(
    &self,
    board: &Board,
    ctx: &EvalContext,
    trace: &mut impl Tracer<EvalTrace>,
  ) -> S {
    use PieceType::*;
    let perspective = if US.is_white() { 1 } else { -1 };
    let their_king = board.kings(!US).first();
    let blockers = board.all_occupied();
    let pawn_pushes = (board.pawns(US)).forward(US);
    let blockers = board.all_occupied();
    let safe = !ctx.threats[!US];

    let mut safe_checks = [Bitboard::default(); 6];
    let mut unsafe_checks = [Bitboard::default(); 6];

    let pawn_checks =
      (ctx.attacked_by[US][Pawn] | pawn_pushes) & pawn_attacks(their_king, !US);
    safe_checks[Pawn] = pawn_checks & !ctx.attacked_by[!US][Pawn];
    unsafe_checks[Pawn] = pawn_checks & ctx.attacked_by[!US][Pawn];

    let knight_checks =
      ctx.attacked_by[US][Knight] & knight_squares(their_king);
    safe_checks[Knight] = knight_checks & safe;
    unsafe_checks[Knight] = knight_checks & !safe;

    let bishop_checks =
      ctx.attacked_by[US][Bishop] & bishop_squares(their_king, blockers);
    safe_checks[Bishop] = bishop_checks & safe;
    unsafe_checks[Bishop] = bishop_checks & !safe;

    let rook_checks =
      ctx.attacked_by[US][Rook] & rook_squares(their_king, blockers);
    safe_checks[Rook] = rook_checks & safe;
    unsafe_checks[Rook] = rook_checks & !safe;

    let queen_checks =
      ctx.attacked_by[US][Queen] & queen_squares(their_king, blockers);
    safe_checks[Queen] = queen_checks & safe;
    unsafe_checks[Queen] = queen_checks & !safe;

    trace.add(|t| {
      t.safe_checks[Pawn] += perspective * safe_checks[Pawn].count() as i32;
      t.safe_checks[Knight] += perspective * safe_checks[Knight].count() as i32;
      t.safe_checks[Bishop] += perspective * safe_checks[Bishop].count() as i32;
      t.safe_checks[Rook] += perspective * safe_checks[Rook].count() as i32;
      t.safe_checks[Queen] += perspective * safe_checks[Queen].count() as i32;

      t.unsafe_checks[Pawn] += perspective * unsafe_checks[Pawn].count() as i32;
      t.unsafe_checks[Knight] +=
        perspective * unsafe_checks[Knight].count() as i32;
      t.unsafe_checks[Bishop] +=
        perspective * unsafe_checks[Bishop].count() as i32;
      t.unsafe_checks[Rook] += perspective * unsafe_checks[Rook].count() as i32;
      t.unsafe_checks[Queen] +=
        perspective * unsafe_checks[Queen].count() as i32;
    });

    PARAMS.safe_checks[Pawn] * safe_checks[Pawn].count() as i32
      + PARAMS.safe_checks[Knight] * safe_checks[Knight].count() as i32
      + PARAMS.safe_checks[Bishop] * safe_checks[Bishop].count() as i32
      + PARAMS.safe_checks[Rook] * safe_checks[Rook].count() as i32
      + PARAMS.safe_checks[Queen] * safe_checks[Queen].count() as i32
      + PARAMS.unsafe_checks[Pawn] * unsafe_checks[Pawn].count() as i32
      + PARAMS.unsafe_checks[Knight] * unsafe_checks[Knight].count() as i32
      + PARAMS.unsafe_checks[Bishop] * unsafe_checks[Bishop].count() as i32
      + PARAMS.unsafe_checks[Rook] * unsafe_checks[Rook].count() as i32
      + PARAMS.unsafe_checks[Queen] * unsafe_checks[Queen].count() as i32
  }

  /// Bonus for a knight behind a pawn
  #[inline(always)]
  pub fn knight_shelter<const US: Color>(
    &self,
    board: &Board,
    trace: &mut impl Tracer<EvalTrace>,
  ) -> S {
    let perspective = if US.is_white() { 1 } else { -1 };
    let sheltered = board.knights(US).forward(US) & board.pawns(US);
    let count = sheltered.count() as i32;

    trace.add(|t| t.knight_shelter += perspective * count);
    PARAMS.knight_shelter * count
  }

  /// Bonus for a bishop behind a pawn
  #[inline(always)]
  pub fn bishop_shelter<const US: Color>(
    &self,
    board: &Board,
    trace: &mut impl Tracer<EvalTrace>,
  ) -> S {
    let perspective = if US.is_white() { 1 } else { -1 };
    let sheltered = board.bishops(US).forward(US) & board.pawns(US);
    let count = sheltered.count() as i32;

    trace.add(|t| t.bishop_shelter += perspective * count);
    PARAMS.bishop_shelter * count
  }

  /// Penalty for having bishops with many of their squares blocked by
  /// our pawns.
  #[inline(always)]
  pub fn bad_bishops<const US: Color>(
    &self,
    board: &Board,
    trace: &mut impl Tracer<EvalTrace>,
  ) -> S {
    let perspective = if US.is_white() { 1 } else { -1 };
    let mut total: S = S::default();

    for bishop in board.bishops(US) {
      let squares = if DARK_SQUARES.contains(bishop) {
        DARK_SQUARES
      } else {
        LIGHT_SQUARES
      };

      let blocking_pawns = (board.pawns(US) & squares).count();

      total += PARAMS.bad_bishops[blocking_pawns as usize];
      trace.add(|t| t.bad_bishops[blocking_pawns as usize] += perspective);
    }

    total
  }

  /// Passed pawn related evaluation that has to be recomputed on each move.
  #[inline(always)]
  pub fn volatile_passers<const US: Color>(
    &self,
    board: &Board,
    ctx: &EvalContext,
    trace: &mut impl Tracer<EvalTrace>,
  ) -> S {
    let mut total = S::default();
    let them = !US;
    let our_king = board.kings(US).first();
    let their_king = board.kings(them).first();
    let perspective = if US.is_white() { 1 } else { -1 };
    let only_kp =
      board.occupied_by(them) == board.kings(them) | board.pawns(them);
    let tempo = board.current == them;

    for passer in self.kp_structure.passed_pawns(US) {
      let stop_sq = passer.forward(US).unwrap();
      let rel_rank = if US.is_white() {
        passer.rank()
      } else {
        7 - passer.rank()
      };
      let free =
        board.get_at(stop_sq).is_none() && !ctx.threats[!US].contains(stop_sq);

      total += PARAMS.free_passer[rel_rank] * free as i32;
      trace.add(|t| t.free_passer[rel_rank] += perspective * free as i32);

      let protected = ctx.threats[US].contains(passer);
      total += PARAMS.protected_passer[rel_rank] * protected as i32;
      trace.add(|t| {
        t.protected_passer[rel_rank] += perspective * protected as i32
      });

      // Square rule
      let queening_dist = 7 - rel_rank;
      let their_king_dist = passer.max_dist(their_king);
      let inside_square = only_kp
        && queening_dist <= 4
        && queening_dist < their_king_dist - tempo as usize;

      total += PARAMS.square_rule * inside_square as i32;
      trace.add(|t| t.square_rule += perspective * inside_square as i32)
    }

    total
  }

  /// Assign a score to how many pawn pushes are available that _would_
  /// threaten a piece.
  /// TODO: Should this also consider forks?
  ///
  /// Look at all available pushes that would attack non-pawn pieces, that are
  /// on safe squares (= not attacked by them, or attacked by one of their
  /// non-pawn pieces and defended by us)
  #[inline(always)]
  pub fn push_threats<const US: Color>(
    &self,
    board: &Board,
    ctx: &EvalContext,
    trace: &mut impl Tracer<EvalTrace>,
  ) -> S {
    use PieceType::*;

    let mut total = S::default();
    let perspective = if US.is_white() { 1 } else { -1 };
    let third = if US.is_white() { RANKS[2] } else { RANKS[5] };

    let targets = board.occupied_by(!US) & !board.pawns(!US);

    // A square is safe if it is
    // 1. Not attacked by the opponent
    // 2. Attacked by an apponent piece (non-pawn), but also defended by us.
    let mut safe = !ctx.threats[!US];
    safe |= ctx.threats[US] & !ctx.attacked_by[!US][Pawn];

    let pushes = board.pawns(US).forward(US) & !board.all_occupied();
    let double_pushes = (pushes & third).forward(US) & !board.all_occupied();

    let safe_pushes = (pushes | double_pushes) & safe;
    let push_attacks =
      safe_pushes.forward_left(US) | safe_pushes.forward_right(US);
    let attacked = targets & push_attacks;

    for sq in attacked {
      let attacked = board.get_at(sq).unwrap().piece_type();
      total += PARAMS.push_threats[attacked];
      trace.add(|t| t.push_threats[attacked] += perspective);
    }

    total
  }
}
