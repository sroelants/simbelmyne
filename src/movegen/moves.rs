use std::fmt::Display;

use crate::bitboard::Bitboard;

/// Pack all the metadata related to a Move in a u16
///
/// 6 bits (0 - 63) for the source square
/// 6 bits (0 - 63) for the target square
/// 4 bits (0 - 16) for additional metadata (castling, captures, promotions)
/// When we get to move sorting, to we also want to squeeze in the sorting rank
/// here? 
/// cf. Rustic https://github.com/mvanthoor/rustic/blob/17b15a34b68000dffb681277c3ef6fc98f935a0b/src/movegen/defs.rs
/// cf. Carp https://github.com/dede1751/carp/blob/main/chess/src/moves.rs
#[derive(Default, Debug, Copy, Clone)]
pub struct Move(u16);

impl Move {
    pub const SRC_MASK: u16        = 0b0000_000000_111111;
    pub const TGT_MASK: u16        = 0b0000_111111_000000;
    pub const CASTLE_MASK: u16     = 0b0001_000000_000000;
    pub const CAPTURE_MASK: u16    = 0b0010_000000_000000;

    pub fn new(src: Bitboard, tgt: Bitboard) -> Move {
        let mut value = 0u16;
        value |= src.trailing_zeros() as u16;
        value |= (tgt.trailing_zeros() as u16) << 6;

        Move(value)
    }

    pub fn src(self) -> Bitboard {
        Bitboard(1u64 << (self.0 & Self::SRC_MASK))
    }

    pub fn tgt(self) -> Bitboard {
        Bitboard(1u64 << ((self.0 & Self::TGT_MASK) >> 6))
    }

    pub fn is_castle(self) -> bool {
        self.0 & Self::CASTLE_MASK != 0
    }

    pub fn set_castle(&mut self) {
        self.0 |= Self::CASTLE_MASK;
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.src().to_alg())?;
        write!(f, "{}", self.tgt().to_alg())?;

        if self.is_castle() {
            write!(f, " (Castle)")?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn src_works() {
        let src = Bitboard::new(3,4);
        let tgt = Bitboard::new(4,5);

        let mv = Move::new(src,tgt);
        assert_eq!(mv.src(), src, "mv.src() should return the source position, as a bitboard");
    }

    fn tgt_works() {
        let src = Bitboard::new(3,4);
        let tgt = Bitboard::new(4,5);

        let mv = Move::new(src,tgt);
        assert_eq!(mv.tgt(), tgt, "mv.tgt() should return the source target, as a bitboard");
    }

    fn castling_bit() {
        let src = Bitboard::new(3,4);
        let tgt = Bitboard::new(4,5);

        let mut mv = Move::new(src,tgt);
        assert!(!mv.is_castle(), "is_castle returns false for a normal move");

        mv.set_castle();
        assert!(mv.is_castle(), "is_castle returns true after setting the castle bit");
    }
}
