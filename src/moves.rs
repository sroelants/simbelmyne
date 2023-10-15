use crate::board::{Board, Piece, PositionSet, Bitboard};

pub fn pawn_pushes(piece: &Piece,  board: &Board) -> PositionSet {
    let mut moves = PositionSet::default();

    if piece.is_white() {
        let Some(up) = piece.position.up() else { return moves; };
        if board.get(up).is_some() { return moves; } else { moves.add(up) };
        if piece.has_moved { return moves; }

        // If piece hasn't moved, and there's no piece directly above, we enter
        // phase two: double pawn moves!
        let Some(two_up) = up.up() else { return moves; };
        if board.get(two_up).is_some() { return moves; } else { moves.add(up) };

    } else {
        let Some(down) = piece.position.down() else { return moves; };
        if board.get(down).is_some() { return moves; } else { moves.add(down) };
        if piece.has_moved { return moves; }

        // If piece hasn't moved, and there's no piece directly above, we enter
        // phase two: double pawn moves!
        let Some(two_down) = down.down() else { return moves; };
        if board.get(two_down).is_some() { return moves; } else { moves.add(down) };
    }

    moves
}

// What would that look like?
