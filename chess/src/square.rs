//! Squares are one of the two data types we use in Simbelmyne 
//! to denote positions, the other being Bitboards.
//!
//! As one might expect, a Square always denotes a single square, where a
//! Bitboard is used to represent an _unordered set_ of positions at the once .

use anyhow::anyhow;
use std::{fmt::Display, str::FromStr};
use crate::piece::Color;
use Square::*;

#[rustfmt::skip]
#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
/// A board square
///
/// Often used to cast to a usize and index into arrays of different sorts
pub enum Square {
    A1, B1, C1, D1, E1, F1, G1, H1,
    A2, B2, C2, D2, E2, F2, G2, H2,
    A3, B3, C3, D3, E3, F3, G3, H3, 
    A4, B4, C4, D4, E4, F4, G4, H4,
    A5, B5, C5, D5, E5, F5, G5, H5,
    A6, B6, C6, D6, E6, F6, G6, H6,
    A7, B7, C7, D7, E7, F7, G7, H7,
    A8, B8, C8, D8, E8, F8, G8, H8,
}


impl Square {
    pub const COUNT: usize = 64;

    #[rustfmt::skip]
    /// A set of all the squares, handy for converting a usize into a square.
    pub const ALL: [Self; Self::COUNT] = [
        A1, B1, C1, D1, E1, F1, G1, H1, 
        A2, B2, C2, D2, E2, F2, G2, H2, 
        A3, B3, C3, D3, E3, F3, G3, H3, 
        A4, B4, C4, D4, E4, F4, G4, H4, 
        A5, B5, C5, D5, E5, F5, G5, H5, 
        A6, B6, C6, D6, E6, F6, G6, H6, 
        A7, B7, C7, D7, E7, F7, G7, H7, 
        A8, B8, C8, D8, E8, F8, G8, H8,
    ];

    #[rustfmt::skip]
    /// Collection of ranks (in reversed order), handy for iterating over a 
    /// board in a double loop (ranks and files).
    pub const RANKS: [[Self; 8]; 8] = [
        [A8, B8, C8, D8, E8, F8, G8, H8],
        [A7, B7, C7, D7, E7, F7, G7, H7], 
        [A6, B6, C6, D6, E6, F6, G6, H6], 
        [A5, B5, C5, D5, E5, F5, G5, H5], 
        [A4, B4, C4, D4, E4, F4, G4, H4], 
        [A3, B3, C3, D3, E3, F3, G3, H3], 
        [A2, B2, C2, D2, E2, F2, G2, H2], 
        [A1, B1, C1, D1, E1, F1, G1, H1], 
    ];

    #[rustfmt::skip]
    /// String labels for all the squares, for printing and parsing purposes
    pub const NAMES: [&str; Self::COUNT] = [
        "a1", "b1", "c1", "d1", "e1", "f1", "g1", "h1", 
        "a2", "b2", "c2", "d2", "e2", "f2", "g2", "h2",
        "a3", "b3", "c3", "d3", "e3", "f3", "g3", "h3", 
        "a4", "b4", "c4", "d4", "e4", "f4", "g4", "h4",
        "a5", "b5", "c5", "d5", "e5", "f5", "g5", "h5", 
        "a6", "b6", "c6", "d6", "e6", "f6", "g6", "h6",
        "a7", "b7", "c7", "d7", "e7", "f7", "g7", "h7", 
        "a8", "b8", "c8", "d8", "e8", "f8", "g8", "h8",
    ];


    /// Get the rank for the square as an index between 0 and 7.
    pub fn rank(&self) -> usize {
        (*self as usize) / 8
    }

    /// Get the file for the square as an index between 0 and 7.
    pub fn file(&self) -> usize {
        (*self as usize) % 8
    }

    /// Get the square "in front of" the current square, as determined by the
    /// player's side.
    pub fn forward(self, side: Color) -> Option<Self> {
        if side.is_white() {
            Self::ALL.get(self as usize + 8).copied()
        } else {
            Self::ALL.get((self as usize).saturating_sub(8)).copied()
        }
    }

    /// Get the square "behind" the current square, as determined by the
    /// player's side.
    pub fn backward(&self, side: Color) -> Option<Self> {
        self.forward(side.opp())
    }

    /// Get the Manhattan distance between two squares.
    pub fn distance(&self, other: Self) -> usize {
        let dx = self.file().abs_diff(other.file());
        let dy = self.rank().abs_diff(other.rank());

        dx + dy
    }

    /// Mirror a square across the board horizontally
    pub fn flip(&self) -> Self {
        ((*self as usize) ^ 56).into()
    }
}

impl From<usize> for Square {
    fn from(value: usize) -> Self {
        Self::ALL[value]
    }
}

impl Display for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Self::NAMES[*self as usize])?;
        Ok(())
    }
}

impl FromStr for Square {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self> {
        let idx = Self::NAMES
            .iter()
            .position(|&name| name == s.to_lowercase())
            .ok_or(anyhow!("Not a valid square identifier"))?;

        Ok(Self::ALL[idx].to_owned())
    }
}
