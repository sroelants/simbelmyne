//! Most of the core logic concerning `Position`s lives in this module
//!
//! A `Position` is a wrapper around a `Board` that keeps track of some 
//! additional game data, that the chess backend doesn't have any knowledge of.
//! These are things such as evaluation, Zobrist hashing, and game history.

use arrayvec::ArrayVec;
use chess::{board::Board, movegen::{castling::CastleType, moves::{BareMove, Move}}, piece::{Color, Piece, PieceType}, square::Square};
use crate::zobrist::ZHash;

// We don't ever expect to exceed 100 entries, because that would be a draw.
const HIST_SIZE: usize = 100;

/// Wrapper around a `Board` that stores additional metadata that is not tied to
/// the board itself, but rather to the search and evaluation algorithms.
#[derive(Debug, Clone)]
pub struct Position {
    /// The board associated with the position.
    pub board: Board,

    /// The Zobrist hash of the current board
    pub hash: ZHash,

    /// The Zobrist hash of the current pawn structure
    /// Used for indexing the pawn cache, as well as pawn-based correction
    /// history.
    pub pawn_hash: ZHash,

    /// The Zobrist hash for non-pawn material
    /// Used for non-pawn correction history
    pub nonpawn_hashes: [ZHash; 2],

    /// A Zobrist-like key that keeps track of material count
    /// Used for material-based correction history
    pub material_hash: ZHash,

    /// A Zobrist-like key that keeps track of the minor piece structure
    /// Used for minor-piece correction history
    pub minor_hash: ZHash,

    /// A history of Zobrist hashes going back to the last half-move counter
    /// reset.
    pub history: ArrayVec<ZHash, HIST_SIZE>
}

impl Position {
    /// Create a new `Position` from a `Board`
    pub fn new(board: Board) -> Self {
        use Color::*;

        Position {
            board, 
            hash: ZHash::from(board),
            pawn_hash: ZHash::pawn_hash(&board),
            nonpawn_hashes: [ZHash::nonpawn_hash(&board, White), ZHash::nonpawn_hash(&board, Black)],
            material_hash: ZHash::material_hash(&board),
            minor_hash: ZHash::minor_hash(&board),
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
        use Square::*;
        use PieceType::*;
        let source = mv.src();
        let target = mv.tgt();
        let capture_sq = mv.get_capture_sq();
        let us = self.board.current;
        let mut new_board = self.board.clone();
        let mut new_hash = self.hash;
        let mut new_pawn_hash = self.pawn_hash;
        let mut new_nonpawn_hashes = self.nonpawn_hashes;
        let mut new_material_hash = self.material_hash;
        let mut new_minor_hash = self.minor_hash;
        assert!(mv != Move::NULL, "Tried processing a null move in `Position::play_move`");
 
        ////////////////////////////////////////////////////////////////////////
        //
        // Copture
        //
        ////////////////////////////////////////////////////////////////////////

        if mv.is_capture() {
            let captured = new_board.remove_at(capture_sq).unwrap();
            new_hash.toggle_piece(captured, capture_sq);

            // Update the hashes
            if captured.is_pawn() {
                new_pawn_hash.toggle_piece(captured, capture_sq);
            } else {
                new_nonpawn_hashes[!us].toggle_piece(captured, capture_sq);

                if matches!(captured.piece_type(), Knight | Bishop | King) {
                    new_minor_hash.toggle_piece(captured, capture_sq);
                }
            }

            // Decrement the material key for this piece
            let count = self.board.piece_bb(captured).count();
            new_material_hash.toggle_material(captured, count);
            new_material_hash.toggle_material(captured, count - 1);

            // Remove castling rights if captured piece is a rook on its 
            // original square
            if captured.is_rook() {
                match target {
                    A1 => new_board.castling_rights.remove(CastleType::WQ),
                    H1 => new_board.castling_rights.remove(CastleType::WK),
                    A8 => new_board.castling_rights.remove(CastleType::BQ),
                    H8 => new_board.castling_rights.remove(CastleType::BK),
                    _ => {}
                }
            }
        }

        ////////////////////////////////////////////////////////////////////////
        //
        // Move piece
        //
        ////////////////////////////////////////////////////////////////////////

        // Remove selected piece from board
        let old_piece = new_board.remove_at(source).unwrap();
        new_hash.toggle_piece(old_piece, source);

        // Figure out what piece to place at the target (considers promotions)
        let new_piece = mv.get_promo_piece(us).unwrap_or(old_piece);

        // Add the (new) piece to the board at the target square
        new_board.add_at(target, new_piece);
        new_hash.toggle_piece(new_piece, target);

        ////////////////////////////////////////////////////////////////////////
        //
        // Update the pawn/non-pawn/material hashes
        //
        ////////////////////////////////////////////////////////////////////////

        // Update the pawn and nonpawn hashes
        if old_piece.is_pawn() {
            new_pawn_hash.toggle_piece(old_piece, source);
        } else {
            new_nonpawn_hashes[us].toggle_piece(old_piece, source);

            if matches!(old_piece.piece_type(), Knight | Bishop | King) {
                new_minor_hash.toggle_piece(old_piece, source);
            }
        }

        if new_piece.is_pawn() {
            new_pawn_hash.toggle_piece(new_piece, target);
        } else {
            new_nonpawn_hashes[us].toggle_piece(new_piece, target);

            if matches!(new_piece.piece_type(), Knight | Bishop | King) {
                new_minor_hash.toggle_piece(new_piece, target);
            }
        }

        // Update the material hash
        if old_piece != new_piece {
            // Decrement the material key for the old piece
            let count = self.board.piece_bb(old_piece).count();
            new_material_hash.toggle_material(old_piece, count);
            new_material_hash.toggle_material(old_piece, count - 1);

            // Increment the material key for the new piece
            let count = self.board.piece_bb(new_piece).count();
            new_material_hash.toggle_material(new_piece, count);
            new_material_hash.toggle_material(new_piece, count + 1);
        }


        ////////////////////////////////////////////////////////////////////////
        //
        // Castling
        //
        ////////////////////////////////////////////////////////////////////////

        // If castle: also account for the rook having moved
        if mv.is_castle() {
            // In case of castle, also move the rook to the appropriate square
            let ctype = CastleType::from_move(mv).unwrap();
            let rook_move = ctype.rook_move();
            let rook_src = rook_move.src();
            let rook_tgt = rook_move.tgt();
            let rook = new_board.remove_at(rook_src).unwrap();
            new_board.add_at(rook_tgt, rook);

            // Update the hash
            new_hash.toggle_piece(rook, rook_src);
            new_hash.toggle_piece(rook, rook_tgt);

            new_nonpawn_hashes[us].toggle_piece(rook, rook_src);
            new_nonpawn_hashes[us].toggle_piece(rook, rook_tgt);
        }

        if old_piece.is_king() {
            if us.is_white() {
                new_board.castling_rights.remove(CastleType::WQ);
                new_board.castling_rights.remove(CastleType::WK);
            } else {
                new_board.castling_rights.remove(CastleType::BQ);
                new_board.castling_rights.remove(CastleType::BK);
            } 
        } else if old_piece.is_rook() {
            match source {
                A1 => new_board.castling_rights.remove(CastleType::WQ),
                H1 => new_board.castling_rights.remove(CastleType::WK),
                A8 => new_board.castling_rights.remove(CastleType::BQ),
                H8 => new_board.castling_rights.remove(CastleType::BK),
                _ => {}
            }
        }

        // Invalidate the previous castling rights, even if the move wasn't a 
        // castle.
        // FIXME: Improve this
        new_hash.toggle_castling(self.board.castling_rights);
        new_hash.toggle_castling(new_board.castling_rights);

        ////////////////////////////////////////////////////////////////////////
        //
        // En-passant
        //
        ////////////////////////////////////////////////////////////////////////

        // Should we unset the old EP square?
        if let Some(ep_sq) = self.board.en_passant {
            new_board.en_passant = None;
            new_hash.toggle_ep(ep_sq)
        }

        // Should we set a new EP square?
        if mv.is_double_push() {
            let ep_sq = target.backward(us).unwrap();
            new_board.en_passant = Some(ep_sq);
            new_hash.toggle_ep(ep_sq)
        }

        ////////////////////////////////////////////////////////////////////////
        //
        // Update state variables
        //
        ////////////////////////////////////////////////////////////////////////

        // Update playing side
        new_board.current = !us;
        new_hash.toggle_side();

        // Update move counter
        if us.is_black() {
            new_board.full_moves += 1;
        }

        // Update half-move clock and repetition history
        let mut new_history;

        if old_piece.is_pawn() || mv.is_capture() {
            new_board.half_moves = 0;
            new_history = ArrayVec::new();
        } else {
            new_board.half_moves += 1;
            new_history = self.history.clone();
            new_history.push(self.hash);
        }

        ////////////////////////////////////////////////////////////////////////
        //
        // Update auxiliary bitboards (pins, checkers, threats)
        //
        ////////////////////////////////////////////////////////////////////////

        new_board.hv_pinrays = [
            new_board.compute_hv_pinrays::<true>(), 
            new_board.compute_hv_pinrays::<false>()
        ];

        new_board.diag_pinrays = [
            new_board.compute_diag_pinrays::<true>(), 
            new_board.compute_diag_pinrays::<false>()
        ];

        new_board.checkers = new_board.compute_checkers();
        new_board.threats = new_board.attacked_squares(!new_board.current);

        Self {
            board: new_board,
            hash: new_hash,
            pawn_hash: new_pawn_hash,
            nonpawn_hashes: new_nonpawn_hashes,
            material_hash: new_material_hash,
            minor_hash: new_minor_hash,
            history: new_history,
        }
    }

    pub fn play_null_move(&self) -> Self {
        let us = self.board.current;
        let mut new_board = self.board.clone();
        let mut new_hash = self.hash;
        let new_history = ArrayVec::new();

        ////////////////////////////////////////////////////////////////////////
        //
        // Update state variables
        //
        ////////////////////////////////////////////////////////////////////////

        // Update player
        new_board.current = !us;
        new_hash.toggle_side();

        // Un-set the old en-passant square
        if let Some(ep_sq) = self.board.en_passant {
            new_board.en_passant = None;
            new_hash.toggle_ep(ep_sq)
        }

        // Update half-move counter
        new_board.half_moves += 1;

        // Update move counter
        if us.is_black() {
            new_board.full_moves += 1;
        }

        new_board.checkers = new_board.compute_checkers();
        new_board.threats = new_board.attacked_squares(!new_board.current);

        Self {
            board: new_board,
            hash: new_hash,
            pawn_hash: self.pawn_hash,
            nonpawn_hashes: self.nonpawn_hashes,
            material_hash: self.material_hash,
            minor_hash: self.minor_hash,
            history: new_history
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

    /// Return a first approximation of the Zobrist hash after playing the 
    /// provided move.
    ///
    /// This method tries to be fast over correct, so the hash will not match 
    /// in certain situations.
    ///
    /// In particular, castling rights are not updated whatsoever.
    pub fn approx_hash_after(&self, mv: Move) -> ZHash {
        let mut new_hash = self.hash;

        // Update playing side
        new_hash.toggle_side();

        // Remove the old piece
        let old_piece = self.board.piece_list[mv.src()]
            .expect("The source target of a move has a piece");

        new_hash.toggle_piece(old_piece, mv.src());

        // Add the new piece, taking promotions into account
        if let Some(promo_type) = mv.get_promo_type() {
            let new_piece = Piece::new(promo_type, self.board.current);
            new_hash.toggle_piece(new_piece, mv.tgt());
        } else {
            new_hash.toggle_piece(old_piece, mv.tgt());
        }

        // Remove any captured pieces
        if mv.is_capture() {
            let captured_sq = mv.get_capture_sq();
            let captured = self.board.get_at(captured_sq)
                .expect("Move is a capture, so must have piece on target");
 
            new_hash.toggle_piece(captured, captured_sq);
        }

        // Un-set the _old_ en-passant square
        if let Some(ep_sq) = self.board.en_passant {
            new_hash.toggle_ep(ep_sq)
        }

        // If there is a new en-passant square, toggle it.
        if mv.is_double_push() {
            if let Some(ep_sq) = mv.tgt().backward(self.board.current) {
                new_hash.toggle_ep(ep_sq)
            }
        }

        new_hash
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
    use chess::{movegen::legal_moves::All, square::Square};
    use chess::square::Square::*;
    use chess::movegen::moves::MoveType::{self, *};
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

            let all_match = board.legal_moves::<All>().into_iter()
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

    #[test]
    fn test_pawn_hash() {
        let pos1 = Position::new("rnbqkbnr/ppp1pppp/3p4/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2".parse().unwrap());
        let pos2 = Position::new("r1bqkbnr/ppp1pppp/2np4/8/2B1P3/8/PPPP1PPP/RNBQK1NR w KQkq - 2 3".parse().unwrap());

        assert_eq!(pos1.pawn_hash, pos2.pawn_hash);
    }

    #[test]
    fn test_incremental_pawn_hash() {
        use Square::*;
        use MoveType::*;

        let initial = Position::new("rnbqkbnr/ppp1pppp/3p4/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2".parse().unwrap());
        let terminal = Position::new("rnbqkb1r/ppp1pppp/3p1n2/8/4P3/3P4/PPP2PPP/RNBQKBNR w KQkq - 1 3".parse().unwrap());

        let terminal_inc = initial
            .play_move(Move::new(D2, D3, Quiet))
            .play_move(Move::new(G8, F6, Quiet));

        assert_eq!(terminal_inc.pawn_hash, terminal.pawn_hash);
    }
}
