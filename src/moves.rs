use crate::board::{Piece, Bitboard, Board, PieceType };
use std::iter::successors;

pub fn pushes(piece: &Piece, board: &Board) -> Bitboard {
    match piece.piece_type {
        PieceType::Pawn => pawn_pushes(piece, board),
        PieceType::Rook => rook_pushes(piece, board),
        PieceType::Knight => knight_pushes(piece, board),
        PieceType::Bishop => bishop_pushes(piece, board),
        PieceType::Queen => queen_pushes(piece, board),
        PieceType::King => king_pushes(piece, board),
    }
}

pub fn attacks(piece: &Piece, board: &Board) -> Bitboard {
    match piece.piece_type {
        PieceType::Pawn => pawn_attacks(piece, board),
        PieceType::Rook => rook_attacks(piece, board),
        PieceType::Knight => knight_attacks(piece, board),
        PieceType::Bishop => bishop_attacks(piece, board),
        PieceType::Queen => queen_attacks(piece, board),
        PieceType::King => king_attacks(piece, board),
    }
}

pub fn pawn_moves(pawn: &Piece) -> Vec<Bitboard> {
    let moves = successors(
        pawn.position.forward(pawn.color), 
        |pos| pos.forward(pawn.color)
    );

    if pawn.has_moved {
        moves.take(1).collect()
    } else {
        moves.take(2).collect()
    }
}

pub fn pawn_pushes(pawn: &Piece, board: &Board) -> Bitboard {
    pawn_moves(pawn)
        .into_iter()
        .take_while(|position| board.get(position).is_none())
        .collect()
}

pub fn pawn_attacks(pawn: &Piece, board: &Board) -> Bitboard {
    vec![
        pawn.position.forward(pawn.color).and_then(|forward| forward.left()),
        pawn.position.forward(pawn.color).and_then(|forward| forward.right()),
    ]
        .into_iter()
        .flatten()
        .filter(|pos| board.has_colored_piece(pos, pawn.color.opp()))
        .collect()
}

pub fn rook_pushes(rook: &Piece, board: &Board) -> Bitboard {
    board.up_while_empty(&rook.position).into_iter()
    .chain(board.left_while_empty(&rook.position).into_iter())
    .chain(board.right_while_empty(&rook.position).into_iter())
    .chain(board.down_while_empty(&rook.position).into_iter())
    .collect()
}

pub fn rook_attacks(rook: &Piece, board: &Board) -> Bitboard {
    board.first_piece_up(&rook.position).into_iter()
    .chain(board.first_piece_left(&rook.position).into_iter())
    .chain(board.first_piece_right(&rook.position).into_iter())
    .chain(board.first_piece_down(&rook.position).into_iter())
    .filter(|occupant| occupant.color == rook.color.opp())
    .map(|occupant| occupant.position)
    .collect()
}

pub fn knight_moves(knight: &Piece) -> Vec<Bitboard> {
    vec![
        knight.position.up().and_then(|pos| pos.up()).and_then(|pos| pos.left()),
        knight.position.up().and_then(|pos| pos.up()).and_then(|pos| pos.right()),
        knight.position.down().and_then(|pos| pos.down()).and_then(|pos| pos.left()),
        knight.position.down().and_then(|pos| pos.down()).and_then(|pos| pos.right()),
        knight.position.left().and_then(|pos| pos.left()).and_then(|pos| pos.up()),
        knight.position.left().and_then(|pos| pos.left()).and_then(|pos| pos.down()),
        knight.position.right().and_then(|pos| pos.right()).and_then(|pos| pos.up()),
        knight.position.right().and_then(|pos| pos.right()).and_then(|pos| pos.down()),
    ].into_iter().flatten().collect()
}

pub fn knight_pushes(knight: &Piece, board: &Board) -> Bitboard {
    knight_moves(knight)
        .into_iter()
        .filter(|pos| board.is_empty(pos))
        .collect()
}

pub fn knight_attacks(knight: &Piece, board: &Board) -> Bitboard {
    knight_moves(knight)
        .into_iter()
        .filter(|pos| board.has_colored_piece(pos, knight.color.opp()))
        .collect()
}

pub fn bishop_pushes(bishop: &Piece, board: &Board) -> Bitboard {
    board.scan_empty(&bishop.position, |pos| pos.up_left()).into_iter()
        .chain(board.scan_empty(&bishop.position, |pos| pos.up_right()).into_iter())
        .chain(board.scan_empty(&bishop.position, |pos| pos.down_left()).into_iter())
        .chain(board.scan_empty(&bishop.position, |pos| pos.down_right()).into_iter())
        .collect()
}

pub fn bishop_attacks(bishop: &Piece, board: &Board) -> Bitboard {
    board.first_piece(&bishop.position, |pos| pos.up_left()).into_iter()
        .chain(board.first_piece(&bishop.position, |pos| pos.up_right()).into_iter())
        .chain(board.first_piece(&bishop.position, |pos| pos.down_left()).into_iter())
        .chain(board.first_piece(&bishop.position, |pos| pos.down_right()).into_iter())
        .filter(|occupant| bishop.color == occupant.color.opp())
        .map(|piece| piece.position)
        .collect()
}

pub fn queen_pushes(queen: &Piece, board: &Board) -> Bitboard {
    rook_pushes(queen, board).add(bishop_pushes(queen, board))
}

pub fn queen_attacks(queen: &Piece, board: &Board) -> Bitboard {
    rook_attacks(queen, board).add(bishop_attacks(queen, board))
}

pub fn king_moves(king: &Piece) -> Vec<Bitboard> {
    vec![
        king.position.up(),
        king.position.up_left(),
        king.position.left(),
        king.position.down_left(),
        king.position.down(),
        king.position.down_right(),
        king.position.right(),
        king.position.up_right(),
    ].into_iter().flatten().collect()
}

pub fn king_pushes(king: &Piece, board: &Board) -> Bitboard {
    king_moves(king).into_iter().filter(|pos| board.is_empty(pos)).collect()
}

pub fn king_attacks(king: &Piece, board: &Board) -> Bitboard {
    king_moves(king)
        .into_iter()
        .filter(|pos| board.has_colored_piece(pos, king.color.opp()))
        .collect()
}

//TODO: Check that this is private?
#[derive(Default, Clone, Copy, Debug)]
pub struct CastlingRights(u8);

impl CastlingRights {
    pub const WQ: u8 = 0b0001;
    pub const WK: u8 = 0b0010;
    pub const BQ: u8 = 0b0100;
    pub const BK: u8 = 0b1000;

    pub fn new() -> CastlingRights {
        CastlingRights(0b1111)
    }

    pub fn add(&mut self, castle: CastlingRights) {
        self.0 = self.0 | castle.0;
    }

    pub fn remove(&mut self, castle: CastlingRights) {
        self.0 = self.0 & !castle.0;
    }

    pub fn toggle(&mut self, castle: CastlingRights) {
        self.0 = self.0 ^ castle.0;
    }
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
