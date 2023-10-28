use crate::{board::{Piece, Board, PieceType, Color }, movegen::moves::Move, bitboard::Step};
use std::iter::successors;
use crate::bitboard::Bitboard;
use itertools::Itertools;


impl Piece {
    pub fn range(&self) -> usize {
        use PieceType::*;

        match self.piece_type() {
            Pawn | Knight | King => 1,
            _ => 7
        }
    }

    pub fn directions(&self) -> Vec<Step> {
        use PieceType::*;

        match self.piece_type() {
            Pawn => vec![
                Step::forward(self.color) + Step::LEFT, 
                Step::forward(self.color) + Step::RIGHT
            ],

            Rook => vec![Step::UP, Step::DOWN, Step::LEFT, Step::RIGHT],

            Knight => vec![
                Step::new( 1,  2),
                Step::new( 1, -2),
                Step::new(-1,  2),
                Step::new(-1, -2),
                Step::new( 2,  1),
                Step::new( 2, -1),
                Step::new(-2,  1),
                Step::new(-2, -1),
            ],

            Bishop => vec![
                Step::UP_LEFT, Step::UP_RIGHT, Step::DOWN_LEFT, Step::DOWN_RIGHT
            ],

            King | Queen => vec![
                Step::UP, 
                Step::DOWN, 
                Step::LEFT, 
                Step::RIGHT,
                Step::UP_LEFT, 
                Step::UP_RIGHT, 
                Step::DOWN_LEFT, 
                Step::DOWN_RIGHT
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
        let mut moves: Vec<Move> = match self.piece_type() {
            Pawn => pawn_pushes(self.position, self.color, board.all_occupied()),
            _ => self.visible_squares(board.all_occupied())
        }.into_iter()
         .map(|tgt| Move::new(self.position.into(), tgt.into()))
         .collect();

        //TODO: Checks
        // Checks should be easy now, right? 
        // 1. King cannot move into a king_danger_square
        // 2. If king is in check, only legal moves are those that get the king
        // out of check. For this, I might need to calculate pins?
        // Let's start with 1.
        



        //TODO:  Pins


        // Add available castles
        if self.piece_type() == King {
            moves.extend(
                board.castling_rights.get_available(self.color)
                    .into_iter()
                    .filter(|ctype| ctype.is_allowed(board))
                    .map(|ctype| ctype.king_move())
            )
        }

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
