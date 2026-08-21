//!  Logic that updates a Board state according to a provided move
//!
//! Many other engines (in particular so-called "pseudo-legal" engines rely
//! on a mechanism of "MakeMove" and "UnmakeMove", where a single board gets
//! updated in-place and reverted in-place.
//!
//! This makes sense, because they already have the machinery for make/unmake
//! for generating legal moves (all pseudo-legal moves ar made, and the ones
//! that lead to illegal boards are "unmade".
//!
//! Since verything is stack-allocated, this hasn't really cost us much
//! of a slowdown. It seems like Carp and Viridithas get away with it, so why
//! shouldn't we?

use super::moves::Move;
use crate::board::Board;
use crate::movegen::castling;
use crate::piece::Color;
use crate::piece::Piece;

impl Board {
  /// Given a board state and a move to play, update the board state to
  /// reflect that move.
  ///
  /// Note that this method will panic when used with NULL moves. If you want
  /// to play a "null" move (e.g., for null move pruning), use`
  /// Self::play_null_move` instead.
  pub fn play_move(&self, mv: Move) -> Board {
    let mut new_board = self.clone();
    let source = mv.src();
    let target = mv.tgt();
    let us = self.current;

    ////////////////////////////////////////////////////////////////////////
    //
    // Play move
    //
    ////////////////////////////////////////////////////////////////////////

    let piece = self.get_at(source).unwrap();

    // Figure out what piece to place at the target (considers promotions)
    let new_piece = if mv.is_promotion() {
      Piece::new(mv.get_promo_type().unwrap(), us)
    } else {
      piece
    };

    if mv.is_capture() {
      let capture_sq = mv.get_capture_sq();
      new_board.remove_at(capture_sq);
    }

    // Should we set the EP square?
    if mv.is_double_push() {
      new_board.en_passant = target.backward(us);
    } else {
      // Clear en-passant square
      new_board.en_passant = None;
    }

    if mv.is_castle() {
      let king_src = source;
      let rook_src = target; // KxR
      let rook_tgt = castling::rook_target(king_src, rook_src);
      let king_tgt = castling::king_target(king_src, rook_src);

      let king = new_board.remove_at(king_src).unwrap();
      let rook = new_board.remove_at(rook_src).unwrap();
      new_board.add_at(king_tgt, king);
      new_board.add_at(rook_tgt, rook);
    } else {
      // Remove selected piece from board
      new_board.remove_at(source);

      // Add the (new) piece to the board at the target square
      new_board.add_at(target, new_piece);
    }

    ////////////////////////////////////////////////////////////////////////
    //
    // Update castling rights
    //
    ////////////////////////////////////////////////////////////////////////

    // If the king moved, revoke their respective castling rights
    if piece.is_king() {
      new_board.castling_rights.remove_for(us);
    }

    new_board.castling_rights.remove(source);
    new_board.castling_rights.remove(target);

    ////////////////////////////////////////////////////////////////////////
    //
    // Update counters and flags
    //
    ////////////////////////////////////////////////////////////////////////

    // Update player
    new_board.current = self.current.opp();

    // Update move counter
    if self.current.is_black() {
      new_board.full_moves += 1;
    }

    if mv.is_capture() || piece.is_pawn() {
      new_board.half_moves = 0;
    } else {
      new_board.half_moves += 1;
    }

    ////////////////////////////////////////////////////////////////////////
    //
    // Update auxiliary bitboards (pins, checkers, threats)
    //
    ////////////////////////////////////////////////////////////////////////

    new_board.hv_pinrays = [
      new_board.compute_hv_pinrays::<true>(),
      new_board.compute_hv_pinrays::<false>(),
    ];

    new_board.diag_pinrays = [
      new_board.compute_diag_pinrays::<true>(),
      new_board.compute_diag_pinrays::<false>(),
    ];

    new_board.checkers = new_board.compute_checkers();
    new_board.threats = new_board.attacked_squares(!new_board.current);

    new_board
  }

  pub fn play_null_move(&self) -> Self {
    let mut new_board = self.clone();

    ////////////////////////////////////////////////////////////////////////
    //
    // Update counters and flags
    //
    ////////////////////////////////////////////////////////////////////////

    // Update player
    new_board.current = self.current.opp();

    // Clear en-passant square
    new_board.en_passant = None;

    // Update half-move counter
    new_board.half_moves += 1;

    // Update move counter
    if self.current == Color::Black {
      new_board.full_moves += 1;
    }

    // In case we're making a null move, update the side-relative stuff
    // and we're done here. 👋
    new_board.checkers = new_board.compute_checkers();
    new_board.threats = new_board.attacked_squares(!new_board.current);

    return new_board;
  }
}
