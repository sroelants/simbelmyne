use crate::{board::{Piece, Board, PieceType, Color }, movegen::moves::Move};
use std::iter::successors;
use crate::bitboard::Bitboard;

impl Piece {
    pub fn pushes(&self, board: &Board) -> Vec<Move> {
        match self.piece_type {
            PieceType::Pawn => pawn_pushes(self.position, self.color, board),
            PieceType::Rook => rook_pushes(self.position, board),
            PieceType::Knight => knight_pushes(self.position, board),
            PieceType::Bishop => bishop_pushes(self.position, board),
            PieceType::Queen => queen_pushes(self.position, board),
            PieceType::King => king_pushes(self.position, board),
        }
    }

    pub fn attacks(&self, board: &Board) -> Vec<Move> {
        match self.piece_type {
            PieceType::Pawn => pawn_attacks(self.position, self.color, board),
            PieceType::Rook => rook_attacks(self.position, self.color, board),
            PieceType::Knight => knight_attacks(self.position, self.color, board),
            PieceType::Bishop => bishop_attacks(self.position, self.color, board),
            PieceType::Queen => queen_attacks(self.position, self.color, board),
            PieceType::King => king_attacks(self.position, self.color, board),
        }
    }

    pub fn legal_moves(&self, board: &Board) -> Vec<Move> {
        let mut moves = Vec::new();

        moves.append(&mut self.pushes(board));
        moves.append(&mut self.attacks(board));

        if self.piece_type == PieceType::King {
            moves.append(&mut castles(self.color, board));
        }

        moves
    }

    pub fn attacked_squares(&self, board: &Board) -> Bitboard {
        self.legal_moves(board).iter().map(|mv| mv.tgt()).collect()
    }
}

pub fn pawn_moves(position: Bitboard, side: Color) -> Vec<Move> {
    let moves = successors(Some(position), |pos| pos.forward(side))
        .skip(1)
        .map(|target| Move::new(position, target));

    if position.within(&Bitboard::PAWN_RANKS[side as usize]) {
        moves.take(1).collect()
    } else {
        moves.take(2).collect()
    }
}

pub fn pawn_pushes(position: Bitboard, side: Color, board: &Board) -> Vec<Move> {
    pawn_moves(position, side)
        .into_iter()
        .take_while(|mov| board.get(&mov.tgt()).is_none())
        .collect()
}

pub fn pawn_attacks(position: Bitboard, side: Color, board: &Board) -> Vec<Move> {
    vec![
        position.forward(side).and_then(|forward| forward.left()),
        position.forward(side).and_then(|forward| forward.right()),
    ]
        .into_iter()
        .flatten()
        .filter(|pos| board.has_colored_piece(pos, side.opp()))
        .map(|target| Move::new(position, target))
        .collect()
}

pub fn rook_pushes(position: Bitboard, board: &Board) -> Vec<Move> {
    board.up_while_empty(&position)
    .chain(board.down_while_empty(&position))
    .chain(board.left_while_empty(&position))
    .chain(board.right_while_empty(&position))
    .map(|target| Move::new(position, target))
    .collect()
}

pub fn rook_attacks(position: Bitboard, side: Color, board: &Board) -> Vec<Move> {
    vec![
        board.first_piece_up(&position),
        board.first_piece_down(&position),
        board.first_piece_left(&position),
        board.first_piece_right(&position),
    ]
        .into_iter()
        .flatten()
        .filter(|occupant| occupant.color == side.opp())
        .map(|occupant| Move::new(position, occupant.position))
        .collect()
}

pub fn knight_moves(position: Bitboard) -> Vec<Move> {
    vec![
        position.up().and_then(|pos| pos.up()).and_then(|pos| pos.left()),
        position.up().and_then(|pos| pos.up()).and_then(|pos| pos.right()),
        position.down().and_then(|pos| pos.down()).and_then(|pos| pos.left()),
        position.down().and_then(|pos| pos.down()).and_then(|pos| pos.right()),
        position.left().and_then(|pos| pos.left()).and_then(|pos| pos.up()),
        position.left().and_then(|pos| pos.left()).and_then(|pos| pos.down()),
        position.right().and_then(|pos| pos.right()).and_then(|pos| pos.up()),
        position.right().and_then(|pos| pos.right()).and_then(|pos| pos.down()),
    ].into_iter().flatten().map(|target| Move::new(position, target)).collect()
}

pub fn knight_pushes(position: Bitboard, board: &Board) -> Vec<Move> {
    knight_moves(position)
        .into_iter()
        .filter(|mov| board.is_empty(&mov.tgt()))
        .collect()
}

pub fn knight_attacks(position: Bitboard, side: Color, board: &Board) -> Vec<Move> {
    knight_moves(position)
        .into_iter()
        .filter(|mov| board.has_colored_piece(&mov.tgt(), side.opp()))
        .collect()
}

pub fn bishop_pushes(position: Bitboard, board: &Board) -> Vec<Move> {
    board.scan_empty(&position, |pos| pos.up_left()).into_iter()
        .chain(board.scan_empty(&position, |pos| pos.up_right()).into_iter())
        .chain(board.scan_empty(&position, |pos| pos.down_left()).into_iter())
        .chain(board.scan_empty(&position, |pos| pos.down_right()).into_iter())
        .map(|target| Move::new(position, target))
        .collect()
}

pub fn bishop_attacks(position: Bitboard, side: Color, board: &Board) -> Vec<Move> {
    board.first_piece(&position, |pos| pos.up_left()).into_iter()
        .chain(board.first_piece(&position, |pos| pos.up_right()).into_iter())
        .chain(board.first_piece(&position, |pos| pos.down_left()).into_iter())
        .chain(board.first_piece(&position, |pos| pos.down_right()).into_iter())
        .filter(|occupant| occupant.color == side.opp())
        .map(|piece| Move::new(position, piece.position))
        .collect()
}

pub fn queen_pushes(position: Bitboard, board: &Board) -> Vec<Move> {
    let mut moves = Vec::new();
    moves.append(&mut rook_pushes(position, board));
    moves.append(&mut bishop_pushes(position, board));
    moves
}

pub fn queen_attacks(position: Bitboard, side: Color, board: &Board) -> Vec<Move> {
    let mut moves = Vec::new();
    moves.append(&mut rook_attacks(position, side, board));
    moves.append(&mut bishop_attacks(position, side, board));
    moves
}

pub fn king_moves(position: Bitboard) -> Vec<Move> {
    vec![
        position.up(),
        position.up_left(),
        position.left(),
        position.down_left(),
        position.down(),
        position.down_right(),
        position.right(),
        position.up_right(),
    ].into_iter().flatten().map(|target| Move::new(position, target)).collect()
}

pub fn king_pushes(position: Bitboard, board: &Board) -> Vec<Move> {
    king_moves(position)
        .into_iter()
        .filter(|mov| board.is_empty(&mov.tgt()))
        .collect()
}

pub fn castles(side: Color, board: &Board) -> Vec<Move> {
    board.castling_rights.get_available(side)
        .into_iter()
        .filter(|ctype| ctype.is_allowed(board))
        .map(|ctype| ctype.king_move())
        .collect()
}

pub fn king_attacks(position: Bitboard, side: Color, board: &Board) -> Vec<Move> {
    king_moves(position)
        .into_iter()
        .filter(|mov| board.has_colored_piece(&mov.tgt(), side.opp()))
        .collect()
}
