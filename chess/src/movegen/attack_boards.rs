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

use crate::{bitboard::Bitboard, piece::Color, constants::FILES};
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

/// Look up a bitboard of all squares in a given direction, from the requested
/// square
pub const RAYS: [BBTable; 8] = [
    UP_RAYS,
    DOWN_RAYS,
    LEFT_RAYS,
    RIGHT_RAYS,
    UP_LEFT_RAYS,
    UP_RIGHT_RAYS,
    DOWN_LEFT_RAYS,
    DOWN_RIGHT_RAYS,
];

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
pub const BISHOP_ATTACKS: BBTable = gen_bishop_attacks();
pub const ROOK_ATTACKS: BBTable = gen_rook_attacks();
pub const QUEEN_ATTACKS: BBTable = gen_queen_attacks();
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
// Generate rays
//
////////////////////////////////////////////////////////////////////////////////

const UP_RAYS: BBTable = gen_up_rays();
const DOWN_RAYS: BBTable = gen_down_rays();
const LEFT_RAYS: BBTable = gen_left_rays();
const RIGHT_RAYS: BBTable = gen_right_rays();
const UP_RIGHT_RAYS: BBTable = gen_up_right_rays();
const UP_LEFT_RAYS: BBTable = gen_up_left_rays();
const DOWN_RIGHT_RAYS: BBTable = gen_down_right_rays();
const DOWN_LEFT_RAYS: BBTable = gen_down_left_rays();

// Compute a table, indexed by a Square, that stores the upward-facing ray 
// starting at that square.
const fn gen_up_rays() -> BBTable {
    let mut bbs: BBTable = [Bitboard(0); 64];
    let mut square: usize = 0;

    while square < 64 {
        let rank = square / 8;

        if rank < 7 {
            bbs[square] = Bitboard(FILES[0].0 << square + 8);
        }

        square += 1;
    }

    bbs
}

// Compute a table, indexed by a Square, that stores the downward-facing ray 
// starting at that square.
const fn gen_down_rays() -> BBTable {
    let mut bbs: BBTable = [Bitboard(0); 64];
    let mut square: usize = 0;

    while square < 64 {
        let rank = square / 8;

        if rank > 0 {
            bbs[square] = Bitboard(FILES[7].0 >> (63 - (square - 8)));
        }

        square += 1;
    }

    bbs
}

// Compute a table, indexed by a Square, that stores the leftward-facing ray 
// starting at that square.
const fn gen_left_rays() -> BBTable {
    let mut bbs: BBTable = [Bitboard(0); 64];
    let mut square: usize = 0;

    while square < 64 {
        let mut curr = square;
        let mut bb: u64 = 0;

        while curr % 8 > 0 {
            curr -= 1;
            bb |= 1 << curr;
        }

        bbs[square] = Bitboard(bb as u64);
        square += 1;
    }

    bbs
}

// Compute a table, indexed by a Square, that stores the rightward-facing ray 
// starting at that square.
const fn gen_right_rays() -> BBTable {
    let mut bbs: BBTable = [Bitboard(0); 64];
    let mut square: usize = 0;

    while square < 64 {
        let mut bb: u64 = 0;
        let mut curr = square;

        while curr % 8 < 7 {
            curr += 1;
            bb |= 1 << curr;
        }

        bbs[square] = Bitboard(bb as u64);
        square += 1;
    }

    bbs
}

// Compute a table, indexed by a Square, that stores the up-and-rightward-facing 
// ray starting at that square.
const fn gen_up_right_rays() -> BBTable {
    let mut bbs: BBTable = [Bitboard(0); 64];
    let mut square: usize = 0;

    while square < 64 {
        let mut curr = square;
        let mut bb: u64 = 0;

        while curr % 8 < 7 && curr / 8 < 7 {
            curr += 9;
            bb |= 1 << curr;
        }

        bbs[square] = Bitboard(bb as u64);
        square += 1;
    }

    bbs
}

// Compute a table, indexed by a Square, that stores the up-and-leftward-facing 
// ray starting at that square.
const fn gen_up_left_rays() -> BBTable {
    let mut bbs: BBTable = [Bitboard(0); 64];
    let mut square: usize = 0;

    while square < 64 {
        let mut curr = square;
        let mut bb: u64 = 0;

        while curr % 8 > 0 && curr / 8 < 7 {
            curr += 7;
            bb |= 1 << curr;
        }

        bbs[square] = Bitboard(bb as u64);
        square += 1;
    }

    bbs
}

// Compute a table, indexed by a Square, that stores the down-and-rightward 
// facing ray starting at that square.
const fn gen_down_right_rays() -> BBTable {
    let mut bbs: BBTable = [Bitboard(0); 64];
    let mut square: usize = 0;

    while square < 64 {
        let mut curr = square;
        let mut bb: u64 = 0;

        while curr % 8 < 7 && curr > 7 {
            curr -= 7;
            bb |= 1 << curr;
        }

        bbs[square] = Bitboard(bb as u64);
        square += 1;
    }

    bbs
}

// Compute a table, indexed by a Square, that stores the down-and-leftward 
// facing ray starting at that square.
const fn gen_down_left_rays() -> BBTable {
    let mut bbs: BBTable = [Bitboard(0); 64];
    let mut square: usize = 0;

    while square < 64 {
        let mut curr = square;
        let mut bb: u64 = 0;

        while curr % 8 > 0 && curr > 7 {
            curr -= 9;
            bb |= 1 << curr;
        }

        bbs[square] = Bitboard(bb as u64);
        square += 1;
    }

    bbs
}

////////////////////////////////////////////////////////////////////////////////
//
// Generate attack boards
//
////////////////////////////////////////////////////////////////////////////////

/// Generate bishop attack squares from a given square
const fn gen_bishop_attacks() -> BBTable {
    let mut bbs: BBTable = [Bitboard(0); 64];
    let mut square: usize = 0;

    while square < 64 {
        bbs[square] = Bitboard(
            UP_LEFT_RAYS[square].0
                | UP_RIGHT_RAYS[square].0
                | DOWN_LEFT_RAYS[square].0
                | DOWN_RIGHT_RAYS[square].0,
        );

        square += 1;
    }

    bbs
}

/// Generate rook attack squares from a given square
const fn gen_rook_attacks() -> BBTable {
    let mut bbs: BBTable = [Bitboard(0); 64];
    let mut square: usize = 0;

    while square < 64 {
        bbs[square] = Bitboard(
            UP_RAYS[square].0 | DOWN_RAYS[square].0 | LEFT_RAYS[square].0 | RIGHT_RAYS[square].0,
        );

        square += 1;
    }

    bbs
}

/// Generate queen attack squares from a given square
const fn gen_queen_attacks() -> BBTable {
    let mut bbs: BBTable = [Bitboard(0); 64];
    let mut square: usize = 0;

    while square < 64 {
        bbs[square] = Bitboard(BISHOP_ATTACKS[square].0 | ROOK_ATTACKS[square].0);

        square += 1;
    }

    bbs
}

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
    fn test_up_ray_a1() {
        assert_eq!(
            UP_RAYS[A1 as usize],
            Bitboard(0x101010101010100),
            "Gets the a file for A1"
        );
    }

    #[test]
    fn test_up_ray_a3() {
        assert_eq!(
            UP_RAYS[A3 as usize],
            Bitboard(0x101010101000000),
            "Gets the correct up-ray for A3"
        );
    }

    #[test]
    fn test_up_ray_c3() {
        assert_eq!(
            UP_RAYS[C3 as usize],
            Bitboard(0x404040404000000),
            "Gets the correct up-ray for C3"
        );
    }

    #[test]
    fn test_left_ray_c3() {
        assert_eq!(
            LEFT_RAYS[C3 as usize],
            Bitboard(0x30000),
            "Gets the correct left-ray for C3"
        );
    }

    #[test]
    fn test_right_ray_d4() {
        assert_eq!(
            RIGHT_RAYS[D4 as usize],
            Bitboard(0xf0000000),
            "Gets the correct right-ray for D4"
        );
    }

    #[test]
    fn test_up_right_ray_d4() {
        assert_eq!(
            UP_RIGHT_RAYS[D4 as usize],
            Bitboard(0x8040201000000000),
            "Gets the correct up-right-ray for D4"
        );
    }

    #[test]
    fn test_up_left_ray_d4() {
        assert_eq!(
            UP_LEFT_RAYS[D4 as usize],
            Bitboard(0x1020400000000),
            "Gets the correct up-left-ray for D4"
        );
    }

    #[test]
    fn test_down_right_ray_d4() {
        assert_eq!(
            DOWN_RIGHT_RAYS[D4 as usize],
            Bitboard(0x102040),
            "Gets the correct down-right-ray for D4"
        );
    }

    #[test]
    fn test_down_left_ray_d4() {
        assert_eq!(
            DOWN_LEFT_RAYS[D4 as usize],
            Bitboard(0x40201),
            "Gets the correct down-left-ray for D4"
        );
    }

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
    fn test_bishop_attacks() {
        assert_eq!(BISHOP_ATTACKS[E5 as usize], Bitboard(0x8244280028448201));
    }

    #[test]
    fn test_rook_attacks() {
        assert_eq!(ROOK_ATTACKS[E5 as usize], Bitboard(0x101010ef10101010));
    }

    #[test]
    fn test_queen_attacks() {
        assert_eq!(QUEEN_ATTACKS[E5 as usize], Bitboard(0x925438ef38549211));
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
