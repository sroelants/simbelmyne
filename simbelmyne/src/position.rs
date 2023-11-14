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

#[cfg(test)]
mod tests {
    use super::*;
    use chess::square::Square::*;
    use chess::movegen::moves::MoveType::*;

    #[test]
    fn test_hash_updates() {
        let initial_pos: Position = Position::new(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
                .parse()
                .unwrap()
        );

        let mut final_pos = initial_pos.clone();

        let expected: Position = Position::new(
            "r1bqkbnr/pppp1ppp/2n5/4p1B1/3P4/8/PPP1PPPP/RN1QKBNR w KQkq - 2 3"
                .parse()
                .unwrap()
        );

        let moves = vec![
            Move::new(D2, D4, DoublePush), 
            Move::new(E7, E5, DoublePush), 
            Move::new(C1, G5, Quiet), 
            Move::new(B8, C6, Quiet)
        ];

        for mv in moves {
            final_pos = final_pos.play_move(mv);
        }

        // Check that incremental updates yield the same result as hashing the entire board
        assert_eq!(final_pos.hash, final_pos.board.into());

        // Check whether the hash matches the expected board's
        assert_eq!(final_pos.hash, expected.hash);
    }
}
