use chess::bitboard::Bitboard;
use std::mem::size_of;

use crate::transpositions::ZKey;
use crate::zobrist::ZHash;

use super::kp_structure::KingPawnStructure;
use super::S;

#[derive(Copy, Clone, Debug)]
pub struct KingPawnCacheEntry {
  pub hash: ZHash,
  pub score: S,
  pub passers: [Bitboard; 2],
  pub semi_opens: [Bitboard; 2],
  pub outposts: [Bitboard; 2],
}

impl Default for KingPawnCacheEntry {
  fn default() -> Self {
    Self {
      hash: ZHash::NULL,
      score: S::default(),
      passers: [Bitboard::EMPTY, Bitboard::EMPTY],
      semi_opens: [!Bitboard::EMPTY, !Bitboard::EMPTY],
      outposts: [Bitboard::EMPTY, Bitboard::EMPTY],
    }
  }
}

impl KingPawnCacheEntry {
  pub fn new(hash: ZHash, kp_structure: KingPawnStructure) -> Self {
    Self {
      hash,
      score: kp_structure.score(),
      passers: kp_structure.passed_pawns,
      semi_opens: kp_structure.semi_open_files,
      outposts: kp_structure.outposts,
    }
  }
}

pub struct KingPawnCache {
  table: Vec<KingPawnCacheEntry>,
  size: usize,
}

impl KingPawnCache {
  /// Create a new table with the requested capacity in megabytes
  pub fn with_capacity(mb_size: usize) -> KingPawnCache {
    // The number of enties in the TT
    let size = (mb_size << 20) / size_of::<KingPawnCacheEntry>();
    let mut table = Vec::with_capacity(size);
    table.resize_with(size, KingPawnCacheEntry::default);

    KingPawnCache { table, size }
  }

  pub fn insert(&mut self, entry: KingPawnCacheEntry) {
    let key: ZKey = ZKey::from_hash(entry.hash, self.size);
    let existing = self.table[key.0];
    self.table[key.0] = entry;
  }

  // Check whether the hash appears in the transposition table, and return it
  // if so.
  pub fn probe(&self, hash: ZHash) -> Option<KingPawnCacheEntry> {
    let key = ZKey::from_hash(hash, self.size);

    self
      .table
      .get(key.0)
      // .filter(|_| hash != ZHash::NULL)
      .filter(|entry| entry.hash == hash)
      .copied()
  }
}

impl From<KingPawnCacheEntry> for KingPawnStructure {
  fn from(value: KingPawnCacheEntry) -> Self {
    Self {
      score: value.score,
      passed_pawns: value.passers,
      semi_open_files: value.semi_opens,
      outposts: value.outposts,
    }
  }
}
