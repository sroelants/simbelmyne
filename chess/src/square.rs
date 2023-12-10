use anyhow::anyhow;

#[rustfmt::skip]
const SQUARE_NAMES: [&str; Square::COUNT] = [
    "a1", "b1", "c1", "d1", "e1", "f1", "g1", "h1", 
    "a2", "b2", "c2", "d2", "e2", "f2", "g2", "h2",
    "a3", "b3", "c3", "d3", "e3", "f3", "g3", "h3", 
    "a4", "b4", "c4", "d4", "e4", "f4", "g4", "h4",
    "a5", "b5", "c5", "d5", "e5", "f5", "g5", "h5", 
    "a6", "b6", "c6", "d6", "e6", "f6", "g6", "h6",
    "a7", "b7", "c7", "d7", "e7", "f7", "g7", "h7", 
    "a8", "b8", "c8", "d8", "e8", "f8", "g8", "h8",
];

#[rustfmt::skip]
#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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

use std::{fmt::Display, str::FromStr};

use Square::*;

use crate::piece::Color;
impl Square {
    pub const COUNT: usize = 64;
    pub const W_PAWN_RANK: usize = 1;
    pub const B_PAWN_RANK: usize = 6;
    pub const W_DPUSH_RANK: usize = 3;
    pub const B_DPUSH_RANK: usize = 4;
    pub const ALL: [Square; Square::COUNT] = [
        A1, B1, C1, D1, E1, F1, G1, H1, 
        A2, B2, C2, D2, E2, F2, G2, H2, 
        A3, B3, C3, D3, E3, F3, G3, H3, 
        A4, B4, C4, D4, E4, F4, G4, H4, 
        A5, B5, C5, D5, E5, F5, G5, H5, 
        A6, B6, C6, D6, E6, F6, G6, H6, 
        A7, B7, C7, D7, E7, F7, G7, H7, 
        A8, B8, C8, D8, E8, F8, G8, H8,
    ];
    pub fn new(rank: usize, file: usize) -> Square {
        Square::ALL[rank * 8 + file]
    }

    pub fn try_new(rank: usize, file: usize) -> Option<Square> {
        if rank <= 7 && file <= 7 {
            Some(Square::new(rank, file))
        } else {
            None
        }
    }

    pub fn try_from_usize(value: usize) -> Option<Square> {
        if value < 64 {
            Some(Square::ALL[value])
        } else {
            None
        }
    }

    pub const RANKS: [[Square; 8]; 8] = [
        [A8, B8, C8, D8, E8, F8, G8, H8],
        [A7, B7, C7, D7, E7, F7, G7, H7], 
        [A6, B6, C6, D6, E6, F6, G6, H6], 
        [A5, B5, C5, D5, E5, F5, G5, H5], 
        [A4, B4, C4, D4, E4, F4, G4, H4], 
        [A3, B3, C3, D3, E3, F3, G3, H3], 
        [A2, B2, C2, D2, E2, F2, G2, H2], 
        [A1, B1, C1, D1, E1, F1, G1, H1], 
    ];

    pub fn rank(&self) -> usize {
        (*self as usize) / 8
    }

    pub fn file(&self) -> usize {
        (*self as usize) % 8
    }

    pub fn forward(&self, side: Color) -> Option<Square> {
        if side.is_white() {
            Square::try_new(self.rank() + 1, self.file())
        } else {
            Square::try_new(self.rank() - 1, self.file())
        }
    }

    pub fn backward(&self, side: Color) -> Option<Square> {
        self.forward(side.opp())
    }

    pub fn is_double_push(source: Square, target: Square) -> bool {
        (source.rank() == Self::W_PAWN_RANK && target.rank() == Self::W_DPUSH_RANK
            || source.rank() == Self::B_PAWN_RANK && target.rank() == Self::B_DPUSH_RANK)
            && source.file() == target.file()
    }

    pub fn flip(&self) -> Self {
        ((*self as usize) ^ 56).into()
    }
}

impl From<usize> for Square {
    fn from(value: usize) -> Self {
        Square::ALL[value]
    }
}

impl Display for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", SQUARE_NAMES[*self as usize])?;
        Ok(())
    }
}

impl FromStr for Square {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self> {
        let idx = SQUARE_NAMES
            .iter()
            .position(|&name| name == s.to_lowercase())
            .ok_or(anyhow!("Not a valid square identifier"))?;

        Ok(Square::ALL[idx].to_owned())
    }
}
