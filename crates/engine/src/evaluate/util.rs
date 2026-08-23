use std::iter::Sum;
use std::ops::Add;
use std::ops::AddAssign;
use std::ops::BitAnd;
use std::ops::BitAndAssign;
use std::ops::BitOr;
use std::ops::BitOrAssign;
use std::ops::Mul;
use std::ops::Neg;
use std::ops::Not;
use std::ops::Sub;
use std::ops::SubAssign;

use arrayvec::ArrayVec;
use bytemuck::Pod;
use bytemuck::Zeroable;
use chess::movegen::legal_moves::MAX_MOVES;
use chess::piece::Piece;
use chess::piece::PieceType;
use chess::square::Square;

pub type Score = i32;

#[derive(Debug, Clone)]
pub struct PieceUpdate {
  pub piece: Piece,
  pub sq: Square,
}

#[derive(Default, Debug, Clone)]
pub struct EvalUpdate {
  added: ArrayVec<PieceUpdate, 2>,
  removed: ArrayVec<PieceUpdate, 2>,
}

impl EvalUpdate {
  pub fn add(&mut self, piece: Piece, sq: Square) {
    self.added.push(PieceUpdate { piece, sq });
  }

  pub fn remove(&mut self, piece: Piece, sq: Square) {
    self.removed.push(PieceUpdate { piece, sq });
  }

  pub fn added(&self) -> &[PieceUpdate] {
    &self.added
  }

  pub fn removed(&self) -> &[PieceUpdate] {
    &self.removed
  }
}

////////////////////////////////////////////////////////////////////////////////
//
// Packed scores
//
// Scores are made sure to fit within an i16, and we pack both of them into an
// 132. This means we can do a poor man's version of SIMD and perform all of
// the operations on midgame/endgame scores in single instructions.
////////////////////////////////////////////////////////////////////////////////

/// A wrapper that stores a midgame and endgame score
///
/// Scores are made sure to fit within an i16, and we pack both of them into an
/// 132. This means we can do a poor man's version of SIMD and perform all of
/// the operations on midgame/endgame scores in single instructions.
#[derive(Default, Copy, Clone, PartialEq, Eq, Pod, Zeroable)]
#[repr(C)]
#[allow(dead_code, unused)]
pub struct S(i32);

// Utility macro that saves us some space when working with many scores at once
// (see [./params.rs]).
#[macro_export]
macro_rules! s {
  ($mg:literal, $eg:literal) => {
    S::new($mg, $eg)
  };
}

impl S {
  /// Create a new packed score.
  pub const fn new(mg: Score, eg: Score) -> Self {
    Self((eg << 16).wrapping_add(mg))
  }

  /// Extract the midgame score from the packed score
  pub fn mg(&self) -> Score {
    self.0 as i16 as Score
  }

  /// Extract the endgame score from the packed score.
  pub fn eg(&self) -> Score {
    ((self.0 + 0x8000) >> 16 as i16) as Score
  }

  /// Interpolate between the midgame and endgame score according to a
  /// given `phase` which is a value between 0 and 24.
  pub fn lerp(&self, phase: u8) -> Score {
    (phase as Score * self.mg() + (24 - phase as Score) * self.eg()) / 24
  }
}

// Utility traits for the packed score, that allow us to use arithmetic
// operations transparently.

impl Add for S {
  type Output = Self;

  fn add(self, rhs: Self) -> Self::Output {
    Self(self.0 + rhs.0)
  }
}

impl AddAssign for S {
  fn add_assign(&mut self, rhs: Self) {
    *self = *self + rhs;
  }
}

impl Sub for S {
  type Output = Self;

  fn sub(self, rhs: Self) -> Self::Output {
    Self(self.0 - rhs.0)
  }
}

impl SubAssign for S {
  fn sub_assign(&mut self, rhs: Self) {
    *self = *self - rhs
  }
}

impl Mul<Score> for S {
  type Output = Self;

  fn mul(self, rhs: Score) -> Self::Output {
    Self(self.0 * rhs)
  }
}

impl Neg for S {
  type Output = Self;

  fn neg(self) -> Self::Output {
    Self::new(-self.mg(), -self.eg())
  }
}

impl Sum for S {
  fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
    iter.fold(Self::default(), Self::add)
  }
}

////////////////////////////////////////////////////////////////////////////////
//
// Score
//
// A `Score` is just a type alias for an i32. This means we can't  really add
// any methods on `Score`s. (because of Rust's orphan rules)
//
// Instead, we define an extension trait that allows us to put some additional
// helper methods on the Score type alias.
//
////////////////////////////////////////////////////////////////////////////////

pub trait ScoreExt {
  const INF: Self;
  const MATE: Self;
  const NO_SCORE: Self;

  /// Return whether the score is a valid value
  fn is_valid(self) -> bool;

  /// Return whether or not a score is a mate score
  fn is_mate(self) -> bool;

  /// Return whether or not a score is a losing score
  fn is_loss(self) -> bool;

  /// Return whether or not a score is a winning score
  fn is_win(self) -> bool;

  /// Return the number of plies until mate.
  fn mate_distance(self) -> i32;

  /// Normalize the score such that mate scores are considered relative to
  /// the _provided ply_.
  fn relative(self, ply: usize) -> Self;

  /// Denormalize a score such that any mate scores are considered relative
  /// to the _root_.
  fn absolute(self, ply: usize) -> Self;
}

impl ScoreExt for Score {
  const MATE: Self = 20_000;
  const INF: Self = 30_000;
  const NO_SCORE: Self = Self::INF + 1;

  fn is_valid(self) -> bool {
    self.abs() <= Self::INF
  }

  fn is_loss(self) -> bool {
    self <= -Self::MATE + MAX_MOVES as i32
  }

  fn is_win(self) -> bool {
    self >= Self::MATE - MAX_MOVES as i32
  }

  fn is_mate(self) -> bool {
    Self::abs(self) >= Self::MATE - MAX_MOVES as i32
  }

  fn mate_distance(self) -> i32 {
    (Self::MATE - self.abs()) as i32
  }

  fn relative(self, ply: usize) -> Self {
    if self.is_mate() {
      self + ply as Self
    } else {
      self
    }
  }

  fn absolute(self, ply: usize) -> Self {
    if self.is_mate() {
      self - ply as Self
    } else {
      self
    }
  }
}

#[derive(Copy, Clone, Default, Eq, PartialEq)]
pub struct PieceSet(u8);

impl PieceSet {
  pub const B: Self = Self(0b000100);

  pub const PN: Self = Self(0b000011);
  pub const PB: Self = Self(0b000101);
  pub const PR: Self = Self(0b001001);
  pub const PQ: Self = Self(0b010001);
  pub const PK: Self = Self(0b100011);

  pub const PRQK: Self = Self(0b111001);

  pub fn new() -> Self {
    Self(0)
  }

  pub fn add(&mut self, ptype: PieceType) {
    self.0 |= 1 << (ptype as usize)
  }

  pub fn has(self, ptype: PieceType) -> bool {
    self.0 & (1 << ptype as usize) > 0
  }

  pub fn empty(self) -> bool {
    self.0 == 0
  }

  pub fn nempty(self) -> bool {
    self.0 != 0
  }
}

impl BitOr for PieceSet {
  type Output = Self;

  fn bitor(self, rhs: Self) -> Self::Output {
    Self(self.0 | rhs.0)
  }
}

impl BitOrAssign for PieceSet {
  fn bitor_assign(&mut self, rhs: Self) {
    self.0 = self.0 | rhs.0;
  }
}

impl BitAnd for PieceSet {
  type Output = Self;

  fn bitand(self, rhs: Self) -> Self::Output {
    Self(self.0 & rhs.0)
  }
}

impl BitAndAssign for PieceSet {
  fn bitand_assign(&mut self, rhs: Self) {
    self.0 = self.0 & rhs.0;
  }
}

impl Not for PieceSet {
  type Output = Self;

  fn not(self) -> Self::Output {
    Self(!self.0)
  }
}
