use std::mem::size_of;

use chess::movegen::moves::Move;

use crate::zobrist::{ZHash, ZKey};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum NodeType {
    Exact = 0b00,
    Upper = 0b01,
    Lower = 0b10,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct TTEntry {
    hash: ZHash,         // 64b
    best_move: Move,     // 16b
    score: i32,          // 32b
    depth: usize,        // 8b
    node_type: NodeType, // 8b
} //                    -------- 128b

impl TTEntry {
    const NULL: TTEntry = TTEntry{
        hash: ZHash::NULL,
        best_move: Move::NULL,
        score: i32::MIN,
        depth: 0,
        node_type: NodeType::Exact,
    };

    pub fn new(
        hash: ZHash, 
        best_move: Move, 
        score: i32, 
        depth: usize, 
        node_type: NodeType,
    ) -> TTEntry {
        TTEntry { hash, best_move, score, depth, node_type }
    }


    pub fn get_hash(&self) -> ZHash {
        self.hash
    }

    pub fn get_move(&self) -> Move {
        self.best_move
    }

    pub fn get_score(&self) -> i32 {
        self.score
    }

    pub fn get_depth(&self) -> usize {
        self.depth
    }

    pub fn get_type(&self) -> NodeType {
        self.node_type
    }
}

impl Default for TTEntry {
    fn default() -> Self {
        TTEntry::NULL
    }
}

pub struct TTable {
    table: Vec<TTEntry>,
    occupancy: usize,
    inserts: usize,
    overwrites: usize,
}

impl TTable {
    pub const DEFAULT_SIZE: usize = 16;
    pub const COUNT: usize = Self::DEFAULT_SIZE * (1 << 20) / std::mem::size_of::<TTEntry>();

    // Resize table to the size requested in MiB
    pub fn resize(&mut self, mb_size: usize) {
        let entries = (mb_size << 20) / size_of::<TTEntry>();
        self.table.resize_with(entries, TTEntry::default);
    }

    pub fn with_capacity(mb_size: usize) -> TTable {
        let mut table: TTable = TTable { 
            table: Vec::new(),
            occupancy: 0,
            inserts: 0,
            overwrites: 0,
        };

        table.resize(mb_size);
        table
    }

    pub fn insert(&mut self, entry: TTEntry) {
        let key: ZKey<{Self::COUNT}> = entry.hash.into();
        let old_entry = self.table[key.0];

        if old_entry.hash == ZHash::default() {
            // Empty slot, count as a new occupation
            self.table[key.0] = entry;
            self.inserts +=1;
            self.occupancy += 1;
        } else if entry.depth > old_entry.depth {
            // Evicting existing record, doesn't change occupancy count
            self.inserts +=1;
            self.overwrites += 1;
            self.table[key.0] = entry;
        }
    }

    // Check whether the hash appears in the transposition table, and return it 
    // if so.
    pub fn probe(&self, hash: ZHash) -> Option<TTEntry> {
        let key: ZKey<{Self::COUNT}> = hash.into();
        self.table.get(key.0)
            .filter(|entry| entry.get_hash() == hash)
            .filter(|entry| entry.get_move() != Move::NULL)
            .copied()
    }

    /// Return the occupancy as a rounded percentage (0 - 100)
    pub fn occupancy(&self) -> usize {
        100 * self.occupancy / self.table.len()
    }

    pub fn inserts(&self) -> usize {
        self.inserts
    }

    pub fn overwrites(&self) -> usize {
        self.overwrites
    }
}

#[cfg(test)]
mod tests {
    use colored::Colorize;

    use super::*;
    use crate::{tests::TEST_POSITIONS, position::Position, search::{SearchOpts, MAX_DEPTH}, time_control::TimeControl};

    #[test]
    /// Running with or without TT should not affect the outcome of the best move
    fn transposition_table() {
        const DEPTH: usize = 5;
        let mut results: Vec<(&str, [Move; MAX_DEPTH], [Move; MAX_DEPTH])> = Vec::new();

        for fen in TEST_POSITIONS {
            let board = fen.parse().unwrap();
            let position = Position::new(board);
            let mut opts = SearchOpts::new();
            opts.ordering = false;
            opts.mvv_lva = false;
            opts.killers = false;
            let mut tt = TTable::with_capacity(64);
            let (tc, _) = TimeControl::fixed_depth(DEPTH);

            let search1 = position.search(&mut tt, opts, tc);

            let mut opts = SearchOpts::new();
            opts.tt = false;
            opts.ordering = false;
            opts.mvv_lva = false;
            opts.killers = false;
            let mut tt = TTable::with_capacity(64);
            let (tc, _) = TimeControl::fixed_depth(DEPTH);

            let search2 = position.search(&mut tt, opts, tc);

            results.push((fen, search1.best_moves, search2.best_moves));
            if search1.best_moves == search2.best_moves {
                println!("{}", fen.green());
            } else {
                println!("{}", fen.red());
            }
        }

        let all = TEST_POSITIONS.len();
        let passed = results.iter().filter(|(_, res1, res2)| res1 == res2).count();
        let failed = all - passed;
        println!("{} passed, {} failed", passed.to_string().green(), failed.to_string().red());

        assert_eq!(
            passed, 
            all, 
            "{} results differed when using TT lookups", 
            failed.to_string().red()
        );
    }
}
