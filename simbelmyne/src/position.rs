use chess::{board::Board, movegen::{moves::{Move, BareMove}, castling::CastleType}};

use crate::{evaluate::Score, zobrist::ZHash};

#[derive(Debug, Clone)]
pub struct Position {
    pub board: Board,
    pub score: Score,
    pub hash: ZHash,
    pub history: Vec<ZHash>,
}

impl Position {
    pub fn new(board: Board) -> Self {
        Position {
            board, 
            score: board.into(),
            hash: board.into(),
            history: Vec::with_capacity(100),
        }
    }

    pub fn is_repetition(&self) -> bool {
        self.history
            .iter()
            .rev()
            .skip(1)
            .step_by(2) 
            .any(|&historic| historic == self.hash)
    }

    pub fn play_move(&self, mv: Move) -> Self {
        let us = self.board.current;
        
        // Update board
        let new_board = self.board.play_move(mv);
        let mut new_score = self.score.clone();
        let mut new_hash = self.hash.clone();
        let mut new_history = self.history.clone();

        // Push the old hash to the history. We know it's in bounds, because at 
        // 100 half-moves, it would be a draw.
        new_history.push(self.hash);


        // Make any updates to hash that don't depend on the move first
        // (In case of a NULL move, we want to cut this function short)
        
        // Update playing side
        new_hash.toggle_side();

        // Remove any previous EP square
        if let Some(ep_sq) = self.board.en_passant {
            new_hash.toggle_ep(ep_sq)
        }

        // If move was double_push: set en-passant
        if let Some(ep_sq) = new_board.en_passant {
            new_hash.toggle_ep(ep_sq)
        }

        // Don't need to do any updates to the hash relating to the moved pieces
        // if we're playing a NULL move
        if mv == Move::NULL {
            new_history.clear();

            return Self {
                board: new_board,
                score: new_score.flipped(),
                hash: new_hash,
                history: new_history
            }
        }

        // In case of a non-NULL move, make the remaining changes to the hash

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

        // If castle: also account for the rook having moved
        if mv.is_castle() {
            let ctype = CastleType::from_move(mv).unwrap();
            let rook_move = ctype.rook_move();

            let old_rook = self.board.piece_list[rook_move.src() as usize]
                .unwrap();

            new_score.remove(us, old_rook, rook_move.src());
            new_hash.toggle_piece(old_rook, rook_move.src());

            // Add back value of new position. This takes care of promotions too
            let new_rook = new_board.piece_list[rook_move.tgt() as usize]
                .unwrap();

            new_score.add(us, new_rook, rook_move.tgt());
            new_hash.toggle_piece(new_rook, rook_move.tgt());
        }

        new_hash.toggle_castling(self.board.castling_rights);
        new_hash.toggle_castling(new_board.castling_rights);

        if old_piece.is_pawn() || mv.is_capture() {
            // A repetition can't happen, so reset the repetition history
            new_history.clear();
        }

        Self {
            board: new_board,
            score: new_score.flipped(),
            hash: new_hash,
            history: new_history,
        }
    }

    pub fn play_bare_move(&self, bare: BareMove) -> Self {
        let mv = self.board
            .find_move(bare)
            .expect("Not a legal move");

        self.play_move(mv)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chess::square::Square::*;
    use chess::movegen::moves::MoveType::*;
    use colored::Colorize;
    use crate::{tests::TEST_POSITIONS, position::Position};

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

    /// Test that, for all of the test suite, playing _every_ single legal move
    /// and incrementally updating the hash yields the same result as 
    /// hashing the resulting board from scratch.
    #[test]
    fn incremental_hashing() {
        let mut results: Vec<bool> = Vec::new();
        use crate::zobrist::Zobrist;

        for fen in TEST_POSITIONS {
            let board = fen.parse().unwrap();
            let position = Position::new(board);

            let all_match = board.legal_moves().iter()
                .map(|&mv| position.play_move(mv))
                .all(|new_pos| new_pos.hash == new_pos.board.hash());

            if all_match {
                println!("{}", fen.green());
            } else {
                println!("{}", fen.red());
            }

            results.push(all_match);
        }

        let all = TEST_POSITIONS.len();
        let passed = results.into_iter().filter(|&passed| passed).count();
        let failed = all - passed;

        println!(
            "{} passed, {} failed", 
            passed.to_string().green(), 
            failed.to_string().red()
        );

        assert_eq!(
            passed, 
            all, 
            "{} hashes came out different when updating incrementally", 
            failed.to_string().red()
        );
    }

    #[test]
    fn test_repetitions() {
        let board = "3k4/8/8/8/8/8/8/3K3P w - - 0 1".parse().unwrap();
        let mut position = Position::new(board);
        println!("{board}");

        position = position.play_move("d1e1".parse().unwrap());
        position = position.play_move("d8e8".parse().unwrap());
        position = position.play_move("e1d1".parse().unwrap());
        position = position.play_move("e8d8".parse().unwrap());
        assert!(position.is_repetition());
        assert!(position.history.len() == 4);
        position = position.play_move("h1h2".parse().unwrap());
        assert!(position.history.len() == 0);
    }

    #[test]
    fn test_faulty_position() {
        // From an actual game against Blunder 3.0
        let board = "8/4k3/1p5p/4PB2/1Pr3P1/1K1N4/8/8 b - - 4 53".parse().unwrap();
        let mut position = Position::new(board);
        let mv = position.board.find_move("b6b5".parse().unwrap()).unwrap();
        position = position.play_move(mv);
        assert!(position.history.is_empty());
        let mv = position.board.find_move("b3b2".parse().unwrap()).unwrap();
        position = position.play_move(mv);

        let mv = position.board.find_move("e7f7".parse().unwrap()).unwrap();
        position = position.play_move(mv);

        assert!(!position.is_repetition());

        let mv = position.board.find_move("b2b3".parse().unwrap()).unwrap();
        position = position.play_move(mv);

        let mv = position.board.find_move("f7e7".parse().unwrap()).unwrap();
        position = position.play_move(mv);
        assert!(position.is_repetition());

        let mv = position.board.find_move("b3b2".parse().unwrap()).unwrap();
        position = position.play_move(mv);
        assert!(position.is_repetition());
    }
}
