// TODO:
// - Don't rely on `board.king_threats`. Maybe just store king_threats as
//   threats on the board? Has a slight effect on eval stuff, but maybe it's
//   fine?
// - Improve EP logic
// - Use precomputed diagonal pinmasks in pawn attack movegen to save on extra
//   masking
use crate::attacks::*;
use crate::bitboard::Bitboard;
use crate::board::Board;
use crate::constants::FILES;
use crate::constants::RANKS;
use crate::movegen::legal_moves::MoveList;
use crate::movegen::moves::Move;
use crate::movegen::moves::MoveType;
use crate::piece::Color;
use crate::square::Square;
use MoveType::*;

pub mod castling;
pub mod legal_moves;
pub mod moves;
pub mod play_move;

#[cfg(not(all(target_arch = "x86_64", target_feature = "bmi2")))]
pub mod magics;

#[cfg(all(target_arch = "x86_64", target_feature = "bmi2"))]
pub mod pext;

pub fn gen_moves(board: &Board, moves: &mut MoveList) {
  match board.current {
    Color::White => {
      gen_tacticals_for::<{ Color::White }>(board, moves);
      gen_quiets_for::<{ Color::White }>(board, moves);
    }
    Color::Black => {
      gen_tacticals_for::<{ Color::Black }>(board, moves);
      gen_quiets_for::<{ Color::Black }>(board, moves);
    }
  }
}

pub fn gen_tacticals(board: &Board, moves: &mut MoveList) {
  match board.current {
    Color::White => gen_tacticals_for::<{ Color::White }>(board, moves),
    Color::Black => gen_tacticals_for::<{ Color::Black }>(board, moves),
  }
}

pub fn gen_quiets(board: &Board, moves: &mut MoveList) {
  match board.current {
    Color::White => gen_quiets_for::<{ Color::White }>(board, moves),
    Color::Black => gen_quiets_for::<{ Color::Black }>(board, moves),
  }
}

fn gen_quiets_for<const US: Color>(board: &Board, moves: &mut MoveList) {
  let occ = board.all_occupied();
  let checkers = board.get_checkers();

  let mut king_targets = !occ & !board.threats;
  let mut targets = !occ;

  if checkers.count() > 1 {
    king_targets &= !board.king_threats();
    king_moves::<US, true>(board, moves, king_targets);
    return;
  } else if !checkers.is_empty() {
    king_targets &= !board.king_threats();
    targets = between(board.kings(US).first(), checkers.first());
  }

  pawn_quiets::<US>(board, moves, targets);
  knight_moves::<US, true>(board, moves, targets);
  slider_moves::<US, true>(board, moves, targets);
  king_moves::<US, true>(board, moves, king_targets);
  castles::<US>(board, moves);
}

fn gen_tacticals_for<const US: Color>(board: &Board, moves: &mut MoveList) {
  let ours = board.occupied_by(US);
  let theirs = board.occupied_by(!US);
  let checkers = board.get_checkers();
  let promo_rank = if US.is_white() { RANKS[7] } else { RANKS[0] };

  let mut king_targets = theirs & !board.threats;
  let mut pawn_targets = theirs | (promo_rank & !ours);
  let mut targets = theirs;

  if checkers.count() > 1 {
    king_targets &= !board.king_threats();
    king_moves::<US, false>(board, moves, king_targets);
    return;
  } else if !checkers.is_empty() {
    let king = board.kings(US).first();
    let checker = checkers.first();

    targets = checkers;
    pawn_targets = checkers | (promo_rank & between(checker, king));
    king_targets &= !board.king_threats();
  }

  pawn_tacticals::<US>(board, moves, pawn_targets);
  knight_moves::<US, false>(board, moves, targets);
  slider_moves::<US, false>(board, moves, targets);
  king_moves::<US, false>(board, moves, king_targets);
}

// TODO: Make this more efficient by having pre-generated diagonal pinmasks
// (It'll save me on a couple of extra masking operations)
#[inline]
fn pawn_tacticals<const US: Color>(
  board: &Board,
  moves: &mut MoveList,
  targets: Bitboard,
) {
  let theirs = board.occupied_by(!US);
  let pinmask = board.get_diag_pinrays(US);
  let promo_rank = if US.is_white() { RANKS[7] } else { RANKS[0] };

  // Horizontally pinned pawns can't capture, so mask them out to get
  // the board of attacker pawns
  let pawns = board.pawns(US) & !board.get_hv_pinrays(US);

  // Split the pawns up into diagonally pinned and unpinned pawns
  let pinned = pawns & pinmask;
  let unpinned = pawns & !pinned;

  // ---- Left attacks ----

  {
    // Left attacks
    let pinned = pinned.forward_left(US) & pinmask;
    let unpinned = unpinned.forward_left(US);
    let attacks = (pinned | unpinned) & targets & theirs;

    // Left regular (non-promo) captures
    let victims = attacks & !promo_rank;
    let attackers = victims.backward_right(US);
    push_paired(moves, attackers, victims, Capture);

    // Left promo captures
    let pr_victims = attacks & promo_rank;
    let pr_attackers = pr_victims.backward_right(US);
    push_promo_captures(moves, pr_attackers, pr_victims);
  }

  // ---- Right attacks ----

  {
    let pinned = pinned.forward_right(US) & pinmask;
    let unpinned = unpinned.forward_right(US);
    let attacks = (pinned | unpinned) & targets & theirs;

    // right regular (non-promo) captures
    let victims = attacks & !promo_rank;
    let attackers = victims.backward_left(US);
    push_paired(moves, attackers, victims, Capture);

    // right promo captures
    let pr_victims = attacks & promo_rank;
    let pr_attackers = pr_victims.backward_left(US);
    push_promo_captures(moves, pr_attackers, pr_victims);
  }

  // ---- Promos ----

  {
    let pinmask = board.get_hv_pinrays(US);

    // Diagonally pinned pawns can't push, so mask them out to get
    // the board of pusher pawns
    let pawns = board.pawns(US) & !board.get_diag_pinrays(US);
    let targets = targets & !theirs;

    // Split the pawns up into diagonally pinned and unpinned pawns
    let pinned = pawns & pinmask;
    let unpinned = pawns & !pinned;

    let pinned_pushes = pinned.forward(US) & targets & pinmask;
    let unpinned_pushes = unpinned.forward(US) & targets;

    let promos = (pinned_pushes | unpinned_pushes) & promo_rank;
    let sources = promos.backward(US);
    push_promos(moves, sources, promos);
  }

  // ---- En passant ----

  if board.en_passant.is_some() {
    gen_ep::<US>(board, moves);
  }
}

// TODO: Rewrite this!
#[inline(always)]
fn gen_ep<const US: Color>(board: &Board, moves: &mut MoveList) {
  debug_assert!(board.en_passant.is_some());

  // SAFETY: Checked in the assert
  let ep_sq = unsafe { board.en_passant.unwrap_unchecked() };
  let checkers = board.checkers;
  let in_check = checkers.count() > 0;
  //SAFETY: EP Square is never on 0th/7th rank
  let attacked_pawn = unsafe { ep_sq.backward(US).unwrap_unchecked() };
  let attacking_pawns =
    board.pawns(US) & !board.get_pinrays(US) & pawn_attacks(ep_sq, !US);

  if in_check && !checkers.contains(attacked_pawn) {
    return;
  }

  // TODO: Make this cheaper by avoiding the `xray_checkers` call
  for attacker in attacking_pawns {
    // Make sure the capture doesn't lead to a discovered check.
    let cleared_rank = RANKS[attacked_pawn.rank()];
    let source = Bitboard::from(attacker);
    let captured = Bitboard::from(attacked_pawn);
    let remove = source | captured;
    let xray_checkers = board.xray_checkers(remove);
    let exposes_check = !(xray_checkers & cleared_rank).is_empty();

    if exposes_check {
      continue;
    }

    moves.push(Move::new(attacker, ep_sq, MoveType::EnPassant));
  }
}

#[inline]
fn pawn_quiets<const US: Color>(
  board: &Board,
  moves: &mut MoveList,
  targets: Bitboard,
) {
  let occ = board.all_occupied();
  let king = board.kings(US).first();
  let pawns = board.pawns(US);
  let pinned = board.pinned(US);
  let pinmask = FILES[king.file()];
  let promo_rank = if US.is_white() { RANKS[7] } else { RANKS[0] };
  let fourth_rank = if US.is_white() { RANKS[3] } else { RANKS[4] };

  let free = (pawns & !pinned) | (pawns & pinmask);
  let pushes = free.forward(US) & !occ;

  // ---- Single pushes ----

  let singles = pushes & !promo_rank & targets;
  let sources = singles.backward(US);
  push_paired(moves, sources, singles, Quiet);

  // ---- Double pushes ----

  let doubles = pushes.forward(US) & fourth_rank & targets;
  let sources = doubles.backward_by(2, US);
  push_paired(moves, sources, doubles, DoublePush);
}

#[inline]
fn knight_moves<const US: Color, const QUIET: bool>(
  board: &Board,
  moves: &mut MoveList,
  targets: Bitboard,
) {
  let mt = if QUIET { Quiet } else { Capture };

  for sq in board.knights(US) & !board.pinned(US) {
    let attacks = knight_squares(sq) & targets;
    push_moves(moves, sq, attacks, mt);
  }
}

#[inline]
fn slider_moves<const US: Color, const QUIET: bool>(
  board: &Board,
  moves: &mut MoveList,
  targets: Bitboard,
) {
  let occ = board.all_occupied();
  let pinned = board.pinned(US);

  let hv = board.hv_sliders(US);
  let diag = board.diag_sliders(US);
  let hv_pinrays = board.get_hv_pinrays(US);
  let diag_pinrays = board.get_diag_pinrays(US);
  let mt = if QUIET { Quiet } else { Capture };

  for sq in hv & !pinned {
    let attacks = rook_squares(sq, occ) & targets;
    push_moves(moves, sq, attacks, mt);
  }

  for sq in hv & hv_pinrays {
    let attacks = rook_squares(sq, occ) & targets & hv_pinrays;
    push_moves(moves, sq, attacks, mt);
  }

  for sq in diag & !pinned {
    let attacks = bishop_squares(sq, occ) & targets;
    push_moves(moves, sq, attacks, mt);
  }

  for sq in diag & diag_pinrays {
    let attacks = bishop_squares(sq, occ) & targets & diag_pinrays;
    push_moves(moves, sq, attacks, mt);
  }
}

#[inline]
fn king_moves<const US: Color, const QUIET: bool>(
  board: &Board,
  moves: &mut MoveList,
  targets: Bitboard,
) {
  let king = board.kings(US).first();
  let attacks = king_squares(king) & targets;
  let mt = if QUIET { Quiet } else { Capture };
  push_moves(moves, king, attacks, mt);
}

#[inline(always)]
fn castles<const US: Color>(board: &Board, moves: &mut MoveList) {
  if !board.checkers.is_empty() {
    return;
  }

  let threats = board.get_threats();
  let blockers = board.all_occupied();

  for ctype in board.castling_rights.get_available(US) {
    let attacked = ctype.vulnerable_squares() & threats;
    let blocked = ctype.los_squares() & blockers;

    if attacked.is_empty() && blocked.is_empty() {
      moves.push(ctype.king_move());
    }
  }
}

#[inline(always)]
fn push_moves(moves: &mut MoveList, sq: Square, tgts: Bitboard, mt: MoveType) {
  for tgt in tgts {
    moves.push(Move::new(sq, tgt, mt));
  }
}

#[inline(always)]
fn push_paired(
  moves: &mut MoveList,
  src: Bitboard,
  tgt: Bitboard,
  mt: MoveType,
) {
  for (src, tgt) in src.zip(tgt) {
    moves.push(Move::new(src, tgt, mt));
  }
}

#[inline(always)]
fn push_promos(moves: &mut MoveList, src: Bitboard, tgt: Bitboard) {
  for (src, tgt) in src.zip(tgt) {
    moves.push(Move::new(src, tgt, QueenPromo));
    moves.push(Move::new(src, tgt, RookPromo));
    moves.push(Move::new(src, tgt, BishopPromo));
    moves.push(Move::new(src, tgt, KnightPromo));
  }
}

#[inline(always)]
fn push_promo_captures(moves: &mut MoveList, src: Bitboard, tgt: Bitboard) {
  for (src, tgt) in src.zip(tgt) {
    moves.push(Move::new(src, tgt, QueenPromoCapture));
    moves.push(Move::new(src, tgt, RookPromoCapture));
    moves.push(Move::new(src, tgt, BishopPromoCapture));
    moves.push(Move::new(src, tgt, KnightPromoCapture));
  }
}
