use crate::board::{Piece, Bitboard, Board, PieceType };
use std::iter::successors;

pub fn pushes(piece: &Piece, board: &Board) -> Bitboard {
    match piece.piece_type {
        PieceType::Pawn => pawn_pushes(piece, board),
        PieceType::Rook => rook_pushes(piece, board),
        _ => pawn_pushes(piece, board)
    }
}

pub fn attacks(piece: &Piece, board: &Board) -> Bitboard {
    match piece.piece_type {
        PieceType::Pawn => pawn_attacks(piece, board),
        PieceType::Rook => rook_attacks(piece, board),
        _ => pawn_pushes(piece, board)
    }
}


pub fn pawn_moves(piece: &Piece) -> Vec<Bitboard> {
    let moves = successors(
        piece.position.forward(piece.color), 
        |pos| pos.forward(piece.color)
    );

    if piece.has_moved {
        moves.take(1).collect()
    } else {
        moves.take(2).collect()
    }
}

pub fn pawn_pushes(piece: &Piece, board: &Board) -> Bitboard {
    pawn_moves(piece)
        .into_iter()
        .take_while(|position| board.get(position).is_none())
        .collect()
}

pub fn pawn_attacks(piece: &Piece, board: &Board) -> Bitboard {
    // Actually check whether the piece on the position is opponent
    vec![
        piece.position.forward(piece.color).and_then(|forward| forward.left()),
        piece.position.forward(piece.color).and_then(|forward| forward.right()),
    ]
        .into_iter()
        .flatten()
        .filter(|pos| board.has_colored_piece(pos, piece.color.opp()))
        .collect()
}

pub fn rook_pushes(piece: &Piece, board: &Board) -> Bitboard {
    board.up_while_empty(&piece.position).into_iter()
    .chain(board.left_while_empty(&piece.position).into_iter())
    .chain(board.right_while_empty(&piece.position).into_iter())
    .chain(board.down_while_empty(&piece.position).into_iter())
    .collect()
}

pub fn rook_attacks(piece: &Piece, board: &Board) -> Bitboard {
    board.first_piece_up(&piece.position).into_iter()
    .chain(board.first_piece_left(&piece.position).into_iter())
    .chain(board.first_piece_right(&piece.position).into_iter())
    .chain(board.first_piece_down(&piece.position).into_iter())
    .filter(|occupant| occupant.color == piece.color.opp())
    .map(|occupant| occupant.position)
    .collect()
}

#[cfg(test)]
mod tests {
    use crate::board::{PieceType, Color};

    use super::*;

    #[test]
    fn test_pawn_moves() {
        let pawn = Piece { 
            piece_type: PieceType::Pawn,
            color: Color::White,
            position: Bitboard::new(6,6),
            has_moved: true
        };

        assert_eq!(pawn_moves(&pawn), vec![Bitboard::new(7,6)]);
    }

    #[test]
    fn test_unmoved_pawn_moves() {
        let pawn = Piece { 
            piece_type: PieceType::Pawn,
            color: Color::White,
            position: Bitboard::new(1,4),
            has_moved: false
        };

        assert_eq!(pawn_moves(&pawn), vec![Bitboard::new(2,4), Bitboard::new(3,4)]);
    }
}
