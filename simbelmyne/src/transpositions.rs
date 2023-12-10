use std::mem::size_of;

use chess::movegen::moves::Move;

use crate::{zobrist::{ZHash, ZKey}, evaluate::{Eval, Score}};

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
    score: Eval,         // 32b
    depth: usize,        // 8b
    node_type: NodeType, // 8b
} //                    -------- 128b

impl TTEntry {
    const NULL: TTEntry = TTEntry{
        hash: ZHash::NULL,
        best_move: Move::NULL,
        score: Score::MIN,
        depth: 0,
        node_type: NodeType::Exact,
    };

    pub fn new(
        hash: ZHash, 
        best_move: Move, 
        score: Eval, 
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

    pub fn get_score(&self) -> Eval {
        self.score
    }

    pub fn get_depth(&self) -> usize {
        self.depth
    }

    pub fn get_type(&self) -> NodeType {
        self.node_type
    }

    pub fn is_empty(&self) -> bool {
        self.hash == ZHash::NULL
    }

    pub fn try_use(&self, depth: usize, alpha: Eval, beta: Eval) -> Option<(Move, Eval)> {
        let entry_type = self.get_type();
        let entry_score = self.get_score();
        let entry_depth = self.get_depth();
        let entry_move = self.get_move();


        if entry_depth < depth {
            return None;
        }

        match entry_type {
            NodeType::Exact => Some((entry_move, entry_score)),
            NodeType::Upper if entry_score <= alpha => {
                Some((entry_move, alpha))
            },
            NodeType::Lower if entry_score >= beta => {
                Some((entry_move, beta))
            },
            _ => None
        }
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
        let slot = self.table[key.0];

        if slot.is_empty() {
            // Empty slot, count as a new occupation
            self.table[key.0] = entry;
            self.inserts +=1;
            self.occupancy += 1;
        } else if entry.depth > slot.depth {
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

    /// Return the occupancy as a fractional number (0 - 1)
    pub fn occupancy(&self) -> f32 {
        self.occupancy as f32 / self.table.len() as f32
    }

    pub fn inserts(&self) -> usize {
        self.inserts
    }

    pub fn overwrites(&self) -> usize {
        self.overwrites
    }
}
