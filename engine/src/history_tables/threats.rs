use std::ops::{Index, IndexMut};
use chess::{bitboard::Bitboard, movegen::moves::Move};
use super::history::HistoryTable;

#[derive(Debug)]
pub struct ThreatsHistoryTable {
    tables: [[HistoryTable; 2]; 2]
}

impl ThreatsHistoryTable {
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

impl Index<ThreatIndex> for ThreatsHistoryTable {
    type Output = HistoryTable;

    fn index(&self, idx: ThreatIndex) -> &Self::Output {
        &self.tables[idx.from_threat][idx.to_threat]
    }
}

impl IndexMut<ThreatIndex> for ThreatsHistoryTable {
    fn index_mut(&mut self, idx: ThreatIndex) -> &mut Self::Output {
        &mut self.tables[idx.from_threat][idx.to_threat]
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ThreatIndex {
    from_threat: usize,
    to_threat: usize,
}

impl ThreatIndex {
    pub fn new(threats: Bitboard, mv: Move) -> Self {
        Self {
            from_threat: threats.contains(mv.src()) as usize,
            to_threat: threats.contains(mv.tgt()) as usize,
        }
    }
}
