use crate::board::{Board, Piece, PositionSet, Bitboard};

pub fn pawn_moves(piece: Piece,  board: Board) -> PositionSet {
    let mut moves: PositionSet = PositionSet(0);

    if piece.is_white() {
        if let Some(up) = piece.position.up() { moves.add(up); }

        if !piece.has_moved {
            let two_up = piece.position.up().and_then(|up| up.up());
            if let Some(two_up) = two_up { moves.add(two_up); }
        }
    } else {
        if let Some(down) = piece.position.down() {
            moves.add(down);
        }

        if !piece.has_moved {
            let two_down = piece.position.down().and_then(|down| down.down());
            if let Some(two_down) = two_down { moves.add(two_down); }
        }
    }

    moves
}
