use chess::bitboard::Bitboard;
use std::mem::size_of;

use crate::transpositions::ZKey;
use crate::zobrist::ZHash;

use super::{pawn_structure::PawnStructure, S};

#[derive(Copy, Clone, Debug, Default)]
pub struct PawnCacheEntry {
    pub hash: ZHash,
    pub score: S,
    pub passers: [Bitboard; 2],
    pub semi_opens: [Bitboard; 2],
    pub outposts: [Bitboard; 2]
}

impl PawnCacheEntry {
    pub fn new(hash: ZHash, pawn_structure: PawnStructure) -> Self {
        Self {
            hash,
            score: pawn_structure.score(),
            passers: pawn_structure.passed_pawns,
            semi_opens: pawn_structure.semi_open_files,
            outposts: pawn_structure.outposts,
        }
    }
}

pub struct PawnCache {
    table: Vec<PawnCacheEntry>,
    size: usize,
}

impl PawnCache {
    /// Create a new table with the requested capacity in megabytes
    pub fn with_capacity(mb_size: usize) -> PawnCache {
        // The number of enties in the TT
        let size = (mb_size << 20) / size_of::<PawnCacheEntry>();
        let mut table = Vec::with_capacity(size);
        table.resize_with(size, PawnCacheEntry::default);

        PawnCache { table, size }
    }

    pub fn insert(&mut self, entry: PawnCacheEntry) {
        let key: ZKey = ZKey::from_hash(entry.hash, self.size);
        let existing = self.table[key.0];
        self.table[key.0] = entry;
    }

    // Check whether the hash appears in the transposition table, and return it 
    // if so.
    //
    pub fn probe(&self, hash: ZHash) -> Option<PawnCacheEntry> {
        let key = ZKey::from_hash(hash, self.size);

        self.table.get(key.0)
            .filter(|entry| entry.hash == hash)
            .copied()
    }
}

impl From<PawnCacheEntry> for PawnStructure {
    fn from(value: PawnCacheEntry) -> Self {
        Self {
            score: value.score,
            passed_pawns: value.passers,
            semi_open_files: value.semi_opens,
            outposts: value.outposts,
        }
    }
}
