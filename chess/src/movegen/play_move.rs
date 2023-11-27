use crate::{board::Board, piece::Color};
use crate::piece::Piece;

use super::{
    castling::{CastleType, CastlingRights},
    moves::Move,
};

/// Given a board state and a move to play, update the board state to reflect
/// that move.
impl Board {
    pub fn play_move(&self, mv: Move) -> Board {
        let mut new_board = self.clone();

        // Remove selected piece from board, and update fields
        let mut selected_piece = new_board
            .remove_at(mv.src().into())
            .expect("We're sure there's a piece on the source square");

        // Update Castling rights
        // If the piece is a king, revoke that side's castling rights
        // TODO: Have all this logic live an the CastlingRights struct
        if selected_piece.is_king() {
            if new_board.current.is_white() {
                new_board.castling_rights.remove(CastlingRights::WQ);
                new_board.castling_rights.remove(CastlingRights::WK);
            } else {
                new_board.castling_rights.remove(CastlingRights::BQ);
                new_board.castling_rights.remove(CastlingRights::BK);
            }
        }

        // If any of the rooks moved, revoke their respective castling rights
        if selected_piece.is_rook() {
            match (mv.src().rank(), mv.src().file()) {
                (0, 0) => new_board.castling_rights.remove(CastlingRights::WQ),
                (0, 7) => new_board.castling_rights.remove(CastlingRights::WK),
                (7, 0) => new_board.castling_rights.remove(CastlingRights::BQ),
                (7, 7) => new_board.castling_rights.remove(CastlingRights::BK),
                _ => {}
            }
        }

        match (mv.tgt().rank(), mv.tgt().file()) {
            (0, 0) => new_board.castling_rights.remove(CastlingRights::WQ),
            (0, 7) => new_board.castling_rights.remove(CastlingRights::WK),
            (7, 0) => new_board.castling_rights.remove(CastlingRights::BQ),
            (7, 7) => new_board.castling_rights.remove(CastlingRights::BK),
            _ => {}
        }

        // play move
        new_board.remove_at(mv.tgt().into()); //Captured piece?
        
        if mv.is_promotion() {
            let ptype = mv.get_promo_type()
                .expect("The move is a promotion and has a promotion type");
            selected_piece = Piece::new(ptype, selected_piece.color());
        }

        new_board.add_at(mv.tgt().into(), selected_piece);

        if mv.is_en_passant() {
            let capture_sq = mv
                .tgt()
                .backward(new_board.current)
                .expect("En-passant capture target is in bounds");
            new_board.remove_at(capture_sq);
        }

        if mv.is_castle() {
            let ctype = CastleType::from_move(mv).unwrap();
            let mv = ctype.rook_move();

            let selected_piece = new_board
                .remove_at(mv.src().into())
                .expect("We're sure there's a piece on the source square");

            new_board.remove_at(mv.tgt().into());
            new_board.add_at(mv.tgt().into(), selected_piece);
        }

        // Update en-passant square
        if mv.is_double_push() {
            new_board.en_passant = mv.src().forward(self.current);
        } else {
            new_board.en_passant = None;
        }

        // Update half-move clock
        if mv.is_capture() || selected_piece.is_pawn() {
            new_board.half_moves = 0;
        } else {
            new_board.half_moves += 1;
        }

        // Update moves
        if self.current == Color::Black {
            new_board.full_moves += 1;
        }

        new_board.current = self.current.opp();

        new_board
    }
}
