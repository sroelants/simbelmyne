use std::{fmt::Display, str::FromStr, ops::Not};
use anyhow::anyhow;

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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
    pub const ALL: [Self; Self::COUNT] = [
        PieceType::Pawn,
        PieceType::Knight,
        PieceType::Bishop,
        PieceType::Rook,
        PieceType::Queen,
        PieceType::King 
    ];
}

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Color {
    White = 0,
    Black = 1,
}

impl Color {
    pub const COUNT: usize = 2;

    pub fn opp(&self) -> Self {
        !*self
    }

    pub fn is_white(&self) -> bool {
        *self == Color::White
    }

    pub fn is_black(&self) -> bool {
        *self == Color::Black
    }

    pub fn to_fen(&self) -> String {
        if self.is_white() {
            String::from("w")
        } else {
            String::from("b")
        }
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Color::White => write!(f, "White")?,
            Color::Black => write!(f, "Black")?,
        }
        Ok(())
    }
}

impl FromStr for Color {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self> {
        match s {
            "w" | "W" | "white" | "White" => Ok(Color::White),
            "b" | "B" | "black" | "Black" => Ok(Color::Black),
            _ => Err(anyhow!("Not a valid color string"))?,
        }
    }
}
impl Not for Color {
    type Output = Color;

    fn not(self) -> Self::Output {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Piece {
    WP, BP, WN, BN, WB, BB, WR, BR, WQ, BQ, WK, BK
}

#[allow(dead_code)]
impl Piece {
    pub const COUNT: usize = 12;

    pub const ALL: [Self; Self::COUNT] = [
        Self::WP, 
        Self::BP, 
        Self::WN, 
        Self::BN, 
        Self::WB, 
        Self::BB, 
        Self::WR, 
        Self::BR, 
        Self::WQ, 
        Self::BQ, 
        Self::WK, 
        Self::BK
    ];

    pub fn new(ptype: PieceType, color: Color) -> Self {
        if color.is_white() {
            Self::ALL[2 * (ptype as usize)]
        } else {
            Self::ALL[2 * (ptype as usize) + 1]
        }
    }

    pub fn color(self) -> Color {
        if self as usize % 2 == 0 { Color::White } else { Color::Black }
    }

    pub fn piece_type(self) -> PieceType {
        PieceType::ALL[self as usize / 2]
    }

    pub fn is_pawn(&self) -> bool {
        self.piece_type() == PieceType::Pawn
    }

    pub fn is_rook(&self) -> bool {
        self.piece_type() == PieceType::Rook
    }

    pub fn is_knight(&self) -> bool {
        self.piece_type() == PieceType::Knight
    }

    pub fn is_bishop(&self) -> bool {
        self.piece_type() == PieceType::Bishop
    }

    pub fn is_queen(&self) -> bool {
        self.piece_type() == PieceType::Queen
    }

    pub fn is_king(&self) -> bool {
        self.piece_type() == PieceType::King
    }

    pub fn is_hv_slider(&self) -> bool {
        self.is_rook() || self.is_queen()
    }

    pub fn is_diag_slider(&self) -> bool {
        self.is_bishop() || self.is_queen()
    }

    pub fn is_slider(&self) -> bool {
        self.is_rook() || self.is_bishop() || self.is_queen()
    }
}

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
