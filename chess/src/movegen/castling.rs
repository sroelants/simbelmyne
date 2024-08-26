use crate::bitboard::Bitboard;
use crate::board::Board;
use crate::square::Square;
use crate::piece::Color;
use crate::movegen::moves::MoveType;
use crate::movegen::moves::Move;
use anyhow::anyhow;
use std::fmt::Display;
use std::ops::Index;
use std::ops::IndexMut;
use std::str::FromStr;
use Square::*;

impl Board {
    /// Return an iterator over the legal castle types for the current side
    ///
    /// Castling is permitted only if
    /// - neither the king nor the rook has previously moved (cf. CastlingRights)
    /// - the squares between the king and the rook are vacant
    /// - the king does not leave, cross over, or finish on a square attacked by
    ///   an enemy piece.
    pub fn legal_castles(&self) -> impl Iterator<Item=CastleType> {
        let threats = self.get_threats();
        let blockers = self.all_occupied();

        self.castling_rights
            .get_available(self.current)
            .filter(move |ctype| {
                let attacked = ctype.vulnerable_squares() & threats;
                let blocked = ctype.los_squares() & blockers;

                attacked.is_empty() && blocked.is_empty()
            })
    }
}

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

    const KING_MOVES: [Move; 4] = [
        Move::new(E1, C1, MoveType::QueenCastle),
        Move::new(E1, G1, MoveType::KingCastle),
        Move::new(E8, C8, MoveType::QueenCastle),
        Move::new(E8, G8, MoveType::KingCastle),
    ];

    const ROOK_MOVES: [Move; 4] = [
        Move::new(A1, D1, MoveType::QueenCastle),
        Move::new(H1, F1, MoveType::KingCastle),
        Move::new(A8, D8, MoveType::QueenCastle),
        Move::new(H8, F8, MoveType::KingCastle),
    ];

    const VULNERABLE_SQUARES: [Bitboard; 4] = [
        Bitboard(0x000000000000001C),
        Bitboard(0x0000000000000070),
        Bitboard(0x1C00000000000000),
        Bitboard(0x7000000000000000),
    ];

    const LOS_SQUARES: [Bitboard; 4] = [
        Bitboard(0x000000000000000E),
        Bitboard(0x0000000000000060),
        Bitboard(0x0E00000000000000),
        Bitboard(0x6000000000000000),
    ];

    const MIRRORED: [Self; 4] = [
        Self::BQ,
        Self::BK,
        Self::BQ,
        Self::BK,
    ];

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
    /// Returns None if the move was not a valid castle
    pub fn from_move(mv: Move) -> Option<Self> {
        match (mv.src(), mv.tgt()) {
            (E1, C1) => Some(CastleType::WQ),
            (E1, G1) => Some(CastleType::WK),
            (E8, C8) => Some(CastleType::BQ),
            (E8, G8) => Some(CastleType::BK),
            _ => None,
        }
    }

    /// Get the king's move for this castle type
    pub fn king_move(self) -> Move {
        Self::KING_MOVES[self]
    }

    /// Get the rook's  move for this castle type
    pub fn rook_move(self) -> Move {
        Self::ROOK_MOVES[self]
    }

    /// The squares we should check for attacks to see whether this castle is
    /// allowed.
    fn vulnerable_squares(self) -> Bitboard {
        Self::VULNERABLE_SQUARES[self]
    }

    /// The line-of-sight squares we should check for occupation to see whether 
    /// this castle is allowed.
    fn los_squares(self) -> Bitboard {
        Self::LOS_SQUARES[self]
    }

    pub fn mirror(self) -> Self {
        Self::MIRRORED[self]
    }
}

/// Type that represents the remaining Castling Rights for a particular 
/// board.
///
/// These only take into account king/rook moves, not temporary conditions such 
/// as attacked squares.
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CastlingRights(pub u8);

impl CastlingRights {
    const MASKS: [CastlingRights; 4] = [
        CastlingRights(0b0001), // WQ
        CastlingRights(0b0010), // WK
        CastlingRights(0b0100), // BQ
        CastlingRights(0b1000), // BK
    ];

    pub const WQ: CastlingRights = CastlingRights(0b0001);
    pub const WK: CastlingRights = CastlingRights(0b0010);
    pub const BQ: CastlingRights = CastlingRights(0b0100);
    pub const BK: CastlingRights = CastlingRights(0b1000);
    pub const WHITE: CastlingRights = CastlingRights(0b0011);
    pub const BLACK: CastlingRights = CastlingRights(0b1100);
    pub const ALL: CastlingRights = CastlingRights(0b1111);

    /// An empty set of castling rights
    pub fn none() -> CastlingRights {
        CastlingRights(0)
    }

    /// Add an additional set of castling rights
    pub fn add(&mut self, ctype: CastleType) {
        self.0 |= Self::MASKS[ctype].0;
    }

    /// Remove a set of castling rights
    pub fn remove(&mut self, castle: CastleType) {
        self.0 = self.0 & !CastlingRights::MASKS[castle].0;
    }

    /// Check whether the requested Castle is still available
    pub fn is_available(&self, ctype: CastleType) -> bool {
        match ctype {
            CastleType::WQ => self.0 & Self::WQ.0 != 0,
            CastleType::WK => self.0 & Self::WK.0 != 0,
            CastleType::BQ => self.0 & Self::BQ.0 != 0,
            CastleType::BK => self.0 & Self::BK.0 != 0,
        }
    }

    /// Return all the available castling types for a given side
    pub fn get_available(&self, side: Color) -> Self {
        if side.is_white() {
            Self(self.0 & Self::WHITE.0)
        } else {
            Self(self.0 & Self::BLACK.0)
        }
    }
}

impl Iterator for CastlingRights {
    type Item = CastleType;

    fn next(&mut self) -> Option<Self::Item> {
        let idx = self.0.trailing_zeros() as u8;

        if idx < 8 {
            // SAFETY: The index is in bounds
            let ctype = unsafe { CastleType::new_unchecked(idx) };
            self.remove(ctype);

            Some(ctype)
        } else {
            None
        }
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
        let mut rights = CastlingRights::none();

        for ch in castling_str.chars() {
            match ch {
                'Q' => rights.add(CastleType::WQ),
                'K' => rights.add(CastleType::WK),
                'q' => rights.add(CastleType::BQ),
                'k' => rights.add(CastleType::BK),
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

        if self.0 == 0 {
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
