use chess::{bitboard::Bitboard, piece::Color, square::Square};

type BBTable = [Bitboard; Square::COUNT];

////////////////////////////////////////////////////////////////////////////////
//
// Lookup tables
//
////////////////////////////////////////////////////////////////////////////////

pub const FILES: BBTable = gen_files();

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
        if sq < 56 {
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
        if sq > 7 {
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
// Files
//
////////////////////////////////////////////////////////////////////////////////

const fn gen_files() -> BBTable {
    const A_FILE: Bitboard = Bitboard(0x101010101010101);
    let mut sq: usize = 0;
    let mut masks = [Bitboard::EMPTY; Square::COUNT];

    while sq < 64 {
        let file = sq % 8;
        masks[sq] = Bitboard(A_FILE.0 << file);
        sq += 1;
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
