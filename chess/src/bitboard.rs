//! Bitboards represent unordered sets of squares.
//!
//! They make use of the fact that, on most modern hardware, CPUs work on 64-bit
//! numbers, which means we can easily represent bitmasks of the chessboard 
//! (remember, 64 squares) and operate on all 64 squares in a single CPU
//! instruction.

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
    pub const EMPTY: Self = Self(0);

    /// Check whether the bitboard is empty
    pub fn is_empty(self) -> bool {
        self == Self::EMPTY
    }

    /// Count the number of squares in this bitboard
    pub fn count(self) -> u32 {
        self.count_ones()
    }

    /// Return a new bitboard with the squares in the provided bitboard removed.
    pub fn without(self, other: Self) -> Self {
        self & !other
    }

    /// Check whether a given square is contained in the bitboard
    pub fn contains(self, square: Square) -> bool {
        self & square.into() != Self::EMPTY
    }

    /// Get the overlap (Set intersection) of two bitboards
    pub fn overlap(self, other: Self) -> Self {
        self & other
    }

    /// Get the square corresponding to the first (leading) bit of this 
    /// bitboard.
    /// Panics when passed an empty bitboard!
    pub fn first(self) -> Square {
        let msb = 63 - self.leading_zeros(); // 0..=63
        (msb as usize).into()
    }

    /// Get the square corresponding to the last (trailing) bit of this 
    /// bitboard.
    /// Panics when passed an empty bitboard!
    pub fn last(self) -> Square {
        let lsb = self.trailing_zeros(); // 0..=63
        (lsb as usize).into()
    }
}

///////////////////////////////////////////////////////////////////////////////
//
// Utility traits
//
///////////////////////////////////////////////////////////////////////////////

impl From<Square> for Bitboard {
    fn from(value: Square) -> Self {
        Self(1) << value as usize
    }
}

impl From<Option<Square>> for Bitboard {
    fn from(value: Option<Square>) -> Self {
        match value {
            Some(sq) => Bitboard::from(sq),
            None => Bitboard::EMPTY,
        }
    }
}

impl FromIterator<Square> for Bitboard {
    fn from_iter<T: IntoIterator<Item = Square>>(iter: T) -> Self {
        iter.into_iter()
            .map(|sq| Bitboard::from(sq))
            .collect()
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
                    write!(f, "{}", ". ")?;
                }
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

/// Collect an iterator of Bitboards into a single bitboard
impl FromIterator<Bitboard> for Bitboard {
    fn from_iter<T: IntoIterator<Item = Self>>(iter: T) -> Self {
        let mut result = Self::EMPTY;

        for bitboard in iter {
            result |= bitboard;
        }

        result
    }
}

/// Collect an iterator of &Bitboards into a single bitboard
impl<'a> FromIterator<&'a Bitboard> for Bitboard {
    fn from_iter<T: IntoIterator<Item = &'a Self>>(iter: T) -> Self {
        let mut result = Self::EMPTY;

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
        let next_sq = Bitboard::last(*self);

        // Unset the bit in the original bitboard
        *self ^= Bitboard::from(next_sq);

        Some(next_sq)
    }
}

impl BitAnd<Bitboard> for Bitboard {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitAndAssign for Bitboard {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl BitOr<Bitboard> for Bitboard {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for Bitboard {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitXor<Bitboard> for Bitboard {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl BitXorAssign for Bitboard {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
    }
}

impl Not for Bitboard {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

impl Shl<usize> for Bitboard {
    type Output = Self;

    fn shl(self, rhs: usize) -> Self::Output {
        Self(self.0 << rhs)
    }
}

impl ShlAssign<usize> for Bitboard {
    fn shl_assign(&mut self, rhs: usize) {
        self.0 <<= rhs;
    }
}

impl Shr<usize> for Bitboard {
    type Output = Self;

    fn shr(self, rhs: usize) -> Self::Output {
        Self(self.0 >> rhs)
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

    #[test]
    fn test_first() {
        let bb: Bitboard = Bitboard::from(D4) | Bitboard::from(F7);
        assert_eq!(bb.first(), F7);
    }

    #[test]
    fn test_last() {
        let bb: Bitboard = Bitboard::from(D4) | Bitboard::from(F7);
        assert_eq!(bb.last(), D4);
    }

}
