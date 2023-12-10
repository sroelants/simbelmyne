//! Bitboards represent unordered sets of squares.
//!
//! They make use of the fact that, on most modern hardware, CPUs work on 64-bit
//! numbers, which means we can easily represent bitmasks of the chessboard 
//! (remember, 64 squares) and operate on all 64 squares in a single CPU
//! instruction.

use colored::Colorize;
use std::fmt::Display;
use crate::square::Square;
use std::ops::{
    BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Deref, Not, Shl,
    ShlAssign, Shr, ShrAssign,
};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
/// A bitboard
///
/// Encodes an unordered collection of board squares as a single 64-bit integer.
pub struct Bitboard(pub u64);

impl Bitboard {
    pub const EMPTY: Bitboard = Bitboard(0);

    /// Check whether the bitboard is empty
    pub fn is_empty(self) -> bool {
        self == Bitboard::EMPTY
    }

    /// Count the number of squares in this bitboard
    pub fn count(self) -> u32 {
        self.count_ones()
    }

    /// Return a new bitboard with the squares in the provided bitboard removed.
    pub fn without(self, other: Self) -> Bitboard {
        self & !other
    }

    /// Check whether a given square is contained in the bitboard
    pub fn contains(self, square: Square) -> bool {
        self & square.into() != Bitboard::EMPTY
    }

    /// Get the overlap (Set intersection) of two bitboards
    pub fn overlap(self, other: Self) -> Bitboard {
        self & other
    }

    /// Get the last (trailing) bit of this bitboard.
    /// Panics when passed an empty bitboard!
    pub fn last(self) -> Square {
        let lsb = self.trailing_zeros();
        (1 << lsb).into()
    }

    /// Get the first (leading) bit of this bitboard.
    /// Panics when passed an empty bitboard!
    pub fn first(self) -> Square {
        let msb = self.leading_zeros() + 1;
        (1u64.rotate_right(msb) as usize).into()
    }
}

///////////////////////////////////////////////////////////////////////////////
//
// Utility traits
//
///////////////////////////////////////////////////////////////////////////////

impl From<Square> for Bitboard {
    fn from(value: Square) -> Self {
        Bitboard(1) << value as usize
    }
}

// Implement Deref so we can easily access the inner value
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
                if self.contains(square) {
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

/// Collect an iterator of Bitboards into a single bitboard
impl FromIterator<Bitboard> for Bitboard {
    fn from_iter<T: IntoIterator<Item = Bitboard>>(iter: T) -> Self {
        let mut result = Bitboard::EMPTY;

        for bitboard in iter {
            result |= bitboard;
        }

        result
    }
}

/// Collect an iterator of &Bitboards into a single bitboard
impl<'a> FromIterator<&'a Bitboard> for Bitboard {
    fn from_iter<T: IntoIterator<Item = &'a Bitboard>>(iter: T) -> Bitboard {
        let mut result = Bitboard::EMPTY;

        for bitboard in iter {
            result |= *bitboard;
        }

        result
    }
}

/// Iterate over the squares in a bitboard, in ascending order (A1 -> H8)
impl Iterator for Bitboard {
    type Item = Square;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_empty() {
            return None;
        }
 
        // Grab the first non-zero bit as a bitboard
        let next = self.first();

        // Unset the bit in the original bitboard
        *self ^= next.into();

        Some(next)
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

///////////////////////////////////////////////////////////////////////////////
//
// Tests
//
///////////////////////////////////////////////////////////////////////////////

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
