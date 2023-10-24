use std::str::FromStr;
use anyhow::anyhow;

use crate::{board::Color, bitboard::Bitboard};

pub enum CastleType {
    Queen,
    King,
}

#[derive(Default, Clone, Copy, Debug)]
pub struct CastlingRights(u8);

/// Index into as `CASTLING_SQUARES[side: Color][castle_type: CastlyType]`
pub const CASTLING_SQUARES: [[Bitboard; 2]; 2] = 
    [[  Bitboard(0x000000000000000E),  // White Queenside
        Bitboard(0x0000000000000060)   // White Kingside
    ], [
        Bitboard(0x6000000000000000), // Black Queenside
        Bitboard(0x3800000000000000)  // Black Kingside
    ]];

impl CastlingRights {
    pub const WQ: CastlingRights = CastlingRights(0b0001);
    pub const WK: CastlingRights = CastlingRights(0b0010);
    pub const BQ: CastlingRights = CastlingRights(0b0100);
    pub const BK: CastlingRights = CastlingRights(0b1000);

    pub fn new() -> CastlingRights {
        CastlingRights(0b1111)
    }

    pub fn none() -> CastlingRights {
        CastlingRights(0)
    }

    pub fn add(&mut self, castle: CastlingRights) {
        self.0 = self.0 | castle.0;
    }

    pub fn remove(&mut self, castle: CastlingRights) {
        self.0 = self.0 & !castle.0;
    }

    pub fn toggle(&mut self, castle: CastlingRights) {
        self.0 = self.0 ^ castle.0;
    }

    pub fn has_kingside_rights(&self, side: Color) -> bool {
        match side {
            Color::White => self.0 & Self::WK.0 != 0,
            Color::Black => self.0 & Self::BK.0 != 0
        }
    }

    pub fn has_queenside_rights(&self, side: Color) -> bool {
        match side {
            Color::White => self.0 & Self::WQ.0 != 0,
            Color::Black => self.0 & Self::BQ.0 != 0
        }
    }

}

impl FromStr for CastlingRights {
    type Err = anyhow::Error;

    /// Parse the castling rights from a FEN string 
    /// rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1
    ///                                               ^^^^
    fn from_str(fen: &str) -> Result<Self, Self::Err> {
        let mut rights = CastlingRights::none();
        let castling_str = fen.split(" ").nth(2).ok_or(anyhow!("Invalid FEN string"))?;

        for ch in castling_str.chars() {
            match ch {
                'Q' => rights.add(CastlingRights::WQ),
                'K' => rights.add(CastlingRights::WK),
                'q' => rights.add(CastlingRights::BQ),
                'k' => rights.add(CastlingRights::BK),
                '-' => {},
                _ => Err(anyhow!("Invalid FEN string"))?
            }
        }

        Ok(rights)
    }
}

