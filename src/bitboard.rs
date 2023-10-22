use std::ops::Deref;
use std::{ops::Div, fmt::Display};
use anyhow::anyhow;

use crate::board::Color;
use crate::parse;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct Bitboard(pub u64);

impl Bitboard {
    pub fn new(rank: u64, file: u64) -> Self {
        Bitboard((1 << 8*rank) << file)
    }

    pub fn rank(&self) -> u64 {
        self.0.trailing_zeros().div(8).into()
    }

    pub fn file(&self) -> u64 {
        (self.0.trailing_zeros() % 8).try_into().unwrap()
    }

    pub fn up(&self) -> Option<Self> {
        if self.0.leading_zeros() > 8 {
            Some(Bitboard(self.0 << 8))
        } else {
            None
        }
    }

    pub fn down(&self) -> Option<Self> {
        if self.0.trailing_zeros() > 8 {
            Some(Bitboard(self.0 >> 8))
        } else {
            None
        }
    }

    pub fn left(&self) -> Option<Self> {
        if self.file() > 0 { Some(Bitboard(self.0 >> 1)) } else { None }
    }

    pub fn right(&self) -> Option<Self> {
        if self.file() < 7 { Some(Bitboard(self.0 << 1)) } else { None }
    }

    pub fn up_left(&self) -> Option<Self> {
        self.up().and_then(|pos| pos.left())
    }

    pub fn up_right(&self) -> Option<Self> {
        self.up().and_then(|pos| pos.right())
    }

    pub fn down_left(&self) -> Option<Self> {
        self.down().and_then(|pos| pos.left())
    }

    pub fn down_right(&self) -> Option<Self> {
        self.down().and_then(|pos| pos.right())
    }

    pub fn forward(&self, side: Color) -> Option<Self> {
        match side {
            Color::White => self.up(),
            Color::Black => self.down()
        }
    }

    pub fn scan_up(&self) -> Vec<Self> {
        std::iter::successors(self.up(), |current| current.up()).collect()
    }

    pub fn scan_right(&self) -> Vec<Self> {
        std::iter::successors(self.right(), |current| current.right()).collect()
    }

    pub fn scan_down(&self) -> Vec<Self> {
        std::iter::successors(self.down(), |current| current.down()).collect()
    }

    pub fn scan_left(&self) -> Vec<Self> {
        std::iter::successors(self.left(), |current| current.left()).collect()
    }

    pub fn scan_up_left(&self) -> Vec<Self> {
        std::iter::successors(self.up_left(), |current| current.up_left())
            .collect()
    }

    pub fn scan_up_right(&self) -> Vec<Self> {
        std::iter::successors(self.up_right(), |current| current.up_right())
            .collect()
    }

    pub fn scan_down_left(&self) -> Vec<Self> {
        std::iter::successors(self.down_left(), |current| current.down_left())
            .collect()
    }

    pub fn scan_down_right(&self) -> Vec<Self> {
        std::iter::successors(self.down_right(), |current| current.down_right())
            .collect()
    }

    pub fn scan<F: Fn(&Bitboard) -> Option<Bitboard>>(&self, next: F) -> Vec<Self> {
        std::iter::successors(next(self), |pos| next(pos)).collect()
    }

    pub fn add_in_place(&mut self, positions: Self) {
        self.0 = self.0 | positions.0;
    }

    pub fn add(&self, bitboard: Self) -> Bitboard {
        Bitboard(self.0 | bitboard.0)
    }


    pub fn remove(&mut self, positions: Self) {
        self.0 = self.0 & !positions.0;
    }

    pub fn contains(&self, positions: Self) -> bool {
        self.0 & positions.0 != 0
    }

    pub fn bits(&self) -> u64 {
        self.0
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
        let rank = (self.rank() + 1).to_string();

        let file = match self.file() {
            0 => "a",
            1 => "b",
            2 => "c",
            3 => "d",
            4 => "e",
            5 => "f",
            6 => "g",
            7 => "h",
            _ => panic!("unreachable")
        }.to_string();

        write!(f, "{}", vec![file, rank].join(""))
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
        let (_, (file, rank)) = parse::algebraic_square(value)
            .map_err(|_| anyhow!("Failed to parse"))?;
        Ok(Bitboard::new(rank, file))
    }
}

impl Iterator  for Bitboard {
    type Item = Bitboard;

    fn next(&mut self) -> Option<Self::Item> {
        // Get the next lsb
        let lsb = 1u64.checked_shl(self.0.trailing_zeros())?;

        // set the current lsb to zero
        self.0 = self.0 ^ lsb;
        
        Some(Bitboard(lsb))
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
    fn position_up() {
        assert_eq!(Bitboard::new(3,7).up(), Some(Bitboard::new(4,7)));
        assert_eq!(Bitboard::new(7,7).up(), None);
    }
}
