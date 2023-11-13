use chess::{board::Board, movegen::moves::Move};

use crate::{evaluate::Score, zobrist::ZHash};

#[derive(Debug, Copy, Clone)]
pub(crate) struct Position {
    pub board: Board,
    pub score: Score,
    pub hash: ZHash,
}

impl Position {
    pub fn new(board: Board) -> Self {
        Position {
            board, 
            score: board.into(),
            hash: board.into()
        }
    }

    pub fn play_move(&self, mv: Move) -> Self {
        let us = self.board.current;
        
        // Update board
        let new_board = self.board.play_move(mv);
        let mut new_score = self.score.clone();
        let mut new_hash = self.hash.clone();

        // Remove piece from score
        let old_piece = self.board.piece_list[mv.src() as usize]
            .expect("The source target of a move has a piece");

        new_score.remove(us, old_piece, mv.src());
        new_hash.toggle_piece(old_piece, mv.src());

        // Add back value of new position. This takes care of promotions too
        let new_piece = new_board.piece_list[mv.tgt() as usize]
          .expect("The target square of a move is occupied after playing");

        new_score.add(us, new_piece, mv.tgt());
        new_hash.toggle_piece(new_piece, mv.tgt());

        // If capture: remove value
        if mv.is_capture() {
            if mv.is_en_passant() {
                let ep_sq = mv.tgt().backward(old_piece.color()).unwrap();

                let &captured = self.board.get_at(ep_sq)
                    .expect("A capture has a piece on the tgt square before playing");

                new_score.remove(us, captured, ep_sq);
                new_hash.toggle_piece(captured, ep_sq);
            } else {
                let &captured = self.board.get_at(mv.tgt())
                    .expect("A capture has a piece on the tgt square before playing");

                new_score.remove(us, captured, mv.tgt());
                new_hash.toggle_piece(captured, mv.tgt());
            }
        }

        // Update remaining Zobrist information

        // If move was double_push: set en-passant
        if let Some(ep_sq) = new_board.en_passant {
            new_hash.toggle_ep(ep_sq)
        }

        // Remove any previous EP square
        if let Some(ep_sq) = self.board.en_passant {
            new_hash.toggle_ep(ep_sq)
        }

        new_hash.toggle_castling(self.board.castling_rights);
        new_hash.toggle_castling(new_board.castling_rights);
        new_hash.toggle_side();

        Self {
            board: new_board,
            score: new_score.flipped(),
            hash: new_hash
        }
    }
}
