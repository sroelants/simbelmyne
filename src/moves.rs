use crate::{board::{Piece, Board, PieceType, Color }, movegen::{moves::Move, castling::CastlingRights}, bitboard::Step};
use std::iter::successors;
use crate::bitboard::Bitboard;
use itertools::Itertools;


impl Piece {
    fn on_pawn_rank(&self) -> bool {
        Bitboard::PAWN_RANKS[self.color as usize].contains(self.position)
    }

    pub fn range(&self) -> usize {
        use PieceType::*;

        match self.piece_type {
            Pawn | Knight | King => 1,
            _ => 7
        }
    }

    pub fn directions(&self) -> Vec<Step> {
        use PieceType::*;

        match self.piece_type {
            Pawn => vec![
                Step::forward(self.color) + Step::LEFT, 
                Step::forward(self.color) + Step::RIGHT
            ],

            Rook => vec![Step::UP, Step::DOWN, Step::LEFT, Step::RIGHT],

            Knight => vec![
                Step::UP   + Step::LEFT  + Step::LEFT,
                Step::UP   + Step::RIGHT + Step::RIGHT,
                Step::DOWN + Step::LEFT  + Step::LEFT,
                Step::DOWN + Step::RIGHT + Step::RIGHT,
                Step::UP   + Step::UP    + Step::LEFT,
                Step::UP   + Step::UP    + Step::RIGHT,
                Step::DOWN + Step::DOWN  + Step::LEFT,
                Step::DOWN + Step::DOWN  + Step::RIGHT,
            ],

            Bishop => vec![
                Step::UP   + Step::LEFT, 
                Step::UP   + Step::RIGHT, 
                Step::DOWN + Step::LEFT, 
                Step::DOWN + Step::RIGHT
            ],

            King | Queen => vec![
                Step::UP, 
                Step::DOWN, 
                Step::LEFT, 
                Step::RIGHT,
                Step::UP   + Step::LEFT, 
                Step::UP   + Step::RIGHT, 
                Step::DOWN + Step::LEFT, 
                Step::DOWN + Step::RIGHT
            ],
        }
    }

    pub fn visible_squares(&self, blockers: Bitboard) -> Bitboard {
        let mut visible = Bitboard::default();

        for step in self.directions() {
            visible |= successors(Some(self.position), |pos| pos.offset(step))
            .skip(1)
            .take(self.range())
            .take_while_inclusive(|&pos| !blockers.contains(pos))
            .collect()
        }

        visible
    }

    pub fn legal_moves(&self, board: &Board) -> Vec<Move> {
        use PieceType::*;

        // - [x] If pawn -> pawn pushes
        // - [x] else -> visible
        // - [x] Include castle
        // - [ ] Filter for checks and pins
        let mut moves: Vec<Move> = match self.piece_type {
            Pawn => pawn_pushes(self.position, self.color, board.all_occupied()),
            _ => self.visible_squares(board.all_occupied())
        }.into_iter()
         .map(|tgt| Move::new(self.position, tgt))
         .collect();

        // Add available castles
        if self.piece_type == King {
            moves.extend(
                board.castling_rights.get_available(self.color)
                    .into_iter()
                    .filter(|ctype| ctype.is_allowed(board))
                    .map(|ctype| ctype.king_move())
            )
        }


        // Checks
        // Pins


        moves
    }
}

fn pawn_pushes(position: Bitboard, side: Color, blockers: Bitboard) -> Bitboard {
    let forward = successors(
        Some(position), 
        |pos| pos.offset(Step::forward(side))
    );

    forward
        .skip(1)
        .take(if position.on_pawn_rank(side) { 2 } else { 1 })
        .take_while_inclusive(|&pos| !blockers.contains(pos))
        .collect()
}
