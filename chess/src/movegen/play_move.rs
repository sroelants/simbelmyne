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

use crate::board::Board;
use crate::piece::Color;
use crate::piece::Piece;
use crate::square::Square;
use super::castling::CastleType;
use super::moves::Move;

impl Board {
    /// Given a board state and a move to play, update the board state to 
    /// reflect that move.
    ///
    /// Note that playing a null move (`Move::NULL`) is valid, and is done 
    /// quite frequently, e.g., during Null Move Pruning.
    pub fn play_move(&self, mv: Move) -> Board {
        use Square::*;
        let mut new_board = self.clone();
        let source = mv.src();
        let target = mv.tgt();
        let us = self.current;

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
        if mv == Move::NULL {
            new_board.checkers = new_board.compute_checkers(new_board.current);
            new_board.threats = new_board.king_threats();

            return new_board;
        }

        ////////////////////////////////////////////////////////////////////////
        //
        // Play move
        //
        ////////////////////////////////////////////////////////////////////////

        let piece = self.get_at(source).unwrap();

        // Remove selected piece from board
        new_board.remove_at(source);

        // Remove any piece that might be on the target square.
        new_board.remove_at(target);

        // Place the moved piece back on the board, taking promotions into 
        // account.
        if mv.is_promotion() {
            let ptype = mv.get_promo_type().unwrap();
            new_board.add_at(target, Piece::new(ptype, us));
        } else {
            new_board.add_at(target, piece);
        }

        // Capture en-passant 
        if mv.is_en_passant() {
            let capture_sq = target.backward(us).unwrap();
            new_board.remove_at(capture_sq);
        }

        // Is case of castle, also move the rook to the appropriate square
        if mv.is_castle() {
            let ctype = CastleType::from_move(mv).unwrap();
            let rook_move = ctype.rook_move();
            let rook = new_board.remove_at(rook_move.src()).unwrap();
            new_board.add_at(rook_move.tgt(), rook);
        }

        // Should we set the EP square?
        if mv.is_double_push() {
            new_board.en_passant = target.backward(us);
        } 

        // Should we reset the half-move counter?
        if mv.is_capture() || piece.is_pawn() {
            new_board.half_moves = 0;
        }

        ////////////////////////////////////////////////////////////////////////
        //
        // Update castling rights
        //
        ////////////////////////////////////////////////////////////////////////
        
        // TODO: Should this live in `castling.rs` ?

        // If the king moved, revoke their respective castling rights
        if piece.is_king() {
            if self.current.is_white() {
                new_board.castling_rights.remove(CastleType::WQ);
                new_board.castling_rights.remove(CastleType::WK);
            } else {
                new_board.castling_rights.remove(CastleType::BQ);
                new_board.castling_rights.remove(CastleType::BK);
            }
        }

        // If any other piece moves from the rook square, assume this also 
        // removesthe castling rights. Otherwise, rook captures wouldn't update 
        // the castling rights correctly.
        match source {
            A1 => new_board.castling_rights.remove(CastleType::WQ),
            H1 => new_board.castling_rights.remove(CastleType::WK),
            A8 => new_board.castling_rights.remove(CastleType::BQ),
            H8 => new_board.castling_rights.remove(CastleType::BK),
            _ => {}
        }

        // If any other piece moves to the rook square, assume this also removes
        // the castling rights. Otherwise, rook captures wouldn't update the
        // castling rights correctly.
        match target {
            A1 => new_board.castling_rights.remove(CastleType::WQ),
            H1 => new_board.castling_rights.remove(CastleType::WK),
            A8 => new_board.castling_rights.remove(CastleType::BQ),
            H8 => new_board.castling_rights.remove(CastleType::BK),
            _ => {}
        }

        new_board.hv_pinrays = [
            new_board.compute_hv_pinrays::<true>(), 
            new_board.compute_hv_pinrays::<false>()
        ];

        new_board.diag_pinrays = [
            new_board.compute_diag_pinrays::<true>(), 
            new_board.compute_diag_pinrays::<false>()
        ];

        new_board.checkers = new_board.compute_checkers(new_board.current);

        new_board.threats = new_board.king_threats();

        new_board
    }
}
