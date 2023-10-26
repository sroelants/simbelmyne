use std::str::FromStr;
use anyhow::anyhow;

use crate::{board::Color, bitboard::Bitboard};

use super::moves::Move;

pub enum CastleType {
    WQ,
    WK,
    BQ,
    BK
}

impl CastleType {
    pub fn get(side: Color, destination_file: u8) -> Self {
        match (side, destination_file) {
            (Color::White, 2) => Self::WQ,
            (Color::White, 5) => Self::WK,
            (Color::Black, 2) => Self::BQ,
            (Color::Black, 5) => Self::BK,
             _ => unreachable!()
        }
    }

    pub fn color(&self) -> Color {
        match self {
            CastleType::WQ | CastleType::WK => Color::White,
            CastleType::BQ | CastleType::BK => Color::Black,
        }
    }

    pub fn get_all() -> [CastleType; 4] {
        [ CastleType::WQ, CastleType::WK, CastleType::BQ, CastleType::BK ]
    }
}

#[derive(Default, Clone, Copy, Debug)]
pub struct CastlingRights(u8);

/// Index into as `DESTINATIONS[side: Color][castle_type: CastlyType]`
pub const DESTINATIONS: [Bitboard; 4] = 
    [  Bitboard(0x0000000000000002), // White Queenside
       Bitboard(0x0000000000000040), // White Kingside
       Bitboard(0x0200000000000000), // Black Queenside
       Bitboard(0x4000000000000000)  // Black Kingside
    ];

pub const VULN_SQUARES: [Bitboard; 4] = 
    [  Bitboard(0x000000000000001F),  // White Queenside
       Bitboard(0x00000000000000F0),   // White Kingside
       Bitboard(0x1f00000000000000), // Black Queenside
       Bitboard(0xf000000000000000)  // Black Kingside
    ];

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

    pub fn is_available(&self, ctype: &CastleType) -> bool {
        match ctype {
            CastleType::WQ => self.0 & Self::WQ.0 != 0,
            CastleType::WK => self.0 & Self::WK.0 != 0,
            CastleType::BQ => self.0 & Self::BQ.0 != 0,
            CastleType::BK => self.0 & Self::BK.0 != 0,
        }
    }

    pub fn get_available(&self, side: Color) -> Vec<CastleType> {
        CastleType::get_all()
            .into_iter()
            .filter(|ctype| ctype.color() == side)
            .filter(|ctype| self.is_available(ctype))
            .collect()
    }
}

pub fn rook_castle_move(ctype: CastleType) -> Move {
    match ctype {
        CastleType::WQ => Move::new(Bitboard::new(0,0), Bitboard::new(0,3)),
        CastleType::WK => Move::new(Bitboard::new(0,7), Bitboard::new(0,5)),
        CastleType::BQ => Move::new(Bitboard::new(7,0), Bitboard::new(7,3)),
        CastleType::BK => Move::new(Bitboard::new(7,7), Bitboard::new(7,5)),
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

