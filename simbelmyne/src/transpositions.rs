//! The transposition table is one of the most important data structures in the
//! engine.
//!
//! As we're searching through the game tree, we're bound to come across 
//! duplicate board positions. It would be great if we could re-use some of the
//! work we've already done in other branches of the tree. Enter the 
//! Transposition table.
//!
//! We store some key information about the node (the score we found, how deep
//! we searched the node for, whether the score is an upper/lower bound, etc...)
//! and use the positions Zobrist hash as a key to index into it.
//!
//! A couple of concerns: 
//! 1. There's more possible board positions than Zobrist hashes (2^64). That 
//! means we'll inevitably have hash collisions. (So-called Type-1 collisions).
//! These should be pretty rare, though...
//!
//! 2. We don't want a lookup table with 2^64 entries, that would be absurd.
//! Instead, we truncate the Zobrist hash to however many bits we need to 
//! accomodate for the requested table size. Reducing the key size means we'll
//! get _many_ more collisions (called Type-2 Collisions). Because these are 
//! much more frequent, we store the full hash along with the the rest of the
//! values, so that when we read the entry from the table, we can check our 
//! board's position with the full hash, to make sure we (probably) didn't get
//! a false positive.

use std::mem::size_of;
use chess::movegen::moves::Move;
use crate::zobrist::ZHash;
use crate::evaluate::Score;
use crate::evaluate::Eval;

/// A flag that stores whether the entry corresponds to a PV, fail-high or 
/// fail-low node. Or, equivalently, whether the score saved in the entry is 
/// exact, and upper bound, or a lower bound.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum NodeType {
    Exact = 0b00,
    Upper = 0b01,
    Lower = 0b10,
}

/// A single TT entry.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct TTEntry {
    /// The hash of the board the entry is for, used to test for hash 
    /// collisions
    hash: ZHash,         // 64b
    
    /// The depth we searched to from this node
    depth: usize,        // 8b
    
    /// The best move we found in the previous search
    best_move: Move,     // 16b

    /// The associated score we found. This could be an upper/lower bound if the
    /// search resulted in a cutoff
    score: Eval,         // 32b
    
    /// A flag to indicate whether the stored value is an upper/lower bound
    node_type: NodeType, // 8b

    age: u8
}

impl TTEntry {
    const NULL: TTEntry = TTEntry{
        hash: ZHash::NULL,
        best_move: Move::NULL,
        score: Score::MIN,
        depth: 0,
        node_type: NodeType::Exact,
        age: 0
    };

    /// Create a new TT entry
    pub fn new(
        hash: ZHash, 
        best_move: Move, 
        score: Eval, 
        depth: usize, 
        node_type: NodeType,
        age: u8
    ) -> TTEntry {
        TTEntry { hash, best_move, score, depth, node_type, age }
    }

    /// Return the hash for the entry
    pub fn get_hash(&self) -> ZHash {
        self.hash
    }

    /// Return the best move for the entry
    pub fn get_move(&self) -> Move {
        self.best_move
    }

    /// Return the score for the entry
    pub fn get_score(&self) -> Eval {
        self.score
    }

    /// Return the depth for the entry
    pub fn get_depth(&self) -> usize {
        self.depth
    }

    /// Return the type for the entry
    pub fn get_type(&self) -> NodeType {
        self.node_type
    }

    /// Check whether there's any data stored in the entry
    pub fn is_empty(&self) -> bool {
        self.hash == ZHash::NULL
    }

    /// Check whether we can use the entry in the first place, and return the 
    /// move/score if so.
    ///
    /// We don't want to use results that didn't search as deep as we're meant 
    /// to search. Additionally,, we want to be careful returning a value that 
    /// isn't the _actual_ value, but an upper/lower bound.
    pub fn try_use(&self, depth: usize, alpha: Eval, beta: Eval) -> Option<(Move, Eval)> {
        let entry_type = self.get_type();
        let entry_score = self.get_score();
        let entry_depth = self.get_depth();
        let entry_move = self.get_move();

        // Was the search deep enough for our tastes?
        if entry_depth < depth {
            return None;
        }

        // If the score is an upper/lower bound, we might still be able to use
        // it: if the score is a lower bound, but it _still_ beats our current
        // beta, then it doesn't really matter, and we just return beta.
        // Same for upper bounds and alpha.
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

/// A transposition table that stores previously searched results
pub struct TTable {
    /// A collection of entries. Stored on the heap because we need to be able
    /// to dynamically resize it. We only instantiate it once at the start of 
    /// the search though, so this isn't a big deal.
    table: Vec<TTEntry>,

    /// The key length of the transposition table
    size: usize,

    ///  The number of non-empty entries stored in the table
    occupancy: usize,

    /// The number of entries that have been inserted so far
    inserts: usize,

    age: u8,
}

impl TTable {
    /// Resize table to the size requested in MiB
    pub fn resize(&mut self, mb_size: usize) {
        let size = (mb_size << 20) / size_of::<TTEntry>();
        self.table.resize_with(size, TTEntry::default);
    }

    /// Create a new table with the requested capacity in megabytes
    pub fn with_capacity(mb_size: usize) -> TTable {
        let size = (mb_size << 20) / size_of::<TTEntry>();

        let mut table: TTable = TTable { 
            table: Vec::new(),
            size,
            occupancy: 0,
            inserts: 0,
            age: 0,
        };

        table.resize(mb_size);
        table
    }

    /// Insert an entry into the transposition table
    pub fn insert(&mut self, entry: TTEntry) {
        let key: ZKey = ZKey::from_hash(entry.hash, self.size);
        let slot = self.table[key.0];

        if slot.is_empty() {
            self.table[key.0] = entry;

            // Empty slot, count as a new occupation
            self.inserts +=1;
            self.occupancy += 1;
        } else if entry.age < self.age || entry.depth > slot.depth {
            self.table[key.0] = entry;

            // Evicting existing record, doesn't change occupancy count
            self.inserts +=1;
        }
    }

    // Check whether the hash appears in the transposition table, and return it 
    // if so.
    pub fn probe(&self, hash: ZHash) -> Option<TTEntry> {
        let key = ZKey::from_hash(hash, self.size);

        self.table.get(key.0)
            .filter(|entry| !entry.is_empty())
            .filter(|entry| entry.hash == hash)
            .copied()
    }

    /// Return the occupancy as a fractional number (0 - 1)
    pub fn occupancy(&self) -> f32 {
        self.occupancy as f32 / self.table.len() as f32
    }

    /// Return the number of entries that were added (excluding overwrites)
    pub fn inserts(&self) -> usize {
        self.inserts
    }

    /// Return the number of entries that were overwritten
    pub fn overwrites(&self) -> usize {
        self.occupancy - self.inserts
    }

    pub fn get_age(&self) -> u8 {
        self.age
    }

    pub fn increment_age(&mut self) {
        self.age += 1;
    }
}

////////////////////////////////////////////////////////////////////////////////
//
// Zobrist keys
//
////////////////////////////////////////////////////////////////////////////////

/// ZKeys are Lookup keys derived from a Zobrist hash. 
///
/// They are truncated to the requested size, in order to acommodate for the 
/// desired transposition table size
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ZKey(pub usize); 

impl ZKey {
    pub fn from_hash(hash: ZHash, size: usize) -> Self {
        ZKey((hash.0 as usize) % size)
    }
}

////////////////////////////////////////////////////////////////////////////////
//
// Utility traits
//
////////////////////////////////////////////////////////////////////////////////

impl Default for TTEntry {
    fn default() -> Self {
        TTEntry::NULL
    }
}
