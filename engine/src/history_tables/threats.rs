use std::ops::{Index, IndexMut};
use chess::{bitboard::Bitboard, movegen::moves::Move};

#[derive(Debug)]
pub struct Threats<T> {
    tables: [[T; 2]; 2]
}

impl<T> Index<ThreatIndex> for Threats<T>{
    type Output = T;

    fn index(&self, idx: ThreatIndex) -> &Self::Output {
        &self.tables[idx.from_threat][idx.to_threat]
    }
}

impl<T> IndexMut<ThreatIndex> for Threats<T> {
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
