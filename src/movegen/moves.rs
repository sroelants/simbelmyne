use std::fmt::Display;
use crate::{board::{Piece, Board, PieceType, Color }, bitboard::Step};
use std::iter::successors;
use crate::bitboard::Bitboard;
use itertools::Itertools;
use crate::board::Square;

/// Pack all the metadata related to a Move in a u16
///
/// 6 bits (0 - 63) for the source square
/// 6 bits (0 - 63) for the target square
/// 4 bits (0 - 16) for additional metadata (castling, captures, promotions)
/// When we get to move sorting, to we also want to squeeze in the sorting rank
/// here? 
/// cf. Rustic https://github.com/mvanthoor/rustic/blob/17b15a34b68000dffb681277c3ef6fc98f935a0b/src/movegen/defs.rs
/// cf. Carp https://github.com/dede1751/carp/blob/main/chess/src/moves.rs
#[derive(Default, Debug, Copy, Clone)]
pub struct Move(u16);

impl Move {
    pub const SRC_MASK: u16        = 0b0000_000000_111111;
    pub const TGT_MASK: u16        = 0b0000_111111_000000;
    pub const CASTLE_MASK: u16     = 0b0001_000000_000000;

    pub fn new(src: Square, tgt: Square) -> Move {
        let mut value = 0u16;
        value |= src as u16;
        value |= (tgt as u16) << 6;

        Move(value)
    }

    pub fn src(self) -> Square {
        ((self.0 & Self::SRC_MASK) as usize).into()
    }

    pub fn tgt(self) -> Square {
        (((self.0 & Self::TGT_MASK) >> 6) as usize).into()
    }

    pub fn is_castle(self) -> bool {
        self.0 & Self::CASTLE_MASK != 0
    }

    pub fn set_castle(&mut self) {
        self.0 |= Self::CASTLE_MASK;
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.src().to_alg())?;
        write!(f, "{}", self.tgt().to_alg())?;

        if self.is_castle() {
            write!(f, " (Castle)")?;
        }

        Ok(())
    }
}

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
        let mut targets: Bitboard = match self.piece_type() {
            Pawn => pawn_pushes(self.position, self.color, board.all_occupied()),
            _ => self.visible_squares(board.all_occupied())
        };

        // The king can't move into an attacked square
        if self.piece_type() == PieceType::King {
            targets &= !board.king_danger_squares[self.color().opp() as usize]
        }


        //TODO: Checks
        // Checks should be easy now, right? 
        // 1. [x] King cannot move into a king_danger_square
        // 2. [ ] If king is in check, only legal moves are those that get the king
        //    out of check.
        //  2.1 [ ] Double check -> Only king move can get you out of check
        //  2.2 [ ] Moves that capture the single checker
        //  2.3 [ ] Moves that block the check (similar to pin calculation)
        // Let's start with 1.

        //TODO:  Pins
        

        let mut moves: Vec<Move> = targets.into_iter()
            .map(|tgt| Move::new(self.position.into(), tgt))
            .collect();
 
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn src_works() {
        let src = Square::new(3,4);
        let tgt = Square::new(4,5);

        let mv = Move::new(src,tgt);
        assert_eq!(mv.src(), src.into(), "mv.src() should return the source position, as a bitboard");
    }

    #[test]
    fn tgt_works() {
        let src = Square::new(3,4);
        let tgt = Square::new(4,5);

        let mv = Move::new(src,tgt);
        assert_eq!(mv.tgt(), tgt.into(), "mv.tgt() should return the source target, as a bitboard");
    }

    #[test]
    fn castling_bit() {
        let src = Square::new(3,4);
        let tgt = Square::new(4,5);

        let mut mv = Move::new(src,tgt);
        assert!(!mv.is_castle(), "is_castle returns false for a normal move");

        mv.set_castle();
        assert!(mv.is_castle(), "is_castle returns true after setting the castle bit");
    }
}
