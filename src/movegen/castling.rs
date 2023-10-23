use std::str::FromStr;
use anyhow::anyhow;

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

