use std::ops::Index;

use chess::piece::PieceType;

use super::history::HistoryTable;

#[derive(Copy, Clone, Debug)]
pub struct TacticalHistoryTable {
    tables: [HistoryTable; PieceType::COUNT]
}

impl TacticalHistoryTable {
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

impl Index<PieceType> for TacticalHistoryTable {
    type Output = HistoryTable;

    fn index(&self, index: PieceType) -> &Self::Output {
        &self.tables[index]
    }
}
