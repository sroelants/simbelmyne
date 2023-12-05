use std::ops::Deref;
use chess::square::Square;
use chess::piece::Piece;

use chess::movegen::moves::Move;

const MAX_KILLERS: usize = 2;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Killers([Move; MAX_KILLERS]);

impl Deref for Killers {
    type Target = [Move; MAX_KILLERS];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Killers {
    pub fn new() -> Self {
        Killers([Move::NULL; MAX_KILLERS])
    }

    pub fn add(&mut self, mv: Move) {
        // Make sure we only add distinct moves
        if !self.contains(&mv) {
            self.0.rotate_right(1);
            self.0[0] = mv;
        }
    }
}

pub struct KillersIter {
    killers: Killers,
    index: usize,
}

impl Iterator for KillersIter {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.killers.len() {
            return None;
        }

        let mv = self.killers.0[self.index];
        self.index += 1;

        if mv == Move::NULL {
            return None;
        }

        Some(mv)
    }
}


impl IntoIterator for Killers {
    type Item = Move;
    type IntoIter = KillersIter;

    fn into_iter(self) -> Self::IntoIter {
        KillersIter {
            killers: self,
            index: 0,
        }
    }
}

// History table
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct HistoryTable([[i32; Square::COUNT]; Piece::COUNT]);

impl HistoryTable {
    pub fn new() -> Self {
        HistoryTable([[0; Square::COUNT]; Piece::COUNT])
    }

    pub fn increment(&mut self, mv: &Move, piece: &Piece, depth: usize) {
        self.0[*piece as usize][mv.tgt() as usize] += (depth * depth) as i32;
    }

    pub fn get(&self, mv: &Move, piece: &Piece) -> i32 {
        self.0[*piece as usize][mv.tgt() as usize]
    }
}
