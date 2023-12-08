use std::fmt::Display;
use std::ops::Deref;
use chess::square::Square;
use chess::piece::Piece;

use chess::movegen::moves::Move;

use crate::search::MAX_DEPTH;

const MAX_KILLERS: usize = 2;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct PVTable {
    pv: [Move; MAX_DEPTH],
    pub len: usize,
}

impl PVTable {
    pub fn new() -> Self {
        Self {
            pv: [Move::NULL; MAX_DEPTH],
            len: 0
        }
    }

    pub fn add_to_front(&mut self, mv: Move, existing: &Self) {
        self.len = existing.len + 1;
        self.pv[0] = mv;
        self.pv[1..=self.len].copy_from_slice(&existing.pv[0..=existing.len]);
    }

    pub fn moves(&self) -> &[Move] {
        &self.pv[..self.len]
    }

    pub fn pv_move(&self) -> Move {
        self.moves()[0]
    }
}

impl Display for PVTable {
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

impl From<PVTable> for Vec<Move> {
    fn from(value: PVTable) -> Self {
        value.pv[..value.len].to_vec()
    }
}


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
