//! Bitboards represent unordered sets of squares.
//!
//! They make use of the fact that, on most modern hardware, CPUs work on 64-bit
//! numbers, which means we can easily represent bitmasks of the chessboard
//! (remember, 64 squares) and operate on all 64 squares in a single CPU
//! instruction.

use crate::constants::FILES;
use crate::square::Square;
use std::fmt::Display;
use std::ops::BitAnd;
use std::ops::BitAndAssign;
use std::ops::BitOr;
use std::ops::BitOrAssign;
use std::ops::BitXor;
use std::ops::BitXorAssign;
use std::ops::Deref;
use std::ops::Not;
use std::ops::Shl;
use std::ops::ShlAssign;
use std::ops::Shr;
use std::ops::ShrAssign;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
/// A bitboard
///
/// Encodes an unordered collection of board squares as a single 64-bit integer.
pub struct Bitboard(pub u64);

impl Bitboard {
  pub const EMPTY: Self = Self(0);
  pub const ALL: Self = Self(!0);

  /// Check whether the bitboard is empty
  #[inline(always)]
  pub fn is_empty(self) -> bool {
    self == Self::EMPTY
  }

  /// Count the number of squares in this bitboard
  #[inline(always)]
  pub fn count(self) -> u32 {
    self.count_ones()
  }

  /// Return a new bitboard with the squares in the provided bitboard removed.
  #[inline(always)]
  pub fn without(self, other: Self) -> Self {
    self & !other
  }

  /// Check whether a given square is contained in the bitboard
  #[inline(always)]
  pub fn contains(self, square: Square) -> bool {
    self & square.into() != Self::EMPTY
  }

  /// Get the overlap (Set intersection) of two bitboards
  #[inline(always)]
  pub fn overlap(self, other: Self) -> Self {
    self & other
  }

  /// Get the square corresponding to the first (leading) bit of this
  /// bitboard.
  /// Panics when passed an empty bitboard!
  #[inline(always)]
  pub fn first(self) -> Square {
    let msb = 63 - self.leading_zeros(); // 0..=63
    Square::new(msb as u8).unwrap()
  }

  /// Get the square corresponding to the last (trailing) bit of this
  /// bitboard.
  /// Panics when passed an empty bitboard!
  #[inline(always)]
  pub fn last(self) -> Square {
    let lsb = self.trailing_zeros(); // 0..=63
    Square::new(lsb as u8).unwrap()
  }

  /// Shift a bitboard left by one file
  #[inline(always)]
  pub fn left(self) -> Self {
    self >> 1 & !FILES[7]
  }

  /// Shift a bitboard right by one file
  #[inline(always)]
  pub fn right(self) -> Self {
    self << 1 & !FILES[0]
  }

  /// Shift a bitboard up by one rank
  #[inline(always)]
  pub fn up(self) -> Self {
    self << 8
  }

  /// Shift a bitboard down by one rank
  #[inline(always)]
  pub fn down(self) -> Self {
    self >> 8
  }

  /// Shift a bitboard up by `n` ranks
  #[inline(always)]
  pub fn up_by(self, n: usize) -> Self {
    self << 8 * n
  }

  /// Shift a bitboard down by `n` ranks
  #[inline(always)]
  pub fn down_by(self, n: usize) -> Self {
    self >> 8 * n
  }

  /// Shift a bitboard one rank forward, relative to the requested color
  #[inline(always)]
  pub fn forward<const WHITE: bool>(self) -> Self {
    if WHITE {
      self.up()
    } else {
      self.down()
    }
  }

  /// Shift a bitboard one rank backward, relative to the requested color
  #[inline(always)]
  pub fn backward<const WHITE: bool>(self) -> Self {
    if WHITE {
      self.down()
    } else {
      self.up()
    }
  }

  /// Shift a bitboard `n` ranks forward, relative to the requested color
  #[inline(always)]
  pub fn forward_by<const WHITE: bool>(self, n: usize) -> Self {
    if WHITE {
      self.up_by(n)
    } else {
      self.down_by(n)
    }
  }

  /// Shift a bitboard `n` ranks backward, relative to the requested color
  #[inline(always)]
  pub fn backward_by<const WHITE: bool>(self, n: usize) -> Self {
    if WHITE {
      self.down_by(n)
    } else {
      self.up_by(n)
    }
  }

  #[inline(always)]
  pub fn forward_left<const WHITE: bool>(self) -> Self {
    if WHITE {
      self << 7 & !FILES[7]
    } else {
      self >> 9 & !FILES[7]
    }
  }

  #[inline(always)]
  pub fn forward_right<const WHITE: bool>(self) -> Self {
    if WHITE {
      self << 9 & !FILES[0]
    } else {
      self >> 7 & !FILES[0]
    }
  }

  #[inline(always)]
  pub fn backward_left<const WHITE: bool>(self) -> Self {
    if WHITE {
      self >> 9 & !FILES[7]
    } else {
      self << 7 & !FILES[7]
    }
  }

  #[inline(always)]
  pub fn backward_right<const WHITE: bool>(self) -> Self {
    if WHITE {
      self >> 7 & !FILES[0]
    } else {
      self << 9 & !FILES[0]
    }
  }
}

///////////////////////////////////////////////////////////////////////////////
//
// Utility traits
//
///////////////////////////////////////////////////////////////////////////////

impl From<Square> for Bitboard {
  #[inline(always)]
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
    iter.into_iter().map(|sq| Bitboard::from(sq)).collect()
  }
}

// Implement Deref so we can easily access the inner value
impl Deref for Bitboard {
  type Target = u64;

  #[inline(always)]
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

  // Implementation yoinked from viri, because it was faster than our
  // naive implementation
  //
  // faster if we have bmi (maybe)
  fn next(&mut self) -> Option<Self::Item> {
    if self.is_empty() {
      None
    } else {
      #[allow(clippy::cast_possible_truncation)]
      let lsb: u8 = self.0.trailing_zeros() as u8;

      self.0 &= self.0 - 1;

      // SAFETY:
      // We made sure the bitboard is not empty, so `u64::trailing_zeros`
      // can only return a number between 0..=63, which are valid square
      // indices.
      Some(unsafe { Square::new_unchecked(lsb) })
    }
  }
}

impl BitAnd<Bitboard> for Bitboard {
  type Output = Self;

  #[inline(always)]
  fn bitand(self, rhs: Self) -> Self::Output {
    Self(self.0 & rhs.0)
  }
}

impl BitAndAssign for Bitboard {
  #[inline(always)]
  fn bitand_assign(&mut self, rhs: Self) {
    self.0 &= rhs.0;
  }
}

impl BitOr<Bitboard> for Bitboard {
  type Output = Self;

  #[inline(always)]
  fn bitor(self, rhs: Self) -> Self::Output {
    Self(self.0 | rhs.0)
  }
}

impl BitOrAssign for Bitboard {
  #[inline(always)]
  fn bitor_assign(&mut self, rhs: Self) {
    self.0 |= rhs.0;
  }
}

impl BitXor<Bitboard> for Bitboard {
  type Output = Self;

  #[inline(always)]
  fn bitxor(self, rhs: Self) -> Self::Output {
    Self(self.0 ^ rhs.0)
  }
}

impl BitXorAssign for Bitboard {
  #[inline(always)]
  fn bitxor_assign(&mut self, rhs: Self) {
    self.0 ^= rhs.0;
  }
}

impl Not for Bitboard {
  type Output = Self;

  #[inline(always)]
  fn not(self) -> Self::Output {
    Self(!self.0)
  }
}

impl Shl<usize> for Bitboard {
  type Output = Self;

  #[inline(always)]
  fn shl(self, rhs: usize) -> Self::Output {
    Self(self.0 << rhs)
  }
}

impl ShlAssign<usize> for Bitboard {
  #[inline(always)]
  fn shl_assign(&mut self, rhs: usize) {
    self.0 <<= rhs;
  }
}

impl Shr<usize> for Bitboard {
  type Output = Self;

  #[inline(always)]
  fn shr(self, rhs: usize) -> Self::Output {
    Self(self.0 >> rhs)
  }
}

impl ShrAssign<usize> for Bitboard {
  #[inline(always)]
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
