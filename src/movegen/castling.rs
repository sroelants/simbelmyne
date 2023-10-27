use std::str::FromStr;
use anyhow::anyhow;
use crate::{board::{Color, Board}, bitboard::Bitboard};

const KING_SOURCES: [Bitboard; 4] = [
   Bitboard(0x0000000000000010), // White Queenside
   Bitboard(0x0000000000000010), // White Kingside
   Bitboard(0x1000000000000000), // Black Queenside
   Bitboard(0x1000000000000000)  // Black Kingside
];

const KING_TARGETS: [Bitboard; 4] = [  
   Bitboard(0x0000000000000004), // White Queenside
   Bitboard(0x0000000000000040), // White Kingside
   Bitboard(0x0400000000000000), // Black Queenside
   Bitboard(0x4000000000000000)  // Black Kingside
];

const ROOK_SOURCES: [Bitboard; 4] = [  
    Bitboard(0x0000000000000001), // White Queenside
    Bitboard(0x0000000000000080), // White Kingside
    Bitboard(0x0100000000000000), // Black Queenside
    Bitboard(0x8000000000000000)  // Black Kingside
];

const ROOK_TARGETS: [Bitboard; 4] = [  
    Bitboard(0x0000000000000008), // White Queenside
    Bitboard(0x0000000000000020), // White Kingside
    Bitboard(0x0800000000000000), // Black Queenside
    Bitboard(0x2000000000000000)  // Black Kingside
];

const VULN_SQUARES: [Bitboard; 4] = [  
   Bitboard(0x000000000000001C), // White Queenside
   Bitboard(0x0000000000000070), // White Kingside
   Bitboard(0x1C00000000000000), // Black Queenside
   Bitboard(0x7000000000000000)  // Black Kingside
];

use super::moves::Move;
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum CastleType {
    WQ = 0,
    WK = 1,
    BQ = 2,
    BK = 3
}

impl CastleType {
    pub fn color(&self) -> Color {
        match self {
            CastleType::WQ | CastleType::WK => Color::White,
            CastleType::BQ | CastleType::BK => Color::Black,
        }
    }

    pub fn from_move(mv: &Move) -> Option<Self> {
        let idx = KING_TARGETS.into_iter().position(|tgt| tgt == mv.tgt())?;
        eprintln!("Index is {idx}");

        match idx {
            0 => Some(CastleType::WQ),
            1 => Some(CastleType::WK),
            2 => Some(CastleType::BQ),
            3 => Some(CastleType::BK),
            _ => None
        }
    }

    pub fn get_all() -> [CastleType; 4] {
        [ CastleType::WQ, CastleType::WK, CastleType::BQ, CastleType::BK ]
    }

    pub fn is_allowed(self, board: &Board) -> bool {
        let opp = self.color().opp();
        let attacked_squares = board.attacked_squares[opp as usize];
        !VULN_SQUARES[self as usize].has_overlap(attacked_squares)
    }

    pub fn king_source(self) -> Bitboard {
        KING_SOURCES[self as usize]
    }

    pub fn king_target(self) -> Bitboard {
        KING_TARGETS[self as usize]
    }

    pub fn king_move(&self) -> Move {
        let mut mv = Move::new(self.king_source(), self.king_target());

        mv.set_castle();
        mv
    }

    pub fn rook_source(self) -> Bitboard {
        ROOK_SOURCES[self as usize]
    }

    pub fn rook_target(self) -> Bitboard {
        ROOK_TARGETS[self as usize]
    }

    pub fn rook_move(self) -> Move {
        Move::new(self.rook_source(), self.rook_target())
    }
}

#[derive(Default, Clone, Copy, Debug)]
pub struct CastlingRights(u8);

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

