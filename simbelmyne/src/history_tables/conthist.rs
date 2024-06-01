/// Continuation history table
///
///
use std::ops::{Index, IndexMut};
use chess::{piece::Piece, square::Square};
use super::history::{HistoryIndex, HistoryTable};

#[derive(Debug)]
pub struct ContHist {
    table: [[HistoryTable; Square::COUNT]; Piece::COUNT]
}

impl ContHist {
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

impl Index<HistoryIndex> for ContHist {
    type Output = HistoryTable;

    fn index(&self, index: HistoryIndex) -> &Self::Output {
        &self.table[index.1 as usize][index.0 as usize]
    }
}

impl IndexMut<HistoryIndex> for ContHist {
    fn index_mut(&mut self, index: HistoryIndex) -> &mut Self::Output {
        &mut self.table[index.1 as usize][index.0 as usize]
    }
}
