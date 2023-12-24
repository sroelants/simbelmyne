//! Precompute any and all lookup data structures we can at compile time
//!
//! Doing this stuff at compile time means we can just bake the tables into 
//! the binary. This means a bigger binary, but less work we need to do on the
//! user's end when they fire up the engine. I don't think anyone cares enough 
//! about their binary being 256kb larger.
//!
//! The downside is that the logic in this file is restricted to what Rust 
//! allows inside `const expressions`. That is to say, whatever the Rust 
//! compiler knows how to execute (either for technical, efficiency, or 
//! safety reasons, I suppose). This means the logic here can be kinda hairy,
//! but also more straightforward than the rest of the code base. 
//!
//! Best just not to look at it.
//!
//! You have been warned, avert your eyes! 🙈

use crate::piece::Color;
use crate::magics::BISHOP_MAGICS;
use crate::magics::ROOK_MAGICS;
use crate::magics::rook_attacks;
use crate::magics::bishop_attacks;
use crate::bitboard::Bitboard;
use crate::square::Square;
use Direction::*;

// For internal use as more readable const parameters
const WHITE: bool = true;
const BLACK: bool = false;
const DOUBLE_PUSH: bool = true;
const SINGLE_PUSH: bool = false;

type BBTable = [Bitboard; 64];
type BBBTable = [[Bitboard; 64]; 64];

/// Helper enum to hulp us index into collections of bitboards more easily
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Direction { U, D, L, R, UL, UR, DL, DR }

impl Direction {
    pub const ALL: [Direction; 8] = [U, D, L, R, UL, UR, DL, DR];
    pub const DIAGS: [Direction; 4] = [UL, UR, DL, DR];
    pub const HVS: [Direction; 4] = [U, D, L, R];

    pub fn is_positive(&self) -> bool {
        match self {
            UL | U | UR | R => true,
            _ => false,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
//
// Line of sight tables
//
////////////////////////////////////////////////////////////////////////////////

/// Look up the Bitboard of all squares between two squares, excluding the
/// endpoints.
pub const BETWEEN: BBBTable = gen_between();

////////////////////////////////////////////////////////////////////////////////
//
// Piece moves
//
////////////////////////////////////////////////////////////////////////////////

pub const PAWN_PUSHES: [BBTable; Color::COUNT] = [
    gen_pawn_pushes::<WHITE, SINGLE_PUSH>(),
    gen_pawn_pushes::<BLACK, SINGLE_PUSH>(),
];

pub const PAWN_DBLPUSHES: [BBTable; Color::COUNT] = [
    gen_pawn_pushes::<WHITE, DOUBLE_PUSH>(),
    gen_pawn_pushes::<BLACK, DOUBLE_PUSH>(),
];

pub const PAWN_ATTACKS: [BBTable; Color::COUNT] = [
    gen_pawn_attacks::<WHITE>(),
    gen_pawn_attacks::<BLACK>(),
];

pub const KNIGHT_ATTACKS: BBTable = gen_knight_attacks();
pub const BISHOP_ATTACKS: [Bitboard; 5248] = gen_bishop_attacks();
pub const ROOK_ATTACKS: [Bitboard; 102400] = gen_rook_attacks();
pub const KING_ATTACKS: BBTable = gen_king_attacks();


////////////////////////////////////////////////////////////////////////////////
//
// Generate Between table
//
////////////////////////////////////////////////////////////////////////////////

const fn gen_between() -> BBBTable {
    let mut between = [[Bitboard::EMPTY; 64]; 64];
    let mut sq1: usize = 0;

    while sq1 < 64 {
        let mut sq2 = 0;

        while sq2 < 64 {
            between[sq1][sq2] = bb_between(sq1, sq2);
            sq2 += 1;
        }

        sq1 += 1;
    }

    between
}

const fn bb_between(sq1: usize, sq2: usize) -> Bitboard {
    let mut bb: u64 = 0;
    let mut x1 = sq1 % 8;
    let mut y1 = sq1 / 8;
    let mut x2 = sq2 % 8;
    let mut y2 = sq2 / 8;

    // Horizontal
    if x1 == x2 && y1 + 1 < y2 {
        while y1 + 1 < y2 {
            y1 += 1;
            bb |= 1 << ( x1 + 8 * y1 )
        }
    } else if x1 == x2 && y2 + 1 < y1 {
        while y2 + 1 < y1 {
            y2 += 1;
            bb |= 1 << ( x2 + 8 * y2 )
        }
    } else if x1 + 1 < x2 && y1 == y2 {
        while x1 + 1 < x2 {
            x1 += 1;
            bb |= 1 << ( x1 + 8 * y1 )
        }
    } else if x2 + 1 < x1 && y1 == y2 {
        while x2 + 1 < x1 {
            x2 += 1;
            bb |= 1 << ( x2 + 8 * y2 )
        }
    } 

    // Diagonal 
    else if x1 + 1 < x2 && y1 + 1 < y2 && x2 - x1 == y2 - y1 {
        while x1 + 1 < x2 && y1 + 1 < y2 {
            x1 += 1;
            y1 += 1;
            bb |= 1 << (x1 + 8 * y1);
        }
    } else if x2 + 1 < x1 && y2 + 1 < y1 && x1 - x2 == y1 - y2 {
        while x2 < x1 - 1 && y2 < y1 - 1 {
            x2 += 1;
            y2 += 1;
            bb |= 1 << (x2 + 8 * y2);
        }
    } else if x1 + 1 < x2 && y2 + 1 < y1 && x2 - x1 == y1 - y2 {
        while x1 + 1 < x2 && y2 + 1 < y1 {
            x1 += 1;
            y1 -= 1;
            bb |= 1 << (x1 + 8 * y1);
        }
    } else if x2 + 1 < x1 && y1 + 1 < y2 && x1 - x2 == y2 - y1 {
        while x2 + 1 < x1 && y1 + 1 < y2 {
            x1 -= 1;
            y1 += 1;
            bb |= 1 << (x1 + 8 * y1);
        }
    }

    Bitboard(bb)
}

////////////////////////////////////////////////////////////////////////////////
//
// Generate attack boards
//
////////////////////////////////////////////////////////////////////////////////


/// Generate pawn push squares from a given square
const fn gen_pawn_pushes<const WHITE: bool, const DOUBLE_PUSH: bool>() -> BBTable {
    let mut bbs: BBTable = [Bitboard(0); 64];
    let mut square: usize = 0;

    while square < 64 {
        let rank = square / 8;
        let mut bitboard: u64 = 0;

        if WHITE {
            if rank < 7 {
                let up = square + 8;
                bitboard |= 1 << up
            }
            if DOUBLE_PUSH && rank < 6 {
                let upup = square + 16;
                bitboard |= 1 << upup
            }
        } else {
            if rank > 0 {
                let down = square - 8;
                bitboard |= 1 << down
            }

            if DOUBLE_PUSH && rank > 1 {
                let downdown = square - 16;
                bitboard |= 1 << downdown
            }
        }

        bbs[square] = Bitboard(bitboard);
        square += 1
    }

    bbs
}

/// Generate pawn attack squares from a given square
const fn gen_pawn_attacks<const WHITE: bool>() -> BBTable {
    let mut bbs: BBTable = [Bitboard(0); 64];
    let mut square: usize = 0;

    while square < 64 {
        let file = square % 8;
        let rank = square / 8;
        let mut bitboard: u64 = 0;

        if WHITE {
            if file > 0 && rank < 7 {
                let up_left = square + 7;
                bitboard |= 1 << up_left
            }
            if file < 7 && rank < 7 {
                let up_right = square + 9;
                bitboard |= 1 << up_right;
            }
        } else {
            // BLACK
            if file > 0 && rank > 0 {
                let down_left = square - 9;
                bitboard |= 1 << down_left;
            }
            if file < 7 && rank > 0 {
                let down_right = square - 7;
                bitboard |= 1 << down_right
            }
        }
        bbs[square] = Bitboard(bitboard);
        square += 1
    }

    bbs
}

/// Generate king attack squares from a given square
const fn gen_king_attacks() -> BBTable {
    let mut bbs: BBTable = [Bitboard(0); 64];
    let mut square: usize = 0;

    while square < 64 {
        let file = square % 8;
        let rank = square / 8;
        let mut bitboard: u64 = 0;

        if file > 0 {
            let left = square - 1;
            bitboard |= 1 << left;

            if rank > 0 {
                let downleft = square - 9;
                bitboard |= 1 << downleft;
            }

            if rank < 7 {
                let upleft = square + 7;
                bitboard |= 1 << upleft;
            }
        }

        if file < 7 {
            let right = square + 1;
            bitboard |= 1 << right;

            if rank < 7 {
                let upright = square + 9;
                bitboard |= 1 << upright;
            }

            if rank > 0 {
                let downright = square - 7;
                bitboard |= 1 << downright;
            }
        }

        if rank > 0 {
            let down = square - 8;
            bitboard |= 1 << down;
        }

        if rank < 7 {
            let up = square + 8;
            bitboard |= 1 << up;
        }

        bbs[square] = Bitboard(bitboard);
        square += 1
    }

    bbs
}

/// Generate knight attack squares from a given square
const fn gen_knight_attacks() -> BBTable {
    let mut bbs: BBTable = [Bitboard(0); 64];
    let mut square: usize = 0;

    while square < 64 {
        let file = square % 8;
        let rank = square / 8;
        let mut bitboard: u64 = 0;

        if file > 1 && rank < 7 {
            let leftleftup = square + 6;
            bitboard |= 1 << leftleftup;
        }

        if file > 0 && rank < 6 {
            let upupleft = square + 15;
            bitboard |= 1 << upupleft;
        }

        if file > 1 && rank > 0 {
            let leftleftdown = square - 10;
            bitboard |= 1 << leftleftdown;
        }

        if file > 0 && rank > 1 {
            let downdownleft = square - 17;
            bitboard |= 1 << downdownleft;
        }

        if file < 6 && rank < 7 {
            let rightrightup = square + 10;
            bitboard |= 1 << rightrightup;
        }

        if file < 7 && rank < 6 {
            let upupright = square + 17;
            bitboard |= 1 << upupright;
        }

        if file < 6 && rank > 0 {
            let rightrightdown = square - 6;
            bitboard |= 1 << rightrightdown;
        }

        if file < 7 && rank > 1 {
            let downdownright = square - 15;
            bitboard |= 1 << downdownright;
        }

        bbs[square] = Bitboard(bitboard);
        square += 1
    }

    bbs
}

const fn gen_bishop_attacks() -> [Bitboard; 5248]  {
    let mut table = [Bitboard::EMPTY; 5248];
    let mut sq: usize = 0;

    while sq < 64 {
        let entry = BISHOP_MAGICS[sq];
        let mut subset: u64 = 0;

        // First treat the empty subset 
        let attacks = bishop_attacks(Square::ALL[sq], Bitboard(subset));
        let blockers = Bitboard(subset);
        let idx = entry.index(blockers);
        table[idx] = attacks;
        subset = subset.wrapping_sub(entry.mask.0) & entry.mask.0;

        // For every subset of possible blockers, get the attacked squares and
        // store them in the table.
        while subset != 0 {
            let attacks = bishop_attacks(Square::ALL[sq], Bitboard(subset));
            let blockers = Bitboard(subset);
            let idx = entry.index(blockers);
            table[idx] = attacks;

            subset = subset.wrapping_sub(entry.mask.0) & entry.mask.0;
        }

        sq += 1;
    }

    table
}


const fn gen_rook_attacks() -> [Bitboard; 102400] {
    let mut table = [Bitboard::EMPTY; 102400];
    let mut sq: usize = 0;

    while sq < 64 {
        let entry = ROOK_MAGICS[sq];
        let mut subset: u64 = 0;

        // First treat the empty subset 
        let attacks = rook_attacks(Square::ALL[sq], Bitboard(subset));
        let blockers = Bitboard(subset);
        let idx = entry.index(blockers);
        table[idx] = attacks;
        subset = subset.wrapping_sub(entry.mask.0) & entry.mask.0;

        // For every subset of possible blockers, get the attacked squares and
        // store them in the table.
        while subset != 0 {
            let attacks = rook_attacks(Square::ALL[sq], Bitboard(subset));
            let blockers = Bitboard(subset);
            let idx = entry.index(blockers);
            table[idx] = attacks;

            subset = subset.wrapping_sub(entry.mask.0) & entry.mask.0;
        }

        sq += 1;
    }

    table
}

////////////////////////////////////////////////////////////////////////////////
//
// Tests
//
////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use crate::square::Square::*;

    use super::*;

    #[test]
    fn test_pawn_pushes() {
        use Color::*;

        assert_eq!(PAWN_PUSHES[White as usize][E5 as usize], Bitboard(0x100000000000));
        assert_eq!(PAWN_PUSHES[White as usize][E8 as usize], Bitboard(0x000000000000));
        assert_eq!(PAWN_PUSHES[Black as usize][E5 as usize], Bitboard(0x10000000));
        assert_eq!(PAWN_PUSHES[Black as usize][E1 as usize], Bitboard(0x000000000000));

        // Double pushes
        assert_eq!(PAWN_DBLPUSHES[White as usize][E5 as usize], Bitboard(0x10100000000000));
        assert_eq!(PAWN_DBLPUSHES[White as usize][E7 as usize], Bitboard(0x1000000000000000));
        assert_eq!(PAWN_DBLPUSHES[Black as usize][E5 as usize], Bitboard(0x10100000));
        assert_eq!(PAWN_DBLPUSHES[Black as usize][E2 as usize], Bitboard(0x10));
    }

    #[test]
    fn test_pawn_attacks() {
        use Color::*;

        assert_eq!(PAWN_ATTACKS[White as usize][E5 as usize], Bitboard(0x280000000000));
        assert_eq!(PAWN_ATTACKS[White as usize][A5 as usize], Bitboard(0x20000000000));
        assert_eq!(PAWN_ATTACKS[White as usize][H5 as usize], Bitboard(0x400000000000));
        assert_eq!(PAWN_ATTACKS[White as usize][E8 as usize], Bitboard(0x00));

        assert_eq!(PAWN_ATTACKS[Black as usize][E5 as usize], Bitboard(0x28000000));
        assert_eq!(PAWN_ATTACKS[Black as usize][A5 as usize], Bitboard(0x2000000));
        assert_eq!(PAWN_ATTACKS[Black as usize][H5 as usize], Bitboard(0x40000000));
        assert_eq!(PAWN_ATTACKS[Black as usize][E1 as usize], Bitboard(0x00));
    }

    #[test]
    fn test_knight_attacks() {
        assert_eq!(KNIGHT_ATTACKS[E5 as usize], Bitboard(0x28440044280000));
        assert_eq!(KNIGHT_ATTACKS[B7 as usize], Bitboard(0x800080500000000));
        assert_eq!(KNIGHT_ATTACKS[G2 as usize], Bitboard(0xa0100010));
    }

    #[test]
    fn test_king_attacks() {
        println!("{}",Bitboard(0x203000000000000));
        assert_eq!(KING_ATTACKS[E5 as usize], Bitboard(0x382838000000));
        assert_eq!(KING_ATTACKS[A8 as usize], Bitboard(0x203000000000000));
    }

    #[test]
    fn test_between() {
        assert!( BETWEEN[A1 as usize][A8 as usize].contains(A2.into()));
        assert!( BETWEEN[A1 as usize][A8 as usize].contains(A3.into()));
        assert!( BETWEEN[A1 as usize][A8 as usize].contains(A4.into()));
        assert!(!BETWEEN[A1 as usize][A8 as usize].contains(B4.into()));

        assert!( BETWEEN[A1 as usize][C3 as usize].contains(B2.into()));
        assert!( BETWEEN[G2 as usize][E4 as usize].contains(F3.into()));
    }
}
