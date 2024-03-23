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
use crate::evaluate::{get_relative_score, get_absolute_score, Score};

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
    depth: u8,        // 8b
    
    /// The best move we found in the previous search
    best_move: Move,     // 16b

    /// The associated score we found. This could be an upper/lower bound if the
    /// search resulted in a cutoff. Mate scores are normalized to be relative
    /// to the node where the entry was stored.
    score: i16,         // 32b
    
    /// The static eval for the board position
    eval: i16,         // 32b
    
    /// A flag to indicate whether the stored value is an upper/lower bound
    node_type: NodeType, // 8b

    /// A "generational index" to keep track of how long ago the entry was added
    /// (how many searches ago)
    age: u8
}

impl TTEntry {
    const NULL: TTEntry = TTEntry {
        hash: ZHash::NULL,
        best_move: Move::NULL,
        score: i16::MIN,
        eval: i16::MIN,
        depth: 0,
        node_type: NodeType::Exact,
        age: 0
    };

    /// Create a new TT entry
    pub fn new(
        hash: ZHash, 
        best_move: Move, 
        score: Score, 
        eval: Score,
        depth: usize, 
        node_type: NodeType,
        age: u8,
        ply: usize,
    ) -> TTEntry {
        TTEntry { 
            hash, 
            best_move, 
            score: get_relative_score(score, ply) as i16, 
            eval: eval as i16, 
            depth: depth as u8, 
            node_type, 
            age }
    }

    /// Return the hash for the entry
    pub fn get_hash(&self) -> ZHash {
        self.hash
    }

    /// Return the best move for the entry
    pub fn get_move(&self) -> Move {
        self.best_move
    }

    /// Return the score for the entry. In case of a mate score, this value is
    /// normalized!
    pub fn get_score(&self) -> Score {
        self.score as Score
    }

    /// Return the static eval for the entry
    pub fn get_eval(&self) -> Score {
        self.eval as Score
    }

    /// Return the depth for the entry
    pub fn get_depth(&self) -> usize {
        self.depth as usize
    }

    /// Return the type for the entry
    pub fn get_type(&self) -> NodeType {
        self.node_type
    }

    /// Return the age for the entry
    pub fn get_age(&self) -> u8 {
        self.age
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
    pub fn try_score(&self, depth: usize, alpha: Score, beta: Score, ply: usize) -> Option<Score> {
        let entry_type = self.get_type();
        let entry_score = self.get_score();
        let entry_depth = self.get_depth();
        let absolute_score = get_absolute_score(entry_score, ply);

        // Was the search deep enough for our tastes?
        if entry_depth < depth {
            return None;
        }

        // If the score is an upper/lower bound, we might still be able to use
        // it: if the score is a lower bound, but it _still_ beats our current
        // beta, then it doesn't really matter, and we just return beta.
        // Same for upper bounds and alpha.
        match entry_type {
            NodeType::Exact => Some(absolute_score),

            NodeType::Upper if absolute_score <= alpha => {
                Some(alpha)
            },

            NodeType::Lower if absolute_score >= beta => {
                Some(beta)
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

    /// The age of the transposition table, incremented every time a new search
    /// is run. This is used to check how long ago an entry was inserted (and
    /// hence, how relevant it still is).
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
    /// 
    /// If there's already an entry present, replace it if:
    /// 1. The existing entry's age is less than the current age
    /// 2. The existing entry's depth is less than the current entry's
    pub fn insert(&mut self, entry: TTEntry) {
        use NodeType::*;
        let key: ZKey = ZKey::from_hash(entry.hash, self.size);
        let existing = self.table[key.0];

        // Don't replace if the recieved entry has a null move
        if !existing.is_empty() && entry.get_move() == Move::NULL {
            return;
        }

        if existing.is_empty() {
            self.table[key.0] = entry;

            // Empty slot, count as a new occupation
            self.inserts +=1;
            self.occupancy += 1;
        } else if existing.age != self.age 
            || existing.depth < entry.depth 
            || existing.hash != entry.hash
            || entry.get_type() == Exact && existing.get_type() != Exact
        {
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

    /// Instruct the CPU to read the requested TT entry into the CPU cache ahead
    /// of time.
    pub fn prefetch(&self, hash: ZHash) {
        // get a reference to the entry in the table:
        let key = ZKey::from_hash(hash, self.size);
        let entry = &self.table[key.0];

        // prefetch the entry:
        #[cfg(target_arch = "x86_64")]
        unsafe {
            use std::arch::x86_64::{_mm_prefetch, _MM_HINT_T0};
            _mm_prefetch((entry as *const TTEntry).cast::<i8>(), _MM_HINT_T0);
        }
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

    /// Get the current age of the transposition table
    pub fn get_age(&self) -> u8 {
        self.age
    }

    /// Increment the age of the transposition table
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
