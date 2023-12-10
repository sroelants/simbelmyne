use crate::{
    bitboard::Bitboard,
    board::Board, 
    square::Square, 
    piece::Color, 
    movegen::moves::MoveType,
};
use anyhow::anyhow;
use std::{fmt::Display, str::FromStr};

const KING_SOURCES: [Square; 4] = [
    Square::E1, // White Queenside
    Square::E1, // White Kingside
    Square::E8, // Black Queenside
    Square::E8, // Black Kingside
];

const KING_TARGETS: [Square; 4] = [
    Square::C1, // White Queenside
    Square::G1, // White Kingside
    Square::C8, // Black Queenside
    Square::G8, // Black Kingside
];

const ROOK_SOURCES: [Square; 4] = [
    Square::A1, // White Queenside
    Square::H1, // White Kingside
    Square::A8, // Black Queenside
    Square::H8, // Black Kingside
];

const ROOK_TARGETS: [Square; 4] = [
    Square::D1, // White Queenside
    Square::F1, // White Kingside
    Square::D8, // Black Queenside
    Square::F8, // Black Kingside
];

const VULNERABLE_SQUARES: [Bitboard; 4] = [
    Bitboard(0x000000000000001C), // White Queenside
    Bitboard(0x0000000000000070), // White Kingside
    Bitboard(0x1C00000000000000), // Black Queenside
    Bitboard(0x7000000000000000), // Black Kingside
];

const OCCUPIABLE_SQUARES: [Bitboard; 4] = [
    Bitboard(0x000000000000000E), // White Queenside
    Bitboard(0x0000000000000060), // White Kingside
    Bitboard(0x0E00000000000000), // Black Queenside
    Bitboard(0x6000000000000000), // Black Kingside
];

use super::moves::Move;
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
/// Type that represents one of the four castling options:
/// White Queenside (WQ), White Kingside (WK), Black Queenside (BQ) and Black
/// Kingside (BK)
pub enum CastleType {
    WQ = 0,
    WK = 1,
    BQ = 2,
    BK = 3,
}

impl CastleType {
    /// Return the color of the side playing the Castle move
    pub fn color(&self) -> Color {
        match self {
            CastleType::WQ | CastleType::WK => Color::White,
            CastleType::BQ | CastleType::BK => Color::Black,
        }
    }

    /// Try and obtain the CastleType from a provided move.
    /// Returns None if the move was not a valid castle
    pub fn from_move(mv: Move) -> Option<Self> {
        let idx = KING_TARGETS
            .into_iter()
            .position(|tgt| tgt == mv.tgt().into())?;

        match idx {
            0 => Some(CastleType::WQ),
            1 => Some(CastleType::WK),
            2 => Some(CastleType::BQ),
            3 => Some(CastleType::BK),
            _ => None,
        }
    }

    pub fn get_all() -> [CastleType; 4] {
        [
            CastleType::WQ,
            CastleType::WK,
            CastleType::BQ,
            CastleType::BK,
        ]
    }

    /// Check whether this particular castle is allowed according to the rules
    ///
    /// Castling is permitted only if
    /// - neither the king nor the rook has previously moved (cf. CastlingRights)
    /// - the squares between the king and the rook are vacant
    /// - the king does not leave, cross over, or finish on a square attacked by
    ///   an enemy piece.
    pub fn is_allowed(self, board: &Board) -> bool {
        let is_attacked = self
            .vulnerable_squares()
            .has_overlap(board.attacked_by(self.color().opp()));

        let is_occupied = self.occupiable_squares().has_overlap(board.all_occupied());

        !is_attacked && !is_occupied
    }

    /// Get the king's source square for this castle type
    fn king_source(self) -> Square {
        KING_SOURCES[self as usize]
    }

    /// Get the king's target square for this castle type
    fn king_target(self) -> Square {
        KING_TARGETS[self as usize]
    }

    /// Get the king's move for this castle type
    pub fn king_move(&self) -> Move {
        use CastleType::*;
        use MoveType::*;

        let mtype = match self {
            WQ | BQ => QueenCastle,
            WK | BK => KingCastle,
        };

        Move::new(self.king_source(), self.king_target(), mtype)
    }

    /// Get the rook's source square for this castle type
    fn rook_source(self) -> Square {
        ROOK_SOURCES[self as usize]
    }

    /// Get the rook's target square for this castle type
    fn rook_target(self) -> Square {
        ROOK_TARGETS[self as usize]
    }

    /// Get the rook's  move for this castle type
    pub fn rook_move(self) -> Move {
        Move::new(self.rook_source(), self.rook_target(), MoveType::Quiet)
    }

    /// The squares we should check for attacks to see whether this castle is
    /// allowed.
    fn vulnerable_squares(self) -> Bitboard {
        VULNERABLE_SQUARES[self as usize]
    }

    /// The squares we should check for occupation to see whether this castle is
    /// allowed.
    fn occupiable_squares(self) -> Bitboard {
        OCCUPIABLE_SQUARES[self as usize]
    }
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CastlingRights(pub u8);

#[allow(dead_code)]
impl CastlingRights {
    pub const WQ: CastlingRights = CastlingRights(0b0001);
    pub const WK: CastlingRights = CastlingRights(0b0010);
    pub const BQ: CastlingRights = CastlingRights(0b0100);
    pub const BK: CastlingRights = CastlingRights(0b1000);
    pub const ALL: CastlingRights = CastlingRights(0b1111);

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

    pub fn is_available(&self, ctype: CastleType) -> bool {
        match ctype {
            CastleType::WQ => self.0 & Self::WQ.0 != 0,
            CastleType::WK => self.0 & Self::WK.0 != 0,
            CastleType::BQ => self.0 & Self::BQ.0 != 0,
            CastleType::BK => self.0 & Self::BK.0 != 0,
        }
    }

    pub fn get_available(&self) -> Vec<CastleType> {
        CastleType::get_all()
            .into_iter()
            .filter(|&ctype| self.is_available(ctype))
            .collect()
    }

    pub fn get_available_for(&self, side: Color) -> Vec<CastleType> {
        CastleType::get_all()
            .into_iter()
            .filter(|ctype| ctype.color() == side)
            .filter(|ctype| self.is_available(*ctype))
            .collect()
    }
}

impl FromStr for CastlingRights {
    type Err = anyhow::Error;

    /// Parse the castling rights from a FEN string
    /// rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1
    ///                                               ^^^^
    fn from_str(castling_str: &str) -> Result<Self, Self::Err> {
        let mut rights = CastlingRights::none();

        for ch in castling_str.chars() {
            match ch {
                'Q' => rights.add(CastlingRights::WQ),
                'K' => rights.add(CastlingRights::WK),
                'q' => rights.add(CastlingRights::BQ),
                'k' => rights.add(CastlingRights::BK),
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

impl CastlingRights {
    pub fn to_fen(&self) -> String {
        self.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::square::Square::*;

    #[test]
    fn color() {
        assert_eq!(CastleType::WQ.color(), Color::White);
        assert_eq!(CastleType::WK.color(), Color::White);
        assert_eq!(CastleType::BQ.color(), Color::Black);
        assert_eq!(CastleType::BK.color(), Color::Black);
    }

    // CastleType#from_move
    #[test]
    fn from_move() {
        let castle = Move::new(E1, G1, MoveType::KingCastle);

        let not_a_castle = Move::new(E1, H1, MoveType::Quiet);

        assert!(
            CastleType::from_move(castle).is_some(),
            "CastleType::from_move() returns Some(...) for a correct castle move"
        );

        assert_eq!(
            CastleType::from_move(castle).unwrap(),
            CastleType::WK,
            "CastleType::from_move() correctly decodes a move"
        );

        assert_eq!(
            CastleType::from_move(not_a_castle),
            None,
            "CastleType::from_move() returns None for an incorrect castle move"
        );
    }

    // CastleType#get_all
    #[test]
    fn get_all() {
        assert!(CastleType::get_all()
            .into_iter()
            .find(|&ct| ct == CastleType::WQ)
            .is_some());
        assert!(CastleType::get_all()
            .into_iter()
            .find(|&ct| ct == CastleType::WK)
            .is_some());
        assert!(CastleType::get_all()
            .into_iter()
            .find(|&ct| ct == CastleType::BQ)
            .is_some());
        assert!(CastleType::get_all()
            .into_iter()
            .find(|&ct| ct == CastleType::BK)
            .is_some());
    }

    // CastleType#king_source
    #[test]
    fn king_source() {
        assert_eq!(CastleType::WQ.king_source().rank(), 0);
        assert_eq!(CastleType::WQ.king_source().file(), 4);

        assert_eq!(CastleType::BK.king_source().rank(), 7);
        assert_eq!(CastleType::BK.king_source().file(), 4);
    }

    // CastleType#king_target
    #[test]
    fn king_target() {
        assert_eq!(CastleType::WQ.king_target().rank(), 0);
        assert_eq!(CastleType::WQ.king_target().file(), 2);

        assert_eq!(CastleType::BK.king_target().rank(), 7);
        assert_eq!(CastleType::BK.king_target().file(), 6);
    }

    // CastleType#king_move
    #[test]
    fn king_move() {
        assert_eq!(
            CastleType::WQ.king_move().src(),
            CastleType::WQ.king_source().into()
        );
        assert_eq!(
            CastleType::WQ.king_move().tgt(),
            CastleType::WQ.king_target().into()
        );
    }

    // CastleType#rook_source
    #[test]
    fn rook_source() {
        assert_eq!(CastleType::WQ.rook_source().rank(), 0);
        assert_eq!(CastleType::WQ.rook_source().file(), 0);

        assert_eq!(CastleType::BK.rook_source().rank(), 7);
        assert_eq!(CastleType::BK.rook_source().file(), 7);
    }

    // CastleType#rook_target
    #[test]
    fn rook_target() {
        assert_eq!(CastleType::WQ.rook_target().rank(), 0);
        assert_eq!(CastleType::WQ.rook_target().file(), 3);

        assert_eq!(CastleType::BK.rook_target().rank(), 7);
        assert_eq!(CastleType::BK.rook_target().file(), 5);
    }

    // CastleType#rook_move
    #[test]
    fn rook_move() {
        assert_eq!(
            CastleType::WQ.rook_move().src(),
            CastleType::WQ.rook_source().into()
        );
        assert_eq!(
            CastleType::WQ.rook_move().tgt(),
            CastleType::WQ.rook_target().into()
        );
    }

    // CastleType#attackable_squares
    #[test]
    fn attackable_squares() {
        assert!(CastleType::WQ
            .vulnerable_squares()
            .contains(Bitboard::new(0, 2)));
        assert!(CastleType::WQ
            .vulnerable_squares()
            .contains(Bitboard::new(0, 3)));
        assert!(CastleType::WQ
            .vulnerable_squares()
            .contains(Bitboard::new(0, 4)));

        assert!(CastleType::WK
            .vulnerable_squares()
            .contains(Bitboard::new(0, 4)));
        assert!(CastleType::WK
            .vulnerable_squares()
            .contains(Bitboard::new(0, 5)));
        assert!(CastleType::WK
            .vulnerable_squares()
            .contains(Bitboard::new(0, 6)));

        assert!(CastleType::BQ
            .vulnerable_squares()
            .contains(Bitboard::new(7, 2)));
        assert!(CastleType::BQ
            .vulnerable_squares()
            .contains(Bitboard::new(7, 3)));
        assert!(CastleType::BQ
            .vulnerable_squares()
            .contains(Bitboard::new(7, 4)));

        assert!(CastleType::BK
            .vulnerable_squares()
            .contains(Bitboard::new(7, 4)));
        assert!(CastleType::BK
            .vulnerable_squares()
            .contains(Bitboard::new(7, 5)));
        assert!(CastleType::BK
            .vulnerable_squares()
            .contains(Bitboard::new(7, 6)));
    }

    // CastleType#occupiable_squares
    #[test]
    fn occupiable_squares() {
        assert!(CastleType::WQ
            .occupiable_squares()
            .contains(Bitboard::new(0, 1)));
        assert!(CastleType::WQ
            .occupiable_squares()
            .contains(Bitboard::new(0, 2)));
        assert!(CastleType::WQ
            .occupiable_squares()
            .contains(Bitboard::new(0, 3)));

        assert!(CastleType::WK
            .occupiable_squares()
            .contains(Bitboard::new(0, 5)));
        assert!(CastleType::WK
            .occupiable_squares()
            .contains(Bitboard::new(0, 6)));

        assert!(CastleType::BQ
            .occupiable_squares()
            .contains(Bitboard::new(7, 1)));
        assert!(CastleType::BQ
            .occupiable_squares()
            .contains(Bitboard::new(7, 2)));
        assert!(CastleType::BQ
            .occupiable_squares()
            .contains(Bitboard::new(7, 3)));

        assert!(CastleType::BK
            .occupiable_squares()
            .contains(Bitboard::new(7, 5)));
        assert!(CastleType::BK
            .occupiable_squares()
            .contains(Bitboard::new(7, 6)));
    }

    // CastleType#is_allowed
    // Create a board where a passing square is under attack
    #[test]
    fn is_allowed_attacked() {
        let board = Board::from_str("r3k2r/8/3B4/8/8/3b4/8/R3K2R w KQkq - 0 1").unwrap();
        assert!(CastleType::BQ.is_allowed(&board));
        assert!(!CastleType::BK.is_allowed(&board));

        assert!(CastleType::WQ.is_allowed(&board));
        assert!(!CastleType::WK.is_allowed(&board));
    }

    // CastleType#is_allowed
    // Create a board where a passing square is occupied
    #[test]
    fn is_allowed_occupied() {
        let board = Board::from_str("rn2k2r/8/8/8/8/8/8/R3K1NR w KQkq - 0 1").unwrap();
        assert!(!CastleType::BQ.is_allowed(&board));
        assert!(CastleType::BK.is_allowed(&board));

        assert!(CastleType::WQ.is_allowed(&board));
        assert!(!CastleType::WK.is_allowed(&board));
    }

    // CastlingRights#add
    #[test]
    fn add_rights() {
        let mut rights = CastlingRights::none();
        rights.add(CastlingRights::WQ);

        assert!(rights.is_available(CastleType::WQ));
        assert!(!rights.is_available(CastleType::WK));
    }

    // CastlingRights#remove
    #[test]
    fn remove_rights() {
        let mut rights = CastlingRights::new();
        rights.remove(CastlingRights::WQ);

        assert!(!rights.is_available(CastleType::WQ));
        assert!(rights.is_available(CastleType::WK));
    }

    // CastlingRights#is_available
    #[test]
    fn is_available() {
        let rights = CastlingRights::new();
        assert!(rights.is_available(CastleType::WQ));
        assert!(rights.is_available(CastleType::BQ));
    }

    // CastlingRights#get_available
    #[test]
    fn get_available_for() {
        let mut rights = CastlingRights::new();
        rights.remove(CastlingRights::WQ);
        let available = rights.get_available_for(Color::White);

        assert!(available.contains(&CastleType::WK));
        assert!(!available.contains(&CastleType::WQ));
    }
}
