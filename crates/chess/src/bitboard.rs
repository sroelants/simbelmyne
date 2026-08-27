//! Bitboards represent unordered sets of squares.
//!
//! They make use of the fact that, on most modern hardware, CPUs work on 64-bit
//! numbers, which means we can easily represent bitmasks of the chessboard
//! (remember, 64 squares) and operate on all 64 squares in a single CPU
//! instruction.

use crate::constants::FILES;
use crate::piece::Color;
use crate::square::Square;
use crate::types::Direction;
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
  pub const fn is_empty(self) -> bool {
    self.0 == 0
  }

  /// Count the number of squares in this bitboard
  #[inline(always)]
  pub const fn count(self) -> u32 {
    self.count_ones()
  }

  /// Check whether a given square is contained in the bitboard
  #[inline(always)]
  pub const fn contains(self, square: Square) -> bool {
    self.0 & 1 << square as usize != 0
  }

  /// Get the square corresponding to the first (leading) bit of this
  /// bitboard.
  /// Panics when passed an empty bitboard!
  #[inline(always)]
  pub const fn first(self) -> Square {
    let msb = 63 - self.leading_zeros(); // 0..=63
    Square::new(msb as u8).unwrap()
  }

  #[inline(always)]
  pub const fn first_checked(self) -> Option<Square> {
    if self.0 == 0 {
      return None;
    }

    Some(self.first())
  }

  #[inline(always)]
  pub const fn shift(self, dir: Direction) -> Bitboard {
    match dir {
      Direction::Up => self << 8,
      Direction::Down => self >> 8,
      Direction::Right => (self & !FILES[7]) << 1,
      Direction::Left => (self & !FILES[0]) >> 1,
      Direction::UpRight => (self & !FILES[7]) << 9,
      Direction::UpLeft => (self & !FILES[0]) << 7,
      Direction::DownRight => (self & !FILES[7]) >> 7,
      Direction::DownLeft => (self & !FILES[0]) >> 9,
    }
  }

  /// Shift a bitboard left by one file
  #[inline(always)]
  pub const fn left(self) -> Self {
    self >> 1 & !FILES[7]
  }

  /// Shift a bitboard right by one file
  #[inline(always)]
  pub const fn right(self) -> Self {
    self << 1 & !FILES[0]
  }

  /// Shift a bitboard up by one rank
  #[inline(always)]
  pub const fn up(self) -> Self {
    self << 8
  }

  /// Shift a bitboard down by one rank
  #[inline(always)]
  pub const fn down(self) -> Self {
    self >> 8
  }

  /// Shift a bitboard up by `n` ranks
  #[inline(always)]
  pub const fn up_by(self, n: usize) -> Self {
    self << 8 * n
  }

  /// Shift a bitboard down by `n` ranks
  #[inline(always)]
  pub const fn down_by(self, n: usize) -> Self {
    self >> 8 * n
  }

  /// Shift a bitboard one rank forward, relative to the requested color
  #[inline(always)]
  pub const fn forward(self, us: Color) -> Self {
    if us.is_white() {
      self.up()
    } else {
      self.down()
    }
  }

  /// Shift a bitboard one rank backward, relative to the requested color
  #[inline(always)]
  pub const fn backward(self, us: Color) -> Self {
    if us.is_white() {
      self.down()
    } else {
      self.up()
    }
  }

  /// Shift a bitboard `n` ranks forward, relative to the requested color
  #[inline(always)]
  pub const fn forward_by(self, n: usize, us: Color) -> Self {
    if us.is_white() {
      self.up_by(n)
    } else {
      self.down_by(n)
    }
  }

  /// Shift a bitboard `n` ranks backward, relative to the requested color
  #[inline(always)]
  pub const fn backward_by(self, n: usize, us: Color) -> Self {
    if us.is_white() {
      self.down_by(n)
    } else {
      self.up_by(n)
    }
  }

  #[inline(always)]
  pub const fn forward_left(self, us: Color) -> Self {
    if us.is_white() {
      self << 7 & !FILES[7]
    } else {
      self >> 9 & !FILES[7]
    }
  }

  #[inline(always)]
  pub const fn forward_right(self, us: Color) -> Self {
    if us.is_white() {
      self << 9 & !FILES[0]
    } else {
      self >> 7 & !FILES[0]
    }
  }

  #[inline(always)]
  pub const fn backward_left(self, us: Color) -> Self {
    if us.is_white() {
      self >> 9 & !FILES[7]
    } else {
      self << 7 & !FILES[7]
    }
  }

  #[inline(always)]
  pub const fn backward_right(self, us: Color) -> Self {
    if us.is_white() {
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

const impl From<Square> for Bitboard {
  #[inline(always)]
  fn from(value: Square) -> Self {
    Self(1) << value as usize
  }
}

const impl From<Option<Square>> for Bitboard {
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
const impl Deref for Bitboard {
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

const impl BitAnd<Bitboard> for Bitboard {
  type Output = Self;

  #[inline(always)]
  fn bitand(self, rhs: Self) -> Self::Output {
    Self(self.0 & rhs.0)
  }
}

const impl BitAndAssign for Bitboard {
  #[inline(always)]
  fn bitand_assign(&mut self, rhs: Self) {
    self.0 &= rhs.0;
  }
}

const impl BitOr<Bitboard> for Bitboard {
  type Output = Self;

  #[inline(always)]
  fn bitor(self, rhs: Self) -> Self::Output {
    Self(self.0 | rhs.0)
  }
}

const impl BitOrAssign for Bitboard {
  #[inline(always)]
  fn bitor_assign(&mut self, rhs: Self) {
    self.0 |= rhs.0;
  }
}

const impl BitXor<Bitboard> for Bitboard {
  type Output = Self;

  #[inline(always)]
  fn bitxor(self, rhs: Self) -> Self::Output {
    Self(self.0 ^ rhs.0)
  }
}

const impl BitXorAssign for Bitboard {
  #[inline(always)]
  fn bitxor_assign(&mut self, rhs: Self) {
    self.0 ^= rhs.0;
  }
}

const impl Not for Bitboard {
  type Output = Self;

  #[inline(always)]
  fn not(self) -> Self::Output {
    Self(!self.0)
  }
}

const impl Shl<usize> for Bitboard {
  type Output = Self;

  #[inline(always)]
  fn shl(self, rhs: usize) -> Self::Output {
    Self(self.0 << rhs)
  }
}

const impl ShlAssign<usize> for Bitboard {
  #[inline(always)]
  fn shl_assign(&mut self, rhs: usize) {
    self.0 <<= rhs;
  }
}

const impl Shr<usize> for Bitboard {
  type Output = Self;

  #[inline(always)]
  fn shr(self, rhs: usize) -> Self::Output {
    Self(self.0 >> rhs)
  }
}

const impl ShrAssign<usize> for Bitboard {
  #[inline(always)]
  fn shr_assign(&mut self, rhs: usize) {
    self.0 >>= rhs;
  }
}
