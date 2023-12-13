use std::fmt::Display;
use chess::square::Square;
use chess::piece::Piece;
use chess::movegen::moves::Move;
use crate::search::MAX_DEPTH;

const MAX_KILLERS: usize = 2;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct PVTable {
    pv: [Move; MAX_DEPTH],
    len: usize,
}

impl PVTable {
    pub fn new() -> Self {
        Self {
            pv: [Move::NULL; MAX_DEPTH],
            len: 0
        }
    }

    pub fn clear(&mut self) {
        self.len = 0;
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
        self.moves.len()
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

    pub fn increment(&mut self, mv: &Move, piece: Piece, depth: usize) {
        self.0[piece as usize][mv.tgt() as usize] += (depth * depth) as i32;
    }

    pub fn get(&self, mv: &Move, piece: Piece) -> i32 {
        self.0[piece as usize][mv.tgt() as usize]
    }
}
