use crate::bitboard::Bitboard;
use crate::square::Square;

use super::lookups::bishop_mask;
use super::lookups::gen_bishop_attacks;
use super::lookups::gen_rook_attacks;
use super::lookups::rook_mask;

#[cfg(not(all(target_arch = "x86_64", target_feature = "bmi2")))]
compile_error!("pext feature can only be enabled if target has BMI2.");

////////////////////////////////////////////////////////////////////////////////
//
// Square method impls
//
////////////////////////////////////////////////////////////////////////////////

impl Square {
  /// Get a bitboard for all the squares visible to a bishop on this square.
  pub fn bishop_squares(self, blockers: Bitboard) -> Bitboard {
    let pext_entry = BISHOP_ENTRIES[self];
    let idx = pext_entry.index(blockers);

    BISHOP_ATTACKS[idx]
  }

  /// Get a bitboard for all the squares visible to a rook on this square.
  pub fn rook_squares(self, blockers: Bitboard) -> Bitboard {
    let pext_entry = ROOK_ENTRIES[self];
    let idx = pext_entry.index(blockers);

    ROOK_ATTACKS[idx]
  }
}

////////////////////////////////////////////////////////////////////////////////
//
// Attack table generation
//
////////////////////////////////////////////////////////////////////////////////

const BISHOP_ATTACKS: [Bitboard; 5248] = gen_bishop_attacks_table();

#[allow(long_running_const_eval)]
const ROOK_ATTACKS: [Bitboard; 102400] = gen_rook_attacks_table();

const BISHOP_ENTRIES: [PextEntry; Square::COUNT] = gen_entries::<true>();
const ROOK_ENTRIES: [PextEntry; Square::COUNT] = gen_entries::<false>();

const fn gen_bishop_attacks_table() -> [Bitboard; 5248] {
  let mut table = [Bitboard::EMPTY; 5248];
  let mut sq: usize = 0;

  while sq < 64 {
    let entry = BISHOP_ENTRIES[sq];
    let mut subset: u64 = 0;

    // First treat the empty subset
    let attacks = gen_bishop_attacks(Square::ALL[sq], Bitboard(subset));
    let blockers = Bitboard(subset);
    let idx = entry.index_const(blockers);
    table[idx] = attacks;
    subset = subset.wrapping_sub(entry.mask.0) & entry.mask.0;

    // For every subset of possible blockers, get the attacked squares and
    // store them in the table.
    while subset != 0 {
      let attacks = gen_bishop_attacks(Square::ALL[sq], Bitboard(subset));
      let blockers = Bitboard(subset);
      let idx = entry.index_const(blockers);
      table[idx] = attacks;

      subset = subset.wrapping_sub(entry.mask.0) & entry.mask.0;
    }

    sq += 1;
  }

  table
}

const fn gen_rook_attacks_table() -> [Bitboard; 102400] {
  let mut table = [Bitboard::EMPTY; 102400];
  let mut sq: usize = 0;

  while sq < 64 {
    let entry = ROOK_ENTRIES[sq];
    let mut subset: u64 = 0;

    // First treat the empty subset
    let attacks = gen_rook_attacks(Square::ALL[sq], Bitboard(subset));
    let blockers = Bitboard(subset);
    let idx = entry.index_const(blockers);
    table[idx] = attacks;
    subset = subset.wrapping_sub(entry.mask.0) & entry.mask.0;

    // For every subset of possible blockers, get the attacked squares and
    // store them in the table.
    while subset != 0 {
      let attacks = gen_rook_attacks(Square::ALL[sq], Bitboard(subset));
      let blockers = Bitboard(subset);
      let idx = entry.index_const(blockers);
      table[idx] = attacks;

      subset = subset.wrapping_sub(entry.mask.0) & entry.mask.0;
    }

    sq += 1;
  }

  table
}

////////////////////////////////////////////////////////////////////////////////
//
// PextEntry
//
////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Copy, Clone)]
pub struct PextEntry {
  pub mask: Bitboard,
  pub offset: u32,
}

impl PextEntry {
  /// Given an entry,
  pub fn index(&self, blockers: Bitboard) -> usize {
    let index = pext_u64(blockers.0, self.mask.0) as usize;
    let offset = self.offset as usize;
    offset + index
  }

  const fn index_const(&self, blockers: Bitboard) -> usize {
    let index = pext_const(blockers.0, self.mask.0) as usize;
    let offset = self.offset as usize;
    offset + index
  }
}

////////////////////////////////////////////////////////////////////////////////
//
// Utilities
//
////////////////////////////////////////////////////////////////////////////////

/// A Sofe wrapper around the _pext_u64 intrinsic
fn pext_u64(value: u64, mask: u64) -> u64 {
  // SAFETY: A compile error is raised if PEXT is not available. PEXT is
  // always safe if available.
  unsafe { core::arch::x86_64::_pext_u64(value, mask) }
}

/// Poor man's pext that can run at compile time
const fn pext_const(value: u64, mut mask: u64) -> u64 {
  let mut res = 0;
  let mut bb = 1;
  loop {
    if mask == 0 {
      break;
    }
    if value & mask & (mask.wrapping_neg()) != 0 {
      res |= bb;
    }
    mask &= mask - 1;
    bb += bb;
  }

  res
}
////////////////////////////////////////////////////////////////////////////////
//
// PEXT entries
//
////////////////////////////////////////////////////////////////////////////////

pub const fn gen_entries<const BISHOP: bool>() -> [PextEntry; Square::COUNT] {
  let mut offset = 0;
  let mut entries: [PextEntry; Square::COUNT] = [PextEntry {
    mask: Bitboard::EMPTY,
    offset: 0,
  }; Square::COUNT];

  let mut sq = 0;
  while sq < 64 {
    let mask = if BISHOP {
      bishop_mask(Square::ALL[sq])
    } else {
      rook_mask(Square::ALL[sq])
    };

    let num_bits = mask.0.count_ones();
    let entry = PextEntry { mask, offset };

    entries[sq] = entry;

    offset += 1 << num_bits;
    sq += 1;
  }

  entries
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::movegen::lookups::gen_rook_attacks;

  #[test]
  fn test_indexing() {
    use Square::*;
    let sq = D4;
    let blockers: Bitboard = vec![D3, F4, A4, D7].into_iter().collect();

    let entry = ROOK_ENTRIES[sq];
    println!("Entry: {entry:?}");

    let index = entry.index(blockers);
    let index_const = entry.index_const(blockers);

    assert_eq!(index, index_const);

    // for attacks in ROOK_ATTACKS.iter().take(100) {
    //     println!("{attacks}");
    // }

    for entry in ROOK_ENTRIES {
      println!("{}", entry.mask);
    }

    panic!();
  }

  #[test]
  fn test_lookups() {
    use Square::*;
    let sq = D4;
    let blockers: Bitboard = vec![D3, F4, A4, D7].into_iter().collect();

    let generated = gen_rook_attacks(sq, blockers);
    let lookup = sq.rook_squares(blockers);

    println!("Generated:\n{generated}");
    println!("Lookup:\n{lookup}");

    assert_eq!(generated, sq.rook_squares(blockers));
  }
}
