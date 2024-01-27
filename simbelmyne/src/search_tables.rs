//! Different lookup tables that are used during the search and move ordering.
//!
//! This does not include the Transposition table, which is complicated enough
//! to get its own module (`transpositions').

use crate::search::params::{MAX_KILLERS, HIST_AGE_DIVISOR};
use std::fmt::Display;
use chess::square::Square;
use chess::piece::Piece;
use chess::movegen::moves::Move;
use crate::search::params::MAX_DEPTH;

////////////////////////////////////////////////////////////////////////////////
//
// PV Table
//
////////////////////////////////////////////////////////////////////////////////

/// A PV table is a fixed length array and an index, and stores the principal
/// variation for a given node.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct PVTable {
    /// The principal variation moves collected so far
    pv: [Move; MAX_DEPTH],

    /// The length, being the index of the last move stored in the array (+ 1).
    len: usize,
}

impl PVTable {
    /// Create a new PV table
    pub fn new() -> Self {
        Self {
            pv: [Move::NULL; MAX_DEPTH],
            len: 0
        }
    }

    /// Clear the PV table by re-setting its index.
    /// Note that we're not actually clearing any data here.
    pub fn clear(&mut self) {
        self.len = 0;
    }

    /// The main operation for the PV table: a PV node will try and prepend 
    /// the PV it got from its children with its own PV move and pass it back
    /// up.
    pub fn add_to_front(&mut self, mv: Move, existing: &Self) {
        self.len = existing.len + 1;
        self.pv[0] = mv;
        self.pv[1..=self.len].copy_from_slice(&existing.pv[0..=existing.len]);
    }

    /// Return the PV moves as a slice
    pub fn moves(&self) -> &[Move] {
        &self.pv[..self.len]
    }

    /// Return "the" PV move, being the first move in the principal variation
    pub fn pv_move(&self) -> Move {
        self.moves()[0]
    }
}

impl Display for PVTable {
    /// Display the PV table as its UCI formatted string
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "pv")?;

        for (i, mv) in self.pv.iter().enumerate() {
            write!(f, " {mv}")?;

            if i == self.len {
                break;
            }
        }

        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////
//
// Killers table
//
////////////////////////////////////////////////////////////////////////////////

/// Store a list of "Killer moves"
///
/// These are quiet moves (i.e., not captures or promotions) that still managed
/// to cause a beta cutoff. Think your forks, mates, etc...
/// Killer moves are stored per ply, and and are less "relevant" than more 
/// exact information like the TT table (which is as exact as it gets: it's a
/// good move _for this specific position_. Still, a mate in a variation 
/// at the same ply is very likely to still be a mate in many other sibling
/// branches.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Killers {
    // The array of killer moves
    moves: [Move; MAX_KILLERS],

    // The length up to which we've stored _actual_ moves. Anything beyond `len`
    // is considered garbage.
    len: usize,
}

impl Killers {
    /// Create a new Killers table
    pub fn new() -> Self {
        Self {
            moves: [Move::NULL; MAX_KILLERS],
            len: 0,
        }
    }

    /// Return the length of the killers table (i.e., the number of stored moves)
    pub fn len(&self) -> usize {
        self.len
    }

    // Return the moves in the table
    pub fn moves(&self) -> &[Move] {
        &self.moves[..self.len]

    }

    /// Add a killer move to the front of the table, with the additional 
    /// semantics that no move can appear twice in the table.
    pub fn add(&mut self, mv: Move) {
        if !self.moves.contains(&mv) {
            self.moves.rotate_right(1);
            self.moves[0] = mv;
            self.len = usize::min(self.len + 1, MAX_KILLERS);
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
//
// History table
//
////////////////////////////////////////////////////////////////////////////////

/// History tables assign scores to quiet moves that caused beta cutoffs 
/// _anywhere_ in the search tree.
///
/// That means that, if TT tables are the most "accurate", and Killer moves are
/// slightly less so, then History tables are the least precise: They don't even
/// relate to moves _in the current ply_. They're literally moves played 
/// _anywhere_ in the search tree. As such, they're typically the least valuable 
/// way or sorting quiet, but still somewhat better than not sorting at all!
///
/// The tables are indexed by piece type and target square (rather than, say,
/// source square and target square) to reduce the footprint of the table.
///
/// The scores are scaled by the remaining search depth (how high up in the 
/// tree the move was played), so we don't flood the scores with moves played at
/// the leaves, which are inherently less valuable.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct HistoryTable {
    scores: [[i16; Square::COUNT]; Piece::COUNT]
}

pub const MAX_HIST_SCORE: i16 = i16::MAX/2;

impl HistoryTable {
    /// Create a new HistoryTable
    pub fn new() -> Self {
        Self {
            scores: [[0; Square::COUNT]; Piece::COUNT]
        }
    }

    fn bonus(depth: usize) -> i32 {
        if depth > 13 {
            32
        } else {
            (16 * depth * depth + 128 * usize::max(depth - 1, 0)) as i32
        }
    }

    // Add a value to a table entry, saturating smoothly as the entry approaches
    // MAX_HIST_SCORE
    fn saturating_add(&mut self, mv: &Move, piece: Piece, bonus: i32) {
        let current = self.get(mv, piece);

        // "tapered" bonus: (1 +- current / MAX_SCORE) * bonus
        // boosted by 2x when adding negative bonus to high positive score,
        // tapered to 0 when adding negative bonus to high negative score
        let modified_bonus = bonus - (current as i32) * bonus.abs() / MAX_HIST_SCORE as i32;

        self.set(mv, piece, current + modified_bonus as i16);
    }

    /// Incerement the value of a given move in the table
    pub fn increment(&mut self, mv: &Move, piece: Piece, depth: usize) {
        self.saturating_add(mv, piece, Self::bonus(depth));
    }

    /// Decrement the value of a given move in the table
    pub fn decrement(&mut self, mv: &Move, piece: Piece, depth: usize) {
        self.saturating_add(mv, piece, -Self::bonus(depth) / 8);
    }

    /// Set the score for a particular move and piece
    pub fn set(&mut self, mv: &Move, piece: Piece, value: i16) {
        self.scores[piece as usize][mv.tgt() as usize] = value;
    }

    /// Get the score for a particular move and piece
    pub fn get(&self, mv: &Move, piece: Piece) -> i16 {
        self.scores[piece as usize][mv.tgt() as usize]
    }

    /// Reduce the values from previous searches so they don't swamp this 
    /// search's history values.
    pub fn age_entries(&mut self) {
        for piece_idx in 0..Piece::COUNT {
            for sq_idx in 0..Square::COUNT {
                self.scores[piece_idx][sq_idx] /= HIST_AGE_DIVISOR;
            }
        }
    }
}
