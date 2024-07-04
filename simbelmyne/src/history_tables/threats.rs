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

    pub fn age_entries(&mut self) {
        for tables in self.tables {
            for mut table in tables {
                table.age_entries();
            }
        }
    }
}

impl Index<ThreatIndex> for ThreatsHistoryTable {
    type Output = HistoryTable;

    fn index(&self, index: ThreatIndex) -> &Self::Output {
        let from_threat = index.threats.contains(index.mv.src());
        let to_threat = index.threats.contains(index.mv.tgt());

        &self.tables[from_threat as usize][to_threat as usize]
    }
}

impl IndexMut<ThreatIndex> for ThreatsHistoryTable {
    fn index_mut(&mut self, index: ThreatIndex) -> &mut Self::Output {
        let from_threat = index.threats.contains(index.mv.src());
        let to_threat = index.threats.contains(index.mv.tgt());

        &mut self.tables[from_threat as usize][to_threat as usize]
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ThreatIndex {
    threats: Bitboard,
    mv: Move,
}

impl ThreatIndex {
    pub fn new(threats: Bitboard, mv: Move) -> Self {
        Self { threats, mv }
    }
}
