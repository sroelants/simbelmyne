use std::fmt::Display;
use crate::{board::{Piece, PieceType, Color}, bitboard::Step, movegen::attack_boards::{ATTACK_RAYS, KNIGHT_ATTACKS, KING_ATTACKS, W_PAWN_ATTACKS, W_PAWN_PUSHES, W_PAWN_DPUSHES, B_PAWN_ATTACKS, B_PAWN_DPUSHES, B_PAWN_PUSHES}};
use std::iter::successors;
use crate::bitboard::Bitboard;
use itertools::Itertools;
use crate::board::Square;
use crate::movegen::attack_boards::Direction;

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
    pub const DPUSH_MASK: u16      = 0b0010_000000_000000;
    pub const EP_MASK: u16         = 0b0100_000000_000000;


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

    pub fn is_double_push(self) -> bool {
        self.0 & Self::DPUSH_MASK != 0
    }

    pub fn set_double_push(&mut self) {
        self.0 |= Self::DPUSH_MASK;
    }

    pub fn is_en_passant(self) -> bool {
        self.0 & Self::EP_MASK != 0
    }

    pub fn set_en_passant(&mut self) {
        self.0 |= Self::EP_MASK;
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.src().to_alg())?;
        write!(f, "{}", self.tgt().to_alg())?;

        if self.is_castle() {
            write!(f, " (Castle)")?;
        }

        if self.is_double_push() {
            write!(f, " (Double push)")?;
        }

        if self.is_en_passant() {
            write!(f, " (En passant)")?;
        }

        Ok(())
    }
}

impl Piece {
    pub fn range(&self) -> usize {
        use PieceType::*;

        match self.piece_type() {
            Pawn => {
                let hasnt_moved = self.position.on_pawn_rank(self.color());
                if hasnt_moved { 2 } else { 1 }
            }
            Knight | King => 1,
            _ => 7 // The entire board
        }
    }

    pub fn directions(&self) -> &[Step] {
        use PieceType::*;

        match self.piece_type() {
            Pawn => { &Step::PAWN_DIRS[self.color() as usize] }

            Rook => &Step::ORTHO_DIRS,

            Knight => &Step::KNIGHT_DIRS,

            Bishop => &Step::DIAG_DIRS,

            King | Queen => &Step::ALL_DIRS,
        }
    }

    /// All the squares that are _visible_ to the piece.
    /// This means all unoccupied squares in the pieces main directions, up
    /// until (and including), the first blocker piece. 
    ///
    /// This blocker can be either friendly or enemy, so we need to mask out
    /// friendly pieces if we're interested in attacks
    pub fn visible_squares(&self, ours: Bitboard, theirs: Bitboard) -> Bitboard {
        use PieceType::*;
        let sq = Square::from(self.position);
        let blockers = ours | theirs;

        match self.piece_type() {
            Bishop => {
                let mut visible = Bitboard::EMPTY;
                for dir in Direction::BISHOP {
                    visible |= visible_ray(dir, sq, blockers);
                }
                visible
            },

            Rook => {
                let mut visible = Bitboard::EMPTY;
                for dir in Direction::ROOK {
                    visible |= visible_ray(dir, sq, blockers);
                }
                visible
            },
            
            Queen => {
                let mut visible = Bitboard::EMPTY;
                for dir in Direction::ALL {
                    visible |= visible_ray(dir, sq, blockers);
                }
                visible
            },

            Knight => { KNIGHT_ATTACKS[Square::from(self.position) as usize] },

            King => { KING_ATTACKS[Square::from(self.position) as usize] },

            Pawn => {
                let square = Square::from(self.position);
                let mut visible = Bitboard::EMPTY;

                if self.color().is_white() {
                    visible |= theirs & W_PAWN_ATTACKS[square as usize];

                    if self.position.on_pawn_rank(self.color()) {
                        visible |= W_PAWN_DPUSHES[square as usize] & !theirs;
                    } else {
                        visible |= W_PAWN_PUSHES[square as usize] & !theirs;
                    }
                } else {
                    visible |= theirs & B_PAWN_ATTACKS[square as usize];

                    if self.position.on_pawn_rank(self.color()) {
                        visible |= B_PAWN_DPUSHES[square as usize] & !theirs;
                    } else {
                        visible |= B_PAWN_PUSHES[square as usize] & !theirs;
                    }
                }

                visible
            }
        }
    }

    pub fn visible_rays(&self, blockers: Bitboard) -> Vec<Bitboard> {
        self.directions()
            .into_iter()
            .map(|step| successors(self.position.offset(*step), |pos| pos.offset(*step))
                .take(self.range())
                .take_while_inclusive(|&pos| !blockers.contains(pos))
                .collect()
        ).collect()
    }
}

/// Given a direction, return the ray of squares starting at (and excluding) 
/// `square`, up till (and including) the first blocker in the `blockers`
/// bitboard.
fn visible_ray(dir: Direction, square: Square, blockers: Bitboard) -> Bitboard {
    let ray = ATTACK_RAYS[dir as usize][square as usize];
    let mut visible = ray;

    if let Some(blocker) = ray_blocker(dir, square, blockers) {
        visible &= !ATTACK_RAYS[dir as usize][blocker as usize];
    }

    visible
}

fn ray_blocker(dir: Direction, square: Square, blockers: Bitboard) -> Option<Square> {
    let ray = ATTACK_RAYS[dir as usize][square as usize];

    let on_ray_bb = blockers & ray;

    //TODO: Clean this up?
    let blocker = if dir.is_positive() {
        let lsb = on_ray_bb.trailing_zeros() as usize;
        Square::try_from_usize(lsb)
    } else { 
        let lsb = (on_ray_bb.leading_zeros() + 1) as usize;
        64usize.checked_sub(lsb).and_then(Square::try_from_usize)
    };
        
    blocker.map(|sq| Square::from(sq))
}


pub fn visible_squares(square: Square, piece_type: PieceType, color: Color, ours: Bitboard, theirs: Bitboard) -> Bitboard {
    use PieceType::*;
    let blockers = ours | theirs;

    match piece_type {
        Bishop => {
            let mut visible = Bitboard::EMPTY;
            for dir in Direction::BISHOP {
                visible |= visible_ray(dir, square, blockers);
            }
            visible
        },

        Rook => {
            let mut visible = Bitboard::EMPTY;
            for dir in Direction::ROOK {
                visible |= visible_ray(dir, square, blockers);
            }
            visible
        },
        
        Queen => {
            let mut visible = Bitboard::EMPTY;
            for dir in Direction::ALL {
                visible |= visible_ray(dir, square, blockers);
            }
            visible
        },

        Knight => { KNIGHT_ATTACKS[square as usize] },

        King => { KING_ATTACKS[square as usize] },

        Pawn => {
            let mut visible = Bitboard::EMPTY;
            let sq_bb = Bitboard::from(square); 

            if color.is_white() {
                visible |= theirs & W_PAWN_ATTACKS[square as usize];

                if sq_bb.on_pawn_rank(color) {
                    visible |= W_PAWN_DPUSHES[square as usize] & !theirs;
                } else {
                    visible |= W_PAWN_PUSHES[square as usize] & !theirs;
                }
            } else {
                visible |= theirs & B_PAWN_ATTACKS[square as usize];

                if sq_bb.on_pawn_rank(color) {
                    visible |= B_PAWN_DPUSHES[square as usize] & !theirs;
                } else {
                    visible |= B_PAWN_PUSHES[square as usize] & !theirs;
                }
            }

            visible
        }

    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ray_blocker() {
        let dir = Direction::Up;
        let square = Square::new(3, 3); // d4
    
        let blocker = Square::new(6, 3); // d7
        let blockers = Bitboard(0xaa98605591844602); // A bunch of crap

        let result = ray_blocker(dir, square, blockers);

        assert!(result.is_some());
        assert_eq!(result.unwrap(), blocker);
    }

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
