use chess::movegen::moves::Move;

use crate::search::params::MAX_KILLERS;

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

