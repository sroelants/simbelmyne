//! Logic pertaining to Pieces, Piece Types and Colors

use std::{fmt::Display, ops::{Index, IndexMut, Not}, str::FromStr};
use anyhow::anyhow;
use PieceType::*;
use Piece::*;
use Color::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
/// A Piece
/// A Piece combines a Piece Type and Color in one entity
pub enum Piece {
    WP, BP, WN, BN, WB, BB, WR, BR, WQ, BQ, WK, BK
}

impl Piece {
    pub const COUNT: usize = 12;

    pub fn new(ptype: PieceType, color: Color) -> Self {
        match (color, ptype) {
            (White, Pawn)   => WP,
            (White, Knight) => WN,
            (White, Bishop) => WB,
            (White, Rook)   => WR,
            (White, Queen)  => WQ,
            (White, King)   => WK,

            (Black, Pawn)   => BP,
            (Black, Knight) => BN,
            (Black, Bishop) => BB,
            (Black, Rook)   => BR,
            (Black, Queen)  => BQ,
            (Black, King)   => BK
        }
    }

    /// Get the color of the piece
    pub fn color(self) -> Color {
        if (self as usize) & 1 == 0 { Color::White } else { Color::Black }
    }

    /// Get the piece type
    pub fn piece_type(self) -> PieceType {
        match self {
            WP | BP => Pawn,
            WN | BN => Knight,
            WB | BB => Bishop,
            WR | BR => Rook,
            WQ | BQ => Queen,
            WK | BK => King,
        }
    }

    /// Check whether the piece is a pawn
    pub fn is_pawn(&self) -> bool {
        self.piece_type() == PieceType::Pawn
    }

    /// Check whether the piece is a knight
    pub fn is_knight(&self) -> bool {
        self.piece_type() == PieceType::Knight
    }

    /// Check whether the piece is a bishop
    pub fn is_bishop(&self) -> bool {
        self.piece_type() == PieceType::Bishop
    }

    /// Check whether the piece is a rook
    pub fn is_rook(&self) -> bool {
        self.piece_type() == PieceType::Rook
    }

    /// Check whether the piece is a queen
    pub fn is_queen(&self) -> bool {
        self.piece_type() == PieceType::Queen
    }

    /// Check whether the piece is a king
    pub fn is_king(&self) -> bool {
        self.piece_type() == PieceType::King
    }

    /// Check whether the piece is a slider
    pub fn is_slider(&self) -> bool {
        self.is_rook() || self.is_bishop() || self.is_queen()
    }

    /// Check whether the piece is a horizontal/vertical slider (rook or queen)
    pub fn is_hv_slider(&self) -> bool {
        self.is_rook() || self.is_queen()
    }

    /// Check whether the piece is a diagonal slider (bishop or queen)
    pub fn is_diag_slider(&self) -> bool {
        self.is_bishop() || self.is_queen()
    }

    pub fn mirror(self) -> Self {
        use Piece::*;

        match self {
            WP => BP,
            WN => BN,
            WB => BB,
            WR => BR,
            WQ => BQ,
            WK => BK,

            BP => WP,
            BN => WN,
            BB => WB,
            BR => WR,
            BQ => WQ,
            BK => WK,
        }
    }

}

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
/// The type of a Piece
pub enum PieceType {
    Pawn = 0,
    Knight = 1,
    Bishop = 2,
    Rook = 3,
    Queen = 4,
    King = 5,
}

impl PieceType {
    pub const COUNT: usize = 6;
}

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
/// The color of a piece
/// 
/// Also used to represent players, etc...

pub enum Color {
    White = 0,
    Black = 1,
}

impl Color {
    pub const COUNT: usize = 2;

    /// Get the opposite color
    pub fn opp(self) -> Self {
        !self
    }

    /// Check whether the color is white
    pub fn is_white(self) -> bool {
        self == White
    }

    /// Check whether the color is black
    pub fn is_black(self) -> bool {
        self == Black
    }
}

////////////////////////////////////////////////////////////////////////////////
//
// Utility Traits
//
////////////////////////////////////////////////////////////////////////////////

impl Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Piece::*;
        let piece = match *self {
            WP => "P",
            WR => "R",
            WN => "N",
            WB => "B",
            WQ => "Q",
            WK => "K",

            BP => "p",
            BR => "r",
            BN => "n",
            BB => "b",
            BQ => "q",
            BK => "k",
        };

        write!(f, "{piece}")
    }
}

impl FromStr for Piece {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self> {
        use Piece::*;

        match s {
            "P" => Ok(WP),
            "N" => Ok(WN),
            "B" => Ok(WB),
            "R" => Ok(WR),
            "Q" => Ok(WQ),
            "K" => Ok(WK),
            "p" => Ok(BP),
            "n" => Ok(BN),
            "b" => Ok(BB),
            "r" => Ok(BR),
            "q" => Ok(BQ),
            "k" => Ok(BK),
            _ => Err(anyhow!("Not a valid piece string"))?,
        }
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            White => write!(f, "w")?,
            Black => write!(f, "b")?,
        }
        Ok(())
    }
}

impl FromStr for Color {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self> {
        match s {
            "w" => Ok(White),
            "b" => Ok(Black),
            _ => Err(anyhow!("Not a valid color string"))?,
        }
    }
}

impl Not for Color {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            White => Black,
            Black => White,
        }
    }
}

// Index traits, yoinked from viri

impl<T> Index<Color> for [T; 2] {
    type Output = T;

    fn index(&self, index: Color) -> &Self::Output {
        // SAFETY: the legal values for this type are all in bounds.
        unsafe { self.get_unchecked(index as usize) }
    }
}

impl<T> IndexMut<Color> for [T; 2] {
    fn index_mut(&mut self, index: Color) -> &mut Self::Output {
        // SAFETY: the legal values for this type are all in bounds.
        unsafe { self.get_unchecked_mut(index as usize) }
    }
}

impl<T> Index<PieceType> for [T; 6] {
    type Output = T;

    fn index(&self, index: PieceType) -> &Self::Output {
        // SAFETY: the legal values for this type are all in bounds.
        unsafe { self.get_unchecked(index as usize) }
    }
}

impl<T> IndexMut<PieceType> for [T; 6] {
    fn index_mut(&mut self, index: PieceType) -> &mut Self::Output {
        // SAFETY: the legal values for this type are all in bounds.
        unsafe { self.get_unchecked_mut(index as usize) }
    }
}

impl<T> Index<Piece> for [T; 12] {
    type Output = T;

    fn index(&self, index: Piece) -> &Self::Output {
        // SAFETY: the legal values for this type are all in bounds.
        unsafe { self.get_unchecked(index as usize) }
    }
}

impl<T> IndexMut<Piece> for [T; 12] {
    fn index_mut(&mut self, index: Piece) -> &mut Self::Output {
        // SAFETY: the legal values for this type are all in bounds.
        unsafe { self.get_unchecked_mut(index as usize) }
    }
}


