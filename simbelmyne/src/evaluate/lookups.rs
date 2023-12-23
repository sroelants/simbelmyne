use chess::{bitboard::Bitboard, piece::Color, square::Square};

use super::Eval;

type BBTable = [Bitboard; Square::COUNT];

////////////////////////////////////////////////////////////////////////////////
//
// Lookup tables
//
////////////////////////////////////////////////////////////////////////////////

pub const PASSED_PAWN_MASKS: [BBTable; Color::COUNT] = gen_passed_pawn_masks();

pub const ISOLATED_PAWN_MASKS: BBTable = gen_isolated_pawn_masks();

pub const DOUBLED_PAWN_MASKS: [Bitboard; 8] = gen_doubled_pawn_masks();

////////////////////////////////////////////////////////////////////////////////
//
// Passed pawn masks
//
////////////////////////////////////////////////////////////////////////////////

const fn gen_passed_pawn_masks() -> [BBTable; Color::COUNT] {
    const A_FILE: Bitboard = Bitboard(0x0101010101010101);
    const H_FILE: Bitboard = Bitboard(0x8080808080808080);

    let mut sq: usize = 0;

    let mut masks = [[Bitboard::EMPTY; Square::COUNT]; Color::COUNT];

    while sq < 64 {
        // White mask
        if sq > 7 && sq < 56 {
            let mut mask = A_FILE.0 << (sq + 8);

            if sq % 8 > 0 {
                mask |= A_FILE.0 << (sq + 7);
            }

            if sq % 8 < 7 {
                mask |= A_FILE.0 << (sq + 9);
            }

            masks[Color::White as usize][sq] = Bitboard(mask);
        }
        
        // Black mask
        if sq > 7 && sq < 56 {
            let offset = 63 - sq;

            let mut mask = H_FILE.0 >> (offset + 8);

            if sq % 8 > 0 {
                mask |= H_FILE.0 >> (offset + 9);
            }

            if sq % 8 < 7 {
                mask |= H_FILE.0 >> (offset + 7);
            }

            masks[Color::Black as usize][sq] = Bitboard(mask);
        }

        sq += 1;
    }
    
    masks
}

pub const MG_PASSED_PAWN_TABLE: [Eval; Square::COUNT]  = [
    0, 0, 0, 0, 0, 0, 0, 0,
    45, 52, 42, 43, 28, 34, 19, 9,
    48, 43, 43, 30, 24, 31, 12, 2,
    28, 17, 13, 10, 10, 19, 6, 1,
    14, 0, -9, -7, -13, -7, 9, 16,
    5, 3, -3, -14, -3, 10, 13, 19,
    8, 9, 2, -8, -3, 8, 16, 9,
    0, 0, 0, 0, 0, 0, 0, 0,
];

pub const EG_PASSED_PAWN_TABLE: [Eval; Square::COUNT]  = [
	0, 0, 0, 0, 0, 0, 0, 0,
	77, 74, 63, 53, 59, 60, 72, 77,
	91, 83, 66, 40, 30, 61, 67, 84,
	55, 52, 42, 35, 30, 34, 56, 52,
	29, 26, 21, 18, 17, 19, 34, 30,
	8, 6, 5, 1, 1, -1, 14, 7,
	2, 3, -4, 0, -2, -1, 7, 6,
	0, 0, 0, 0, 0, 0, 0, 0,
];

////////////////////////////////////////////////////////////////////////////////
//
// Isolated pawn masks
//
////////////////////////////////////////////////////////////////////////////////

const fn gen_isolated_pawn_masks() -> BBTable {
    const A_FILE: Bitboard = Bitboard(0x101010101010101);

    let mut sq: usize = 0;

    let mut masks = [Bitboard::EMPTY; Square::COUNT];

    while sq < 64 {
        let file = sq % 8;
        let mut mask = 0;

        if file > 0 {
            mask |= A_FILE.0 << file - 1;
        }

        if file < 7 {
            mask |= A_FILE.0 << file + 1;
        }

        masks[sq] = Bitboard(mask);

        sq += 1;
    }
    
    masks
}

////////////////////////////////////////////////////////////////////////////////
//
// Doubled pawn masks
//
////////////////////////////////////////////////////////////////////////////////

const fn gen_doubled_pawn_masks() -> [Bitboard; 8] {
    const A_FILE: Bitboard = Bitboard(0x101010101010101);
    let mut rank: usize = 0;
    let mut masks = [Bitboard::EMPTY; 8];

    while rank < 8 {
        let mask = A_FILE.0 << rank;
        masks[rank] = Bitboard(mask);
        rank += 1;
    }
    
    masks
}

////////////////////////////////////////////////////////////////////////////////
//
// Tests
//
////////////////////////////////////////////////////////////////////////////////

#[test]
fn passed_pawn_masks() {
    use Square::*;
    use Color::*;

    assert_eq!(
        PASSED_PAWN_MASKS[White as usize][E4 as usize], 
        Bitboard(0x3838383800000000)
    );
    assert_eq!(
        PASSED_PAWN_MASKS[Black as usize][E5 as usize], 
        Bitboard(0x38383838)
    );
}

#[test]
fn isolated_pawn_masks() {
    use Square::*;

    assert_eq!(
        ISOLATED_PAWN_MASKS[A6 as usize], 
        Bitboard(0x202020202020202)
    );

    assert_eq!(
        ISOLATED_PAWN_MASKS[E4 as usize], 
        Bitboard(0x2828282828282828)
    );
}
