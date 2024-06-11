//! Most of the core logic concerning `Position`s lives in this module
//!
//! A `Position` is a wrapper around a `Board` that keeps track of some 
//! additional game data, that the chess backend doesn't have any knowledge of.
//! These are things such as evaluation, Zobrist hashing, and game history.

use arrayvec::ArrayVec;
use chess::{board::Board, movegen::{moves::{Move, BareMove}, castling::CastleType}};
use crate::{evaluate::Eval, zobrist::ZHash};

// We don't ever expect to exceed 100 entries, because that would be a draw.
const HIST_SIZE: usize = 100;

/// Wrapper around a `Board` that stores additional metadata that is not tied to
/// the board itself, but rather to the search and evaluation algorithms.
#[derive(Debug, Clone)]
pub struct Position {
    /// The board associated with the position.
    pub board: Board,

    /// The score object associated with the position.
    pub score: Eval,

    /// The Zobrist hash of the current board
    pub hash: ZHash,

    /// A history of Zobrist hashes going back to the last half-move counter
    /// reset.
    pub history: ArrayVec<ZHash, HIST_SIZE>
}

impl Position {
    /// Create a new `Position` from a `Board`
    pub fn new(board: Board) -> Self {
        Position {
            board, 
            score: Eval::new(&board),
            hash: ZHash::from(board),
            history: ArrayVec::new(),
        }
    }

    /// Check whether the current board state is a repetition by going through 
    /// the history list. The history list tends to be fairly short, so it's not
    /// as expensive as it sounds.
    pub fn is_repetition(&self) -> bool {
        self.history.iter()
            // Look through the history backwards
            .rev()

            // Skip the position the opponent just played
            .skip(1)      

            // In fact, skip every other board position, since they can't be
            // repetitions
            .step_by(2)   

            // Check if the zobrist hash matches to indicate a repetition
            .any(|&historic| historic == self.hash)
    }


    /// Play a move and update the board, scores and hashes accordingly.
    pub fn play_move(&self, mv: Move) -> Self {
        let mut new_score = self.score;
        let mut new_history = self.history.clone();
 
        // Update board
        let new_board = self.board.play_move(mv);

        ////////////////////////////////////////////////////////////////////////
        //
        // Update History
        //
        // If the move is a null move, clear the history. Null moves are 
        // considered "irreversible", otherwise they would lead to a ton of 
        // ficticious draws by repetition.
        //
        ////////////////////////////////////////////////////////////////////////

        if mv == Move::NULL {
            new_history.clear();
        } else {
            new_history.push(self.hash);
        }

        ////////////////////////////////////////////////////////////////////////
        //
        // Update state variables
        //
        // Make any updates to hash that don't depend on the move first
        // (In case of a NULL move, we want to cut this function short)
        //
        ////////////////////////////////////////////////////////////////////////
        let mut new_hash = self.hash;

        // Update playing side
        new_hash.toggle_side();

        // Un-set the _old_ en-passant square
        if let Some(ep_sq) = self.board.en_passant {
            new_hash.toggle_ep(ep_sq)
        }

        // If there is a new en-passant square, toggle it.
        if let Some(ep_sq) = new_board.en_passant {
            new_hash.toggle_ep(ep_sq)
        }

        // Don't need to do any updates to the hash relating to the moved pieces
        // if we're playing a NULL move 👋
        if mv == Move::NULL {
            return Self {
                board: new_board,
                score: new_score,
                hash: new_hash,
                history: new_history
            }
        }

        ////////////////////////////////////////////////////////////////////////
        //
        // Coptures
        //
        ////////////////////////////////////////////////////////////////////////

        if mv.is_capture() {
            let captured_sq = mv.get_capture_sq();
            let captured = self.board.get_at(captured_sq)
                .expect("Move is a capture, so must have piece on target");
 
            new_score.remove(captured, captured_sq, &new_board);
            new_hash.toggle_piece(captured, captured_sq);
        }

        ////////////////////////////////////////////////////////////////////////
        //
        // Move piece
        //
        ////////////////////////////////////////////////////////////////////////

        let old_piece = self.board.piece_list[mv.src() as usize]
            .expect("The source target of a move has a piece");

        // note: might be different from original piece because of promotion
        let new_piece = new_board.piece_list[mv.tgt() as usize]
          .expect("The target square of a move is occupied after playing");

        // Update the score
        if mv.is_promotion() {
            new_score.remove(old_piece, mv.src(), &new_board);
            new_score.add(new_piece, mv.tgt(), &new_board);
        } else {
            new_score.update(old_piece, mv.src(), mv.tgt(), &new_board);
        }

        // Update the hash
        new_hash.toggle_piece(old_piece, mv.src());
        new_hash.toggle_piece(new_piece, mv.tgt());

        ////////////////////////////////////////////////////////////////////////
        //
        // Castling
        //
        ////////////////////////////////////////////////////////////////////////

        // If castle: also account for the rook having moved
        if mv.is_castle() {
            let ctype = CastleType::from_move(mv).unwrap();
            let rook_move = ctype.rook_move();
            let rook = self.board.piece_list[rook_move.src() as usize]
                .expect("We know there is a rook at the starting square");

            // Update the score
            new_score.update(rook, rook_move.src(), rook_move.tgt(), &new_board);

            // Update the hash
            new_hash.toggle_piece(rook, rook_move.src());
            new_hash.toggle_piece(rook, rook_move.tgt());
        }

        // Invalidate the previous castling rights, even if the move wasn't a 
        // castle.

        // Remove the old castling rights from the hash
        new_hash.toggle_castling(self.board.castling_rights);

        // Add in the current castling rights 
        new_hash.toggle_castling(new_board.castling_rights);

        ////////////////////////////////////////////////////////////////////////
        //
        // Update history, part 2
        //
        // Clear the history table if this move was irreversible
        //
        ////////////////////////////////////////////////////////////////////////

        if old_piece.is_pawn() || mv.is_capture() {
            new_history.clear();
        }

        Self {
            board: new_board,
            score: new_score,
            hash: new_hash,
            history: new_history,
        }
    }

    /// Play a bare move
    ///
    /// Given a bare move, try and find a legal move that corresponds to it, and
    /// play it. Panics if the bare move didn't correspond to a legal move!
    pub fn play_bare_move(&self, bare: BareMove) -> Self {
        let mv = self.board
            .find_move(bare)
            .expect("Not a legal move");

        self.play_move(mv)
    }
}

////////////////////////////////////////////////////////////////////////////////
//
// Tests
//
////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;
    use chess::square::Square::*;
    use chess::movegen::moves::MoveType::*;
    use colored::Colorize;
    use crate::{tests::TEST_POSITIONS, position::Position};
    const QUIETS: bool = true;

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

            let all_match = board.legal_moves::<QUIETS>().into_iter()
                .map(|mv| position.play_move(mv))
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
