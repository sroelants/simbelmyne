use std::iter::Sum;
use std::ops::Add;
use std::ops::AddAssign;
use std::ops::Mul;
use std::ops::Neg;
use std::ops::Sub;
use std::ops::SubAssign;

use bytemuck::Pod;
use bytemuck::Zeroable;
use chess::movegen::legal_moves::MAX_MOVES;

pub type Score = i32;

////////////////////////////////////////////////////////////////////////////////
//
// Packed scores
//
/// Scores are made sure to fit within an i16, and we pack both of them into an
/// 132. This means we can do a poor man's version of SIMD and perform all of 
/// the operations on midgame/endgame scores in single instructions.
///
////////////////////////////////////////////////////////////////////////////////

/// A wrapper that stores a midgame and endgame score
///
/// Scores are made sure to fit within an i16, and we pack both of them into an
/// 132. This means we can do a poor man's version of SIMD and perform all of 
/// the operations on midgame/endgame scores in single instructions.
#[derive(Default, Copy, Clone, PartialEq, Eq, Pod, Zeroable)]
#[repr(C)]
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
    const MINUS_INF: Self;
    const PLUS_INF: Self;
    const MATE: Self;
    const NO_SCORE: Self;
    const LOWEST_MATE: Self;

    /// Return whether or not a score is a mate score
    fn is_mate(self) -> bool;

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
    const MINUS_INF: Self = Self::MIN + 1;
    const PLUS_INF: Self = Self::MAX;
    const MATE: Self = 20_000;
    const NO_SCORE: Self = Self::MINUS_INF;
    const LOWEST_MATE: Self = Self::MATE - MAX_MOVES as Self;

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
