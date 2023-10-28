use std::ops::{Deref, BitAnd, BitOr, BitXor, Not, BitAndAssign, BitOrAssign, BitXorAssign, Shl, ShlAssign, Shr, ShrAssign, Add};
use std::{ops::Div, fmt::Display};
use anyhow::anyhow;
use colored::*;

use crate::board::{Color, Square};
use crate::parse;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct Bitboard(pub u64);

impl Bitboard {
    pub const PAWN_RANKS: [Bitboard; 2] = [ 
        Bitboard(0x000000F0), 
        Bitboard(0x0F000000) 
    ];

    pub fn on_pawn_rank(&self, color: Color) -> bool {
        Bitboard::PAWN_RANKS[color as usize].contains(*self)
    }

    pub fn new(rank: usize, file: usize) -> Self {
        (Bitboard(1) << 8*rank) << file
    }

    pub fn rank(&self) -> usize {
        self.trailing_zeros().div(8) as usize
    }

    pub fn file(&self) -> usize {
        (self.trailing_zeros() % 8) as usize
    }

    pub fn add_in_place(&mut self, positions: Self) {
        *self |= positions;
    }

    pub fn remove(self, positions: Self) -> Bitboard{
        Bitboard(self.0 & !positions.0)
    }

    pub fn contains(self, positions: Self) -> bool {
        self & positions == positions
    }

    pub fn has_overlap(self, bb: Self) -> bool {
        self & bb != Bitboard(0)
    }

    pub fn is_empty(self) -> bool {
        self == Bitboard(0)
    }
}

impl From<Bitboard> for Square {
    fn from(value: Bitboard) -> Self {
        Square::ALL[value.trailing_zeros() as usize]
    }
}

impl From<Square> for Bitboard {
    fn from(value: Square) -> Self {
        Bitboard(1) << value as usize
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Step {
    delta_rank: isize,
    delta_file: isize,
}

impl Step {
    pub const UP: Step         = Step { delta_rank:  1, delta_file:  0 };
    pub const DOWN: Step       = Step { delta_rank: -1, delta_file:  0 };
    pub const LEFT: Step       = Step { delta_rank:  0, delta_file: -1 };
    pub const RIGHT: Step      = Step { delta_rank:  0, delta_file:  1 };
    pub const UP_LEFT: Step    = Step { delta_rank:  1, delta_file: -1 };
    pub const UP_RIGHT: Step   = Step { delta_rank:  1, delta_file:  1 };
    pub const DOWN_LEFT: Step  = Step { delta_rank: -1, delta_file: -1 };
    pub const DOWN_RIGHT: Step = Step { delta_rank: -1, delta_file:  1 };

    pub fn new(delta_rank: isize, delta_file: isize) -> Step {
        Step { delta_rank, delta_file } 
    }

    pub fn forward(side: Color) -> Self {
        if side == Color::White { Self::UP } else { Self::DOWN }
    }
}

impl Add for Step {
    type Output = Step;

    fn add(self, rhs: Self) -> Self::Output {
        Step { 
            delta_rank: self.delta_rank + rhs.delta_rank,
            delta_file: self.delta_file + rhs.delta_file,
        }
    }
}

impl Bitboard { 
    pub fn offset(&self, step: Step) -> Option<Bitboard> {
        let Step { delta_rank, delta_file } = step;
        let rank = self.rank();
        let file = self.file();
        let new_rank = rank.checked_add_signed(delta_rank)?;
        let new_file = file.checked_add_signed(delta_file)?;

        if new_rank < 8 && new_file < 8 {
            Some(Bitboard::new(new_rank, new_file))
        } else {
            None
        }
    }
}


impl Deref for Bitboard {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for Bitboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for rank in (0..8).rev() {
            for file in 0..8 {
                if self.contains(Bitboard::new(rank, file)) {
                    write!(f, "x ")?;
                } else {
                    write!(f, "{}", ". ".bright_black())?;
                }
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

impl FromIterator<Bitboard> for Bitboard {
    fn from_iter<T: IntoIterator<Item = Bitboard>>(iter: T) -> Self {
        let mut result = Bitboard::default();

        for positions in iter {
            result.add_in_place(positions);
        }

        result
    }
}

impl From<Vec<Bitboard>> for Bitboard {
    fn from(boards: Vec<Bitboard>) -> Bitboard {
        let mut result = Bitboard::default();

        for board in boards {
            result.add_in_place(board);
        }

        result
    }
}

impl TryFrom<&str> for Bitboard {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let (_, square) = parse::algebraic_square(value)
            .map_err(|_| anyhow!("Failed to parse"))?;
        Ok(Bitboard::from(square))
    }
}

impl Iterator  for Bitboard {
    type Item = Square;

    fn next(&mut self) -> Option<Self::Item> {
        // Get the next lsb
        let pos = Bitboard(1u64.checked_shl(self.0.trailing_zeros())?);

        // set the current pos to zero
        *self ^= pos;

        Some(pos.into())
    }
}

impl BitAnd<Bitboard> for Bitboard {
    type Output = Bitboard;

    fn bitand(self, rhs: Bitboard) -> Self::Output {
        Bitboard(self.0 & rhs.0)
    }
}

impl BitAndAssign for Bitboard {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl BitOr<Bitboard> for Bitboard {
    type Output = Bitboard;

    fn bitor(self, rhs: Bitboard) -> Self::Output {
        Bitboard(self.0 | rhs.0)
    }
}

impl BitOrAssign for Bitboard {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitXor<Bitboard> for Bitboard {
    type Output = Bitboard;

    fn bitxor(self, rhs: Bitboard) -> Self::Output {
        Bitboard(self.0 ^ rhs.0)
    }
}

impl BitXorAssign for Bitboard {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
    }
}

impl Not for Bitboard {
    type Output = Bitboard;

    fn not(self) -> Self::Output {
        Bitboard(!self.0)
    }
}

impl Shl<usize> for Bitboard {
    type Output = Bitboard;

    fn shl(self, rhs: usize) -> Self::Output {
        Bitboard(self.0 << rhs)
    }
}

impl ShlAssign<usize> for Bitboard {
    fn shl_assign(&mut self, rhs: usize) {
        self.0 <<= rhs;
    }
}

impl Shr<usize> for Bitboard {
    type Output = Bitboard;

    fn shr(self, rhs: usize) -> Self::Output {
        Bitboard(self.0 >> rhs)
    }
}

impl ShrAssign<usize> for Bitboard {
    fn shr_assign(&mut self, rhs: usize) {
        self.0 >>= rhs;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn position_new_00() {
        assert_eq!(Bitboard::new(0,0).0, 1);
    }

    #[test]
    fn position_new_10() {
        assert_eq!(Bitboard::new(1,0).0.trailing_zeros(), 8 );
    }

    #[test]
    fn position_new_05() {
        assert_eq!(Bitboard::new(0,5).0.trailing_zeros(), 5 );
    }

    #[test]
    fn position_new_25() {
        assert_eq!(Bitboard::new(2,5).0.trailing_zeros(), 21 );
    }

    #[test]
    fn position_rank() {
        assert_eq!(Bitboard::new(2,5).rank(), 2 );
        assert_eq!(Bitboard::new(7,7).rank(), 7 );
        assert_eq!(Bitboard::new(4,2).rank(), 4 );
    }

    #[test]
    fn position_file() {
        assert_eq!(Bitboard::new(2,5).file(), 5 );
        assert_eq!(Bitboard::new(7,7).file(), 7 );
        assert_eq!(Bitboard::new(4,2).file(), 2 );
    }

    #[test]
    fn test_offset() {
        assert_eq!(
            Bitboard::new(1,1).offset(Step{ delta_rank: 1, delta_file: 1 }),
            Some(Bitboard::new(2,2))
        );

        assert_eq!(
            Bitboard::new(1,1).offset(Step{ delta_rank: -1, delta_file: 1 }),
            Some(Bitboard::new(0,2))
        );

        assert_eq!(
            Bitboard::new(1,1).offset(Step{ delta_rank: -2, delta_file: 1 }),
            None
        );

        assert_eq!(
            Bitboard::new(1,1).offset(Step{ delta_rank: 1, delta_file: 6 }),
            Some(Bitboard::new(2,7))
        );

        assert_eq!(
            Bitboard::new(1,1).offset(Step{ delta_rank: 1, delta_file: 7 }),
            None
        );
    }
}
