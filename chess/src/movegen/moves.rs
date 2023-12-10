use crate::piece::PieceType;
use crate::piece::Piece;
use crate::piece::Color;
use crate::square::Square;
use itertools::Itertools;
use anyhow::anyhow;
use std::{fmt::Display, str::FromStr};
use MoveType::*;

/// Packs all the metadata related to a Move in a u16
///
/// 6 bits (0 - 63) for the source square
/// 6 bits (0 - 63) for the target square
/// 4 bits (0 - 16) for additional metadata (castling, captures, promotions)
#[derive(Default, Debug, Copy, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct Move(u16);

impl Move {
    pub const NULL: Move = Move(0);
    const SRC_MASK: u16  = 0b0000_0000_0011_1111;
    const TGT_MASK: u16  = 0b0000_1111_1100_0000;
    const TYPE_MASK: u16 = 0b1111_0000_0000_0000;

    /// Create a new Move from source, target, and movetype
    pub fn new(src: Square, tgt: Square, mtype: MoveType) -> Move {
        let mut value = 0u16;
        value |= src as u16;
        value |= (tgt as u16) << 6;
        value |= (mtype as u16) << 12;

        Move(value)
    }

    ///  Get the source square for a move
    pub fn src(self) -> Square {
        ((self.0 & Self::SRC_MASK) as usize).into()
    }

    /// Get the target square for a move
    pub fn tgt(self) -> Square {
        (((self.0 & Self::TGT_MASK) >> 6) as usize).into()
    }

    /// Get the move type for a move.
    pub fn get_type(self) -> MoveType {
        let idx = (self.0 & Self::TYPE_MASK) >> 12;
        MoveType::ALL[idx as usize]
    }

    /// Check whether the move is quiet (no capture, promotion, castle, etc...)
    pub fn is_quiet(self) -> bool {
        self.get_type() == MoveType::Quiet
    }

    /// Check whether the move is a castling move
    pub fn is_castle(self) -> bool {
        self.get_type() == MoveType::KingCastle 
        || self.get_type() == MoveType::QueenCastle
    }

    /// Check whether the move is a double push
    pub fn is_double_push(self) -> bool {
        self.get_type() == MoveType::DoublePush
    }

    /// Check whether the move is an en-passant capture
    pub fn is_en_passant(self) -> bool {
        self.get_type() == MoveType::EnPassant
    }

    /// Check whether the move is a promotion
    pub fn is_promotion(self) -> bool {
        self.0 & (1 << 15) != 0
    }

    /// Check whether the move is a capture
    pub fn is_capture(self) -> bool {
        self.0 & (1 << 14) != 0
    }

    /// Get the promotion type from a move
    pub fn get_promo_type(self) -> Option<PieceType> {
        use PieceType::*;

        match self.get_type() {
            KnightPromo | KnightPromoCapture => Some(Knight),
            BishopPromo | BishopPromoCapture => Some(Bishop),
            RookPromo | RookPromoCapture => Some(Rook),
            QueenPromo | QueenPromoCapture => Some(Queen),
            _ => None
        }
    }

    // Get the color associated with the promotion based on the rank the piece
    // is moving to
    //
    // This method doesn't check whether or not the move is actually a promotion
    // (i.e., if it's a pawn move), or that the MoveType is correctly set to
    // a promotion MoveType.
    pub fn get_promo_color(self) -> Option<Color> {
        if self.tgt().rank() == 7 {
            Some(Color::White)
        } else if self.tgt().rank() == 0 {
            Some(Color::Black)
        } else {
            None
        }
    }

    /// Return the algebraic character for the promotion (e.g., Q, N, b, r, ...)
    pub fn get_promo_label(self) -> Option<&'static str> {
        use PieceType::*;
        use Color::*;
        let ptype = self.get_promo_type()?;
        let color = self.get_promo_color()?;
        
        match (color, ptype) {
            (White, Knight) => Some("N"),
            (White, Bishop) => Some("B"),
            (White, Rook) => Some("R"),
            (White, Queen) => Some("Q"),
            (Black, Knight) => Some("n"),
            (Black, Bishop) => Some("b"),
            (Black, Rook) => Some("r"),
            (Black, Queen) => Some("q"),
            _ => None,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct BareMove {
    src: Square,
    tgt: Square,
    promo_type: Option<Piece>,
}

impl BareMove {
    pub fn new(src: Square, tgt: Square, promo_type: Option<Piece>) -> Self {
        Self {
            src,
            tgt,
            promo_type,
        }
    }

    pub fn src(&self) -> Square {
        self.src
    }

    pub fn tgt(&self) -> Square {
        self.tgt
    }
    
    pub fn promo_type(&self) -> Option<Piece> {
        self.promo_type
    }
}


/// Nibble-sized encoding of some metadata associated with a Move
#[rustfmt::skip]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u16)]
pub enum MoveType {
    Quiet                 = 0b0000,
    DoublePush            = 0b0001,
    KingCastle            = 0b0010,
    QueenCastle           = 0b0011,
    Capture               = 0b0100,
    EnPassant             = 0b0101,
    KnightPromo           = 0b1000,
    BishopPromo           = 0b1001,
    RookPromo             = 0b1010,
    QueenPromo            = 0b1011,
    KnightPromoCapture    = 0b1100,
    BishopPromoCapture    = 0b1101,
    RookPromoCapture      = 0b1110,
    QueenPromoCapture     = 0b1111,
}

impl MoveType {
    const ALL: [MoveType; 16] = [
        Quiet,
        DoublePush,
        KingCastle,
        QueenCastle,
        Capture,
        EnPassant,
        Quiet,
        Quiet,
        KnightPromo,
        BishopPromo,
        RookPromo,
        QueenPromo,
        KnightPromoCapture,
        BishopPromoCapture,
        RookPromoCapture,
        QueenPromoCapture,
    ];
}

////////////////////////////////////////////////////////////////////////////////
//
// Utility traits
//
////////////////////////////////////////////////////////////////////////////////

impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.src())?;
        write!(f, "{}", self.tgt())?;

        if self.is_promotion() {
            let label = self.get_promo_label().expect("The promotion has a label");
            write!(f, "{label}")?;
        }

        Ok(())
    }
}

impl FromStr for Move {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self> {
        let chunks = s.chars().chunks(2);

        // Collect chunks into 2char strings, or one char, if not enough are left
        // (i.e., promotion markers)
        let mut chunks = chunks
            .into_iter()
            .map(|chunk| chunk.collect::<String>());

        let sq1: Square = chunks.next()
            .ok_or(anyhow!("Not a valid move string"))?
            .parse()?;

        let sq2: Square = chunks.next()
            .ok_or(anyhow!("Not a valid move string"))?
            .parse()?;

        let mtype = match chunks.next() {
            Some(label) => label.parse()?,
            None => MoveType::Quiet
        };

        Ok(Move::new(sq1, sq2, mtype))
    }
}

impl FromStr for MoveType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self> {
        match s {
            "N" | "n" => Ok(KnightPromo),
            "B" | "b" => Ok(BishopPromo),
            "R" | "r" => Ok(RookPromo),
            "Q" | "q" => Ok(QueenPromo),
            _ => Err(anyhow!("Not a valid promotion label"))
        }
        
    }
}

impl PartialEq<BareMove> for Move {
    fn eq(&self, bare: &BareMove) -> bool {
        self.src() == bare.src()
            && self.tgt() == bare.tgt()
            && bare.promo_type().map(|piece| piece.piece_type()) == self.get_promo_type()
    }
}

impl FromStr for BareMove {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {

        // Split up the string into 2-character tuple, and potentially a trailing
        // character for the promo type
        let chunks = s.chars().chunks(2);

        // Collect the tuples back into 2-chracter strings
        let mut chunks = chunks
            .into_iter()
            .map(|chunk| chunk.collect::<String>());

        let src: Square = chunks.next()
            .ok_or(anyhow!("Not a valid move string"))?
            .parse()?;

        let tgt: Square = chunks.next()
            .ok_or(anyhow!("Not a valid move string"))?
            .parse()?;

        let promo_type = chunks.next().and_then(|label| {
            use Piece::*;

            match label.as_str() {
                "N"  => Some(WN),
                "B"  => Some(WB),
                "R"  => Some(WR),
                "Q"  => Some(WQ),
                "n" => Some(BN),
                "b" => Some(BB),
                "r" => Some(BR),
                "q" => Some(BQ),
                _ => None
            }
        });
        
        Ok(BareMove::new(src, tgt, promo_type))
    }
}

impl Display for BareMove {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.src())?;
        write!(f, "{}", self.tgt())?;

        if let Some(ptype) = self.promo_type() {
            write!(f, "{ptype}")?;
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
    fn src_works() {
        let src = Square::D5;
        let tgt = Square::E6;

        let mv = Move::new(src, tgt, MoveType::Quiet);
        assert_eq!(
            mv.src(),
            src.into(),
            "mv.src() should return the source position, as a bitboard"
        );
    }

    #[test]
    fn tgt_works() {
        let src = Square::D5;
        let tgt = Square::E6;

        let mv = Move::new(src, tgt, MoveType::Quiet);
        assert_eq!(
            mv.tgt(),
            tgt.into(),
            "mv.tgt() should return the source target, as a bitboard"
        );
    }

    #[test]
    fn bare_moves() {
        use Square::*;
        use Piece::*;
        use MoveType::*;

        // Parsing
        assert_eq!("a7a8Q".parse::<BareMove>().unwrap(), BareMove::new(A7, A8, Some(WQ)));
        assert_eq!("e7e8r".parse::<BareMove>().unwrap(), BareMove::new(E7, E8, Some(BR)));
        assert_eq!("e2e4".parse::<BareMove>().unwrap(), BareMove::new(E2, E4, None));

        // (Partial) Equality
        assert_eq!(Move::new(A7, A8, QueenPromo), "a7a8Q".parse::<BareMove>().unwrap());
        assert_eq!(Move::new(E7, E8, RookPromoCapture), "e7e8r".parse::<BareMove>().unwrap());

        // printing
        assert_eq!(BareMove::new(A7, A8, Some(WQ)).to_string(), "a7a8Q");
        assert_eq!(BareMove::new(A7, A8, Some(BR)).to_string(), "a7a8r");
        assert_eq!(BareMove::new(A7, A8, None).to_string(), "a7a8");
    }
}
