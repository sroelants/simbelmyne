use chess::bitboard::Bitboard;
use chess::piece::Color;
use chess::square::Square;

type BBTable = [Bitboard; Square::COUNT];

pub const QUEENSIDE: Bitboard = Bitboard(
  0x0101010101010101
    | 0x0101010101010101 << 1
    | 0x0101010101010101 << 2
    | 0x0101010101010101 << 3,
);

pub const KINGSIDE: Bitboard = Bitboard(
  0x0101010101010101 << 4
    | 0x0101010101010101 << 5
    | 0x0101010101010101 << 6
    | 0x0101010101010101 << 7,
);

pub const CENTER_SQUARES: Bitboard = Bitboard(0x0000001818000000);

////////////////////////////////////////////////////////////////////////////////
//
// Lookup tables
//
////////////////////////////////////////////////////////////////////////////////

pub const FILES: BBTable = gen_files();

pub const PASSED_PAWN_MASKS: [BBTable; Color::COUNT] = gen_passed_pawn_masks();

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
  use Color::*;
  use Square::*;

  assert_eq!(PASSED_PAWN_MASKS[White][E4], Bitboard(0x3838383800000000));
  assert_eq!(PASSED_PAWN_MASKS[Black][E5], Bitboard(0x38383838));
}
