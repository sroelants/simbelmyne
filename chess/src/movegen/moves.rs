use crate::{
    bitboard::Step,
    board::{Color, Piece, PieceType},
    movegen::attack_boards::{
        ATTACK_RAYS, B_PAWN_ATTACKS, B_PAWN_DPUSHES, B_PAWN_PUSHES, KING_ATTACKS, KNIGHT_ATTACKS,
        W_PAWN_ATTACKS, W_PAWN_DPUSHES, W_PAWN_PUSHES,
    },
};
use std::{fmt::Display, str::FromStr};

use crate::bitboard::Bitboard;
use crate::board::Square;
use crate::movegen::attack_boards::Direction;
use itertools::Itertools;
use std::iter::successors;
use anyhow::anyhow;

use super::attack_boards::Rank;

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
        MoveType::Quiet,
        MoveType::DoublePush,
        MoveType::KingCastle,
        MoveType::QueenCastle,
        MoveType::Capture,
        MoveType::EnPassant,
        MoveType::Quiet,
        MoveType::Quiet,
        MoveType::KnightPromo,
        MoveType::BishopPromo,
        MoveType::RookPromo,
        MoveType::QueenPromo,
        MoveType::KnightPromoCapture,
        MoveType::BishopPromoCapture,
        MoveType::RookPromoCapture,
        MoveType::QueenPromoCapture,
    ];
}

impl FromStr for MoveType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self> {
        use MoveType::*;

        match s {
            "N" | "n" => Ok(KnightPromo),
            "B" | "b" => Ok(BishopPromo),
            "R" | "r" => Ok(RookPromo),
            "Q" | "q" => Ok(QueenPromo),
            _ => Err(anyhow!("Not a valid promotion label"))
        }
        
    }
}

/// Pack all the metadata related to a Move in a u16
///
/// 6 bits (0 - 63) for the source square
/// 6 bits (0 - 63) for the target square
/// 4 bits (0 - 16) for additional metadata (castling, captures, promotions)
/// When we get to move sorting, to we also want to squeeze in the sorting rank
/// here?
/// cf. Rustic https://github.com/mvanthoor/rustic/blob/17b15a34b68000dffb681277c3ef6fc98f935a0b/src/movegen/defs.rs
/// cf. Carp https://github.com/dede1751/carp/blob/main/chess/src/moves.rs
#[derive(Default, Debug, Copy, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct Move(u16);

impl Move {
    const SRC_MASK: u16  = 0b0000_0000_0011_1111;
    const TGT_MASK: u16  = 0b0000_1111_1100_0000;
    const TYPE_MASK: u16 = 0b1111_0000_0000_0000;
    pub const NULL: Move = Move(0);

    pub fn new(src: Square, tgt: Square, mtype: MoveType) -> Move {
        let mut value = 0u16;
        value |= src as u16;
        value |= (tgt as u16) << 6;
        value |= (mtype as u16) << 12;

        Move(value)
    }

    pub fn src(self) -> Square {
        ((self.0 & Self::SRC_MASK) as usize).into()
    }

    pub fn tgt(self) -> Square {
        (((self.0 & Self::TGT_MASK) >> 6) as usize).into()
    }

    pub fn get_type(self) -> MoveType {
        let idx = (self.0 & Self::TYPE_MASK) >> 12;
        MoveType::ALL[idx as usize]
    }

    pub fn is_castle(self) -> bool {
        self.get_type() == MoveType::KingCastle 
        || self.get_type() == MoveType::QueenCastle
    }

    pub fn is_double_push(self) -> bool {
        self.get_type() == MoveType::DoublePush
    }

    pub fn is_en_passant(self) -> bool {
        self.get_type() == MoveType::EnPassant
    }

    pub fn is_promotion(self) -> bool {
        self.0 & (1 << 15) != 0
    }

    pub fn is_capture(self) -> bool {
        self.0 & (1 << 14) != 0
    }

    pub fn get_promo_type(self) -> Option<PieceType> {
        use MoveType::*;
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
        let target: Bitboard = self.tgt().into();

        if (target & Rank::W_PROMO_RANK) != Bitboard::EMPTY {
            Some(Color::White)
        } else if (target & Rank::B_PROMO_RANK) != Bitboard::EMPTY {
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


impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.src().to_alg())?;
        write!(f, "{}", self.tgt().to_alg())?;

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


impl Piece {
    pub fn range(&self) -> usize {
        use PieceType::*;

        match self.piece_type() {
            Pawn => {
                let hasnt_moved = self.position.on_pawn_rank(self.color());
                if hasnt_moved {
                    2
                } else {
                    1
                }
            }
            Knight | King => 1,
            _ => 7, // The entire board
        }
    }

    pub fn directions(&self) -> &[Step] {
        use PieceType::*;

        match self.piece_type() {
            Pawn => &Step::PAWN_DIRS[self.color() as usize],

            Rook => &Step::ORTHO_DIRS,

            Knight => &Step::KNIGHT_DIRS,

            Bishop => &Step::DIAG_DIRS,

            King | Queen => &Step::ALL_DIRS,
        }
    }

    /// All the squares that are _visible_ to the piece.
    /// This means all unoccupied squares in the pieces main directions, up
    /// until (and including), the first blocker piece.
    ///
    /// This blocker can be either friendly or enemy, so we need to mask out
    /// friendly pieces if we're interested in attacks
    pub fn visible_squares(&self, ours: Bitboard, theirs: Bitboard) -> Bitboard {
        use PieceType::*;
        let sq = Square::from(self.position);
        let blockers = ours | theirs;

        match self.piece_type() {
            Bishop => {
                let mut visible = Bitboard::EMPTY;
                for dir in Direction::BISHOP {
                    visible |= visible_ray(dir, sq, blockers);
                }
                visible
            }

            Rook => {
                let mut visible = Bitboard::EMPTY;
                for dir in Direction::ROOK {
                    visible |= visible_ray(dir, sq, blockers);
                }
                visible
            }

            Queen => {
                let mut visible = Bitboard::EMPTY;
                for dir in Direction::ALL {
                    visible |= visible_ray(dir, sq, blockers);
                }
                visible
            }

            Knight => KNIGHT_ATTACKS[Square::from(self.position) as usize],

            King => KING_ATTACKS[Square::from(self.position) as usize],

            Pawn => {
                let square = Square::from(self.position);
                let mut visible = Bitboard::EMPTY;
                let on_original_rank = self.position.on_pawn_rank(self.color());

                if self.color().is_white() {
                    visible |= theirs & W_PAWN_ATTACKS[square as usize];
                    let single_push = W_PAWN_PUSHES[square as usize] & !blockers;
                    visible |= single_push;

                    if on_original_rank && single_push != Bitboard::EMPTY {
                        visible |= W_PAWN_DPUSHES[square as usize] & !blockers;
                    } 
                } else {
                    visible |= theirs & B_PAWN_ATTACKS[square as usize];

                    let single_push = B_PAWN_PUSHES[square as usize] & !blockers;
                    visible |= single_push;

                    if on_original_rank && single_push != Bitboard::EMPTY {
                        visible |= B_PAWN_DPUSHES[square as usize] & !blockers;
                    }
                }

                visible
            }
        }
    }

    pub fn visible_rays(&self, blockers: Bitboard) -> Vec<Bitboard> {
        self.directions()
            .into_iter()
            .map(|step| {
                successors(self.position.offset(*step), |pos| pos.offset(*step))
                    .take(self.range())
                    .take_while_inclusive(|&pos| !blockers.contains(pos))
                    .collect()
            })
            .collect()
    }
}

/// Given a direction, return the ray of squares starting at (and excluding)
/// `square`, up till (and including) the first blocker in the `blockers`
/// bitboard.
pub fn visible_ray(dir: Direction, square: Square, blockers: Bitboard) -> Bitboard {
    let ray = ATTACK_RAYS[dir as usize][square as usize];
    let mut visible = ray;

    if let Some(blocker) = ray_blocker(dir, square, blockers) {
        visible &= !ATTACK_RAYS[dir as usize][blocker as usize];
    }

    visible
}

fn ray_blocker(dir: Direction, square: Square, blockers: Bitboard) -> Option<Square> {
    let ray = ATTACK_RAYS[dir as usize][square as usize];

    let on_ray_bb = blockers & ray;

    //TODO: Clean this up?
    let blocker = if dir.is_positive() {
        let lsb = on_ray_bb.trailing_zeros() as usize;
        Square::try_from_usize(lsb)
    } else {
        let lsb = (on_ray_bb.leading_zeros() + 1) as usize;
        64usize.checked_sub(lsb).and_then(Square::try_from_usize)
    };

    blocker.map(|sq| Square::from(sq))
}

pub fn visible_squares(
    square: Square,
    piece_type: PieceType,
    color: Color,
    ours: Bitboard,
    theirs: Bitboard,
) -> Bitboard {
    use PieceType::*;
    let blockers = ours | theirs;

    match piece_type {
        Bishop => {
            let mut visible = Bitboard::EMPTY;
            for dir in Direction::BISHOP {
                visible |= visible_ray(dir, square, blockers);
            }
            visible
        }

        Rook => {
            let mut visible = Bitboard::EMPTY;
            for dir in Direction::ROOK {
                visible |= visible_ray(dir, square, blockers);
            }
            visible
        }

        Queen => {
            let mut visible = Bitboard::EMPTY;
            for dir in Direction::ALL {
                visible |= visible_ray(dir, square, blockers);
            }
            visible
        }

        Knight => KNIGHT_ATTACKS[square as usize],

        King => KING_ATTACKS[square as usize],

        Pawn => {
            let mut visible = Bitboard::EMPTY;
            let sq_bb = Bitboard::from(square);

            if color.is_white() {
                visible |= theirs & W_PAWN_ATTACKS[square as usize];

                if sq_bb.on_pawn_rank(color) {
                    visible |= W_PAWN_DPUSHES[square as usize] & !theirs;
                } else {
                    visible |= W_PAWN_PUSHES[square as usize] & !theirs;
                }
            } else {
                visible |= theirs & B_PAWN_ATTACKS[square as usize];

                if sq_bb.on_pawn_rank(color) {
                    visible |= B_PAWN_DPUSHES[square as usize] & !theirs;
                } else {
                    visible |= B_PAWN_PUSHES[square as usize] & !theirs;
                }
            }

            visible
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ray_blocker() {
        let dir = Direction::Up;
        let square = Square::new(3, 3); // d4

        let blocker = Square::new(6, 3); // d7
        let blockers = Bitboard(0xaa98605591844602); // A bunch of crap

        let result = ray_blocker(dir, square, blockers);

        assert!(result.is_some());
        assert_eq!(result.unwrap(), blocker);
    }

    #[test]
    fn src_works() {
        let src = Square::new(3, 4);
        let tgt = Square::new(4, 5);

        let mv = Move::new(src, tgt, MoveType::Quiet);
        assert_eq!(
            mv.src(),
            src.into(),
            "mv.src() should return the source position, as a bitboard"
        );
    }

    #[test]
    fn tgt_works() {
        let src = Square::new(3, 4);
        let tgt = Square::new(4, 5);

        let mv = Move::new(src, tgt, MoveType::Quiet);
        assert_eq!(
            mv.tgt(),
            tgt.into(),
            "mv.tgt() should return the source target, as a bitboard"
        );
    }
}
