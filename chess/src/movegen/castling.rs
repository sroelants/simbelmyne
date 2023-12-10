use crate::bitboard::Bitboard;
use crate::board::Board;
use crate::square::Square;
use crate::piece::Color;
use crate::movegen::moves::MoveType;
use crate::movegen::moves::Move;
use anyhow::anyhow;
use std::fmt::Display;
use std::str::FromStr;
use Square::*;

const KING_SOURCES: [Square; 4] = [
    E1, // White Queenside
    E1, // White Kingside
    E8, // Black Queenside
    E8, // Black Kingside
];

const KING_TARGETS: [Square; 4] = [
    C1, // White Queenside
    G1, // White Kingside
    C8, // Black Queenside
    G8, // Black Kingside
];

const ROOK_SOURCES: [Square; 4] = [
    A1, // White Queenside
    H1, // White Kingside
    A8, // Black Queenside
    H8, // Black Kingside
];

const ROOK_TARGETS: [Square; 4] = [
    D1, // White Queenside
    F1, // White Kingside
    D8, // Black Queenside
    F8, // Black Kingside
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

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
/// Type that represents one of the four castling options:
///
/// White Queenside (WQ), White Kingside (WK), Black Queenside (BQ) and Black
/// Kingside (BK)
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

    /// Check whether this particular castle is allowed according to the rules
    ///
    /// Castling is permitted only if
    /// - neither the king nor the rook has previously moved (cf. CastlingRights)
    /// - the squares between the king and the rook are vacant
    /// - the king does not leave, cross over, or finish on a square attacked by
    ///   an enemy piece.
    pub fn is_allowed(self, board: &Board) -> bool {
        let attacked_squares = board.attacked_by(self.color().opp());
        let is_attacked = !self
            .vulnerable_squares()
            .overlap(attacked_squares)
            .is_empty();

        let is_occupied = !self
            .occupiable_squares()
            .overlap(board.all_occupied())
            .is_empty();

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
        CastleType::ALL
            .into_iter()
            .filter(|&ctype| self.is_available(ctype))
            .collect()
    }

    pub fn get_available_for(&self, side: Color) -> Vec<CastleType> {
        CastleType::ALL
            .into_iter()
            .filter(|ctype| ctype.color() == side)
            .filter(|ctype| self.is_available(*ctype))
            .collect()
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

////////////////////////////////////////////////////////////////////////////////
//
// Tests
//
////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(CastleType::WQ.vulnerable_squares().contains(C1));
        assert!(CastleType::WQ.vulnerable_squares().contains(D1));
        assert!(CastleType::WQ.vulnerable_squares().contains(E1));

        assert!(CastleType::WK.vulnerable_squares().contains(E1));
        assert!(CastleType::WK.vulnerable_squares().contains(F1));
        assert!(CastleType::WK.vulnerable_squares().contains(G1));

        assert!(CastleType::BQ.vulnerable_squares().contains(C8));
        assert!(CastleType::BQ.vulnerable_squares().contains(D8));
        assert!(CastleType::BQ.vulnerable_squares().contains(E8));

        assert!(CastleType::BK.vulnerable_squares().contains(E8));
        assert!(CastleType::BK.vulnerable_squares().contains(F8));
        assert!(CastleType::BK.vulnerable_squares().contains(G8));
    }

    // CastleType#occupiable_squares
    #[test]
    fn occupiable_squares() {
        assert!(CastleType::WQ.occupiable_squares().contains(B1));
        assert!(CastleType::WQ.occupiable_squares().contains(C1));
        assert!(CastleType::WQ.occupiable_squares().contains(D1));

        assert!(CastleType::WK.occupiable_squares().contains(F1));
        assert!(CastleType::WK.occupiable_squares().contains(G1));

        assert!(CastleType::BQ.occupiable_squares().contains(B8));
        assert!(CastleType::BQ.occupiable_squares().contains(C8));
        assert!(CastleType::BQ.occupiable_squares().contains(D8));

        assert!(CastleType::BK.occupiable_squares().contains(F8));
        assert!(CastleType::BK.occupiable_squares().contains(G8));
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
