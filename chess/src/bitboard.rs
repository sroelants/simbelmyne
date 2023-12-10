use anyhow::anyhow;
use colored::Colorize;
use std::ops::{
    BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Deref, Not, Shl,
    ShlAssign, Shr, ShrAssign,
};
use std::fmt::Display;

use crate::square::Square;
use crate::util::parse;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct Bitboard(pub u64);

impl Bitboard {
    pub const EMPTY: Bitboard = Bitboard(0);

    /// Add the squares of the provided bitboard into this one.
    pub fn add_in_place(&mut self, positions: Self) {
        *self |= positions;
    }

    /// Return a new bitboard with the squares in the provided bitboard removed.
    pub fn remove(self, positions: Self) -> Bitboard {
        Bitboard(self.0 & !positions.0)
    }

    /// Check whether the provided bitboard is entirely contained within this
    /// bitboard.
    pub fn contains(self, positions: Self) -> bool {
        self & positions == positions
    }

    /// Check whether this bitboard has any squares in common with the provided
    /// bitboard.
    pub fn has_overlap(self, bb: Self) -> bool {
        self & bb != Bitboard(0)
    }

    /// Check whether the bitboard is empty
    pub fn is_empty(self) -> bool {
        self == Bitboard(0)
    }

    /// Count the number of squares in this bitboard
    pub fn count(self) -> u32 {
        self.count_ones()
    }

    /// Get the last (trailing) bit of this bitboard.
    /// Panics when passed an empty bitboard!
    pub fn last(self) -> Self {
        let lsb = self.trailing_zeros();
        Self(1 << lsb)
    }

    /// Get the first (leading) bit of this bitboard.
    /// Panics when passed an empty bitboard!
    pub fn first(self) -> Self {
        let msb = self.leading_zeros() + 1;
        Self(1u64.rotate_right(msb))
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

impl Deref for Bitboard {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for Bitboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for rank in Square::RANKS {
            for square in rank {
                if self.contains(Bitboard::from(square)) {
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
        let mut result = Bitboard::EMPTY;

        for bitboard in iter {
            result |= bitboard;
        }

        result
    }
}

impl<'a> FromIterator<&'a Bitboard> for Bitboard {
    fn from_iter<T: IntoIterator<Item = &'a Bitboard>>(iter: T) -> Bitboard {
        let mut result = Bitboard::EMPTY;

        for bitboard in iter {
            result |= *bitboard;
        }

        result
    }
}

impl TryFrom<&str> for Bitboard {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let (_, square) = parse::algebraic_square(value).map_err(|_| anyhow!("Failed to parse"))?;
        Ok(Bitboard::from(square))
    }
}

impl Iterator for Bitboard {
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
    use Square::*;

    #[test]
    fn position_new_00() {
        assert_eq!(Bitboard::from(A1).0, 1);
    }

    #[test]
    fn position_new_10() {
        assert_eq!(Bitboard::from(A2).0.trailing_zeros(), 8);
    }

    #[test]
    fn position_new_05() {
        assert_eq!(Bitboard::from(F1).0.trailing_zeros(), 5);
    }

    #[test]
    fn position_new_25() {
        assert_eq!(Bitboard::from(F3).0.trailing_zeros(), 21);
    }
}
