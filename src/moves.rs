use crate::board::{Piece, Board, PieceType };
use std::{iter::successors, str::FromStr};
use crate::bitboard::Bitboard;
use anyhow::anyhow;

impl Piece {
    pub fn pushes(&self, board: &Board) -> Bitboard {
        match self.piece_type {
            PieceType::Pawn => pawn_pushes(self, board),
            PieceType::Rook => rook_pushes(self, board),
            PieceType::Knight => knight_pushes(self, board),
            PieceType::Bishop => bishop_pushes(self, board),
            PieceType::Queen => queen_pushes(self, board),
            PieceType::King => king_pushes(self, board),
        }
    }

    pub fn attacks(&self, board: &Board) -> Bitboard {
        match self.piece_type {
            PieceType::Pawn => pawn_attacks(self, board),
            PieceType::Rook => rook_attacks(self, board),
            PieceType::Knight => knight_attacks(self, board),
            PieceType::Bishop => bishop_attacks(self, board),
            PieceType::Queen => queen_attacks(self, board),
            PieceType::King => king_attacks(self, board),
        }
    }

    pub fn legal_moves(&self, board: &Board) -> Bitboard {
        self.pushes(board).add(self.attacks(board))

    }
}

pub fn pawn_moves(pawn: &Piece) -> Bitboard {
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
    board.up_while_empty(&rook.position)
    .chain(board.left_while_empty(&rook.position))
    .chain(board.right_while_empty(&rook.position))
    .chain(board.down_while_empty(&rook.position))
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

pub fn knight_moves(knight: &Piece) -> Bitboard {
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

#[derive(Default, Clone, Copy, Debug)]
pub struct CastlingRights(u8);

impl CastlingRights {
    pub const WQ: CastlingRights = CastlingRights(0b0001);
    pub const WK: CastlingRights = CastlingRights(0b0010);
    pub const BQ: CastlingRights = CastlingRights(0b0100);
    pub const BK: CastlingRights = CastlingRights(0b1000);

    pub fn new() -> CastlingRights {
        CastlingRights(0b1111)
    }

    pub fn none() -> CastlingRights {
        CastlingRights(0)
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

impl FromStr for CastlingRights {
    type Err = anyhow::Error;

    /// Parse the castling rights from a FEN string 
    /// rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1
    ///                                               ^^^^
    fn from_str(fen: &str) -> Result<Self, Self::Err> {
        let mut rights = CastlingRights::none();
        let castling_str = fen.split(" ").nth(2).ok_or(anyhow!("Invalid FEN string"))?;

        for ch in castling_str.chars() {
            match ch {
                'Q' => rights.add(CastlingRights::WQ),
                'K' => rights.add(CastlingRights::WK),
                'q' => rights.add(CastlingRights::BQ),
                'k' => rights.add(CastlingRights::BK),
                '-' => {},
                _ => Err(anyhow!("Invalid FEN string"))?
            }
        }

        Ok(rights)
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

        assert_eq!(pawn_moves(&pawn), Bitboard::new(7,6));
    }

    #[test]
    fn test_unmoved_pawn_moves() {
        let pawn = Piece { 
            piece_type: PieceType::Pawn,
            color: Color::White,
            position: Bitboard::new(1,4),
            has_moved: false
        };

        assert_eq!(pawn_moves(&pawn), Bitboard::new(2,4).add(Bitboard::new(3,4)));
    }
}
