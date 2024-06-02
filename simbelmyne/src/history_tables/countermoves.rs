use std::ops::{Index, IndexMut};

/// Countermove table
///
/// The countermove table, similar to the killers table, stores quiet moves that
/// produced a beta-cutoff. Where the Killers table will store them by ply (and,
/// hence, not super local), the countermove table stores them by _the previously
/// played move_. (Get it? Countermove?)
///
/// We only store a single countermove, and play it right after the killer moves.

use chess::movegen::moves::Move;
use chess::piece::Piece;
use chess::square::Square;

use super::history::HistoryIndex;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct CountermoveTable {
    scores: [[Option<Move>; Square::COUNT]; Piece::COUNT]
}

impl CountermoveTable {
    /// Create a new CountermoveTable on the heap
    pub fn boxed() -> Box<Self> {
        #![allow(clippy::cast_ptr_alignment)]
        // SAFETY: we're allocating a zeroed block of memory, and then casting 
        // it to a Box<Self>. This is fine! 
        // [[HistoryTable; Square::COUNT]; Piece::COUNT] is just a bunch of i16s
        // in disguise, which are fine to zero-out.
        unsafe {
            let layout = std::alloc::Layout::new::<Self>();
            let ptr = std::alloc::alloc_zeroed(layout);
            if ptr.is_null() {
                std::alloc::handle_alloc_error(layout);
            }
            Box::from_raw(ptr.cast())
        }
    }
}

impl Index<HistoryIndex> for CountermoveTable {
    type Output = Option<Move>;

    fn index(&self, index: HistoryIndex) -> &Self::Output {
        &self.scores[index.1 as usize][index.0 as usize]
    }
}

impl IndexMut<HistoryIndex> for CountermoveTable {
    fn index_mut(&mut self, index: HistoryIndex) -> &mut Self::Output {
        &mut self.scores[index.1 as usize][index.0 as usize]
    }
}
