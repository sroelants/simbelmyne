//! Most of the core logic concerning `Position`s lives in this module
//!
//! A `Position` is a wrapper around a `Board` that keeps track of some 
//! additional game data, that the chess backend doesn't have any knowledge of.
//! These are things such as evaluation, Zobrist hashing, and game history.

use arrayvec::ArrayVec;
use chess::{board::Board, movegen::{castling::CastleType, moves::{BareMove, Move}}, piece::{Color, Piece}};
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
    pub pawn_hash: ZHash,

    /// The Zobrist hash for non-pawn material
    pub nonpawn_hashes: [ZHash; 2],

    /// A Zobrist-like key that keeps track of material count
    pub material_hash: ZHash,

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
        let mut new_pawn_hash = self.pawn_hash;
        let mut new_nonpawn_hashes = self.nonpawn_hashes;
        let mut new_material_hash = self.material_hash;

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
                hash: self.hash,
                pawn_hash: self.pawn_hash,
                nonpawn_hashes: self.nonpawn_hashes,
                material_hash: self.material_hash,
                history: new_history
            }
        }

        ////////////////////////////////////////////////////////////////////////
        //
        // Coptures
        //
        ////////////////////////////////////////////////////////////////////////
        let captured_sq = mv.get_capture_sq();
        let captured = self.board.get_at(captured_sq);

        if mv.is_capture() {
            let captured = captured.unwrap();
 
            new_hash.toggle_piece(captured, captured_sq);

            // Decrement the material key for this piece
            let count = self.board.piece_bb(captured).count();
            new_material_hash.toggle_material(captured, count);
            new_material_hash.toggle_material(captured, count - 1);

            if captured.is_pawn() {
                new_pawn_hash.toggle_piece(captured, captured_sq);
            } else {
                let color = captured.color();
                new_nonpawn_hashes[color].toggle_piece(captured, captured_sq);
            }
        }

        ////////////////////////////////////////////////////////////////////////
        //
        // Move piece
        //
        ////////////////////////////////////////////////////////////////////////

        let old_piece = self.board.piece_list[mv.src()]
            .expect("The source target of a move has a piece");

        // note: might be different from original piece because of promotion
        let new_piece = new_board.piece_list[mv.tgt()]
          .expect("The target square of a move is occupied after playing");

        // Update the hash
        new_hash.toggle_piece(old_piece, mv.src());
        new_hash.toggle_piece(new_piece, mv.tgt());

        // Update the pawn and nonpawn hashes
        if old_piece.is_pawn() {
            new_pawn_hash.toggle_piece(old_piece, mv.src());
        } else {
            let color = old_piece.color();
            new_nonpawn_hashes[color].toggle_piece(old_piece, mv.src());
        }

        if new_piece.is_pawn() {
            new_pawn_hash.toggle_piece(new_piece, mv.tgt());
        } else {
            let color = new_piece.color();
            new_nonpawn_hashes[color].toggle_piece(new_piece, mv.tgt());
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
            let ctype = CastleType::from_move(mv).unwrap();
            let rook_src = self.board.castling_rights[ctype].unwrap();
            let rook_tgt = ctype.rook_target();
            let rook = self.board.piece_list[rook_src]
                .expect("We know there is a rook at the starting square");

            // Update the hash
            new_hash.toggle_piece(rook, rook_src);
            new_hash.toggle_piece(rook, rook_tgt);
            new_nonpawn_hashes[self.board.current].toggle_piece(rook, rook_src);
            new_nonpawn_hashes[self.board.current].toggle_piece(rook, rook_tgt);
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
            hash: new_hash,
            pawn_hash: new_pawn_hash,
            nonpawn_hashes: new_nonpawn_hashes,
            material_hash: new_material_hash,
            history: new_history,
        }
    }

    pub fn play_null_move(&self) -> Self {
        let new_history = ArrayVec::new();
 
        // Update board
        let new_board = self.board.play_null_move();

        ////////////////////////////////////////////////////////////////////////
        //
        // Update state variables
        //
        ////////////////////////////////////////////////////////////////////////

        let mut new_hash = self.hash;

        // Update playing side
        new_hash.toggle_side();

        // Un-set the old en-passant square
        if let Some(ep_sq) = self.board.en_passant {
            new_hash.toggle_ep(ep_sq)
        }

        Self {
            board: new_board,
            hash: new_hash,
            pawn_hash: self.pawn_hash,
            nonpawn_hashes: self.nonpawn_hashes,
            material_hash: self.material_hash,
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
    
}
