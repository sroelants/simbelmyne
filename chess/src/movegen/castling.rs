use crate::board::Board;
use crate::square::Square;
use crate::piece::Color;
use crate::movegen::moves::Move;
use std::fmt::Display;
use std::ops::Index;
use std::ops::IndexMut;
use std::str::FromStr;
use anyhow::anyhow;
use Square::*;

/// Type that represents one of the four castling options:
///
/// White Queenside (WQ), White Kingside (WK), Black Queenside (BQ) and Black
/// Kingside (BK)
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[repr(u8)]
pub enum CastleType {
    WQ, WK, BQ, BK,
}

impl CastleType {
    pub const ALL: [CastleType; 4] = [ 
        CastleType::WQ, 
        CastleType::WK, 
        CastleType::BQ, 
        CastleType::BK 
    ];

    pub const KING_TARGETS: [Square; 4] = [C1, G1, C8, G8];
    pub const ROOK_TARGETS: [Square; 4] = [D1, F1, D8, F8];

    /// Get the castling rights from an index.
    /// Returns None if the index is out of range
    pub fn new(idx: u8) -> Option<Self> {
        if idx < 4 {
            Some(unsafe { std::mem::transmute::<u8, Self>(idx) })
        } else {
            None
        }
    }

    /// Get the castling rights from an index.
    /// 
    /// SAFETY: Does not check that the index is in range (< 4), so be 
    /// absolutely sure the index was obtained from a legal castle type
    pub unsafe fn new_unchecked(idx: u8) -> Self {
        unsafe { std::mem::transmute::<u8, Self>(idx) }
    }

    /// Return the color of the side playing the Castle move
    pub fn color(&self) -> Color {
        match self {
            CastleType::WQ | CastleType::WK => Color::White,
            CastleType::BQ | CastleType::BK => Color::Black,
        }
    }

    /// Try and obtain the CastleType from a provided king move.
    ///
    /// This is a pretty cheap check, so only use it if you already know the
    /// move was a castling move!
    pub fn from_move(mv: Move) -> Option<Self> {
        use Square::*;

        match mv.tgt() {
            C1 => Some(CastleType::WQ),
            G1 => Some(CastleType::WK),
            C8 => Some(CastleType::BQ),
            G8 => Some(CastleType::BK),
            _ => None,
        }
    }

    pub fn rook_target(self) -> Square {
        Self::ROOK_TARGETS[self]
    }

    pub fn king_target(self) -> Square {
        Self::KING_TARGETS[self]
    }

    fn get_for(side: Color) -> &'static [CastleType] {
        if side.is_white() {
            &Self::ALL[..2]
        } else {
            &Self::ALL[2..]
        }
    }
}

/// Type that represents the remaining Castling Rights for a particular 
/// board.
///
/// These only take into account king/rook moves, not temporary conditions such 
/// as attacked squares.
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CastlingRights([Option<Square>; 4]);

impl CastlingRights {
    /// An empty set of castling rights
    pub fn none() -> Self {
        Self([None; 4])
    }

    /// Add an additional set of castling rights
    pub fn add(&mut self, ctype: CastleType, sq: Square) {
        self[ctype] = Some(sq);
    }

    /// Remove a set of castling rights
    pub fn remove(&mut self, ctype: CastleType) {
        self[ctype] = None;
    }

    /// Check whether the requested Castle is still available
    pub fn is_available(&self, ctype: CastleType) -> bool {
        self[ctype].is_some()
    }

    /// Return all the available castling types for a given side
    pub fn get_available(&self, side: Color) -> impl Iterator<Item = &CastleType> {
        CastleType::get_for(side)
            .iter()
            .filter(|&&ctype| self.is_available(ctype))
    }

    pub fn remove_for(&mut self, side: Color) {
        use CastleType::*;

        if side.is_white() {
            self.remove(WQ);
            self.remove(WK);
        } else {
            self.remove(BQ);
            self.remove(BK);
        }
    }
}

impl Index<CastleType> for CastlingRights {
    type Output = Option<Square>;

    fn index(&self, index: CastleType) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<CastleType> for CastlingRights {
    fn index_mut(&mut self, index: CastleType) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl Index<CastleType> for Board {
    type Output = Option<Square>;

    fn index(&self, ctype: CastleType) -> &Self::Output {
        &self.castling_rights[ctype]
    }
}

////////////////////////////////////////////////////////////////////////////////
//
// Utility traits
//
////////////////////////////////////////////////////////////////////////////////

impl FromStr for CastlingRights {
    type Err = anyhow::Error;

    /// Parse the castling rights from a FEN string
    /// rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1
    ///                                               ^^^^
    fn from_str(castling_str: &str) -> Result<Self, Self::Err> {
        use Square::*;
        use CastleType::*;
        let mut rights = CastlingRights::none();

        for ch in castling_str.chars() {
            match ch {
                'Q' => rights.add(WQ, A1),
                'K' => rights.add(WK, H1),
                'q' => rights.add(BQ, A8),
                'k' => rights.add(BK, H8),
                '-' => {}
                _ => Err(anyhow!("Invalid FEN string"))?,
            }
        }

        Ok(rights)
    }
}

impl Display for CastlingRights {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use CastleType::*;

        if self.is_available(WK) {
            write!(f, "K")?;
        }

        if self.is_available(WQ) {
            write!(f, "Q")?;
        }

        if self.is_available(BK) {
            write!(f, "k")?;
        }

        if self.is_available(BQ) {
            write!(f, "q")?;
        }

        if self.0 == [None, None, None, None] {
            write!(f, "-")?;
        }

        Ok(())
    }
}

impl<T> Index<CastleType> for [T; 4] {
    type Output = T;

    fn index(&self, index: CastleType) -> &Self::Output {
        // SAFETY: the legal values for this type are all in bounds.
        unsafe { self.get_unchecked(index as usize) }
    }
}

impl<T> IndexMut<CastleType> for [T; 4] {
    fn index_mut(&mut self, index: CastleType) -> &mut Self::Output {
        // SAFETY: the legal values for this type are all in bounds.
        unsafe { self.get_unchecked_mut(index as usize) }
    }
}
