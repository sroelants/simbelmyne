use crate::search::params::MAX_DEPTH;
use chess::movegen::moves::Move;
use std::fmt::Display;

////////////////////////////////////////////////////////////////////////////////
//
// PV Table
//
////////////////////////////////////////////////////////////////////////////////

/// A PV table is a fixed length array and an index, and stores the principal
/// variation for a given node.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct PVTable {
  /// The principal variation moves collected so far
  pv: [Move; MAX_DEPTH],

  /// The length, being the index of the last move stored in the array (+ 1).
  len: usize,
}

impl PVTable {
  /// Create a new PV table
  pub fn new() -> Self {
    Self {
      pv: [Move::NULL; MAX_DEPTH],
      len: 0,
    }
  }

  /// Clear the PV table by re-setting its index.
  /// Note that we're not actually clearing any data here.
  pub fn clear(&mut self) {
    self.len = 0;
  }

  /// The main operation for the PV table: a PV node will try and prepend
  /// the PV it got from its children with its own PV move and pass it back
  /// up.
  pub fn add_to_front(&mut self, mv: Move, existing: &Self) {
    self.len = existing.len + 1;
    self.pv[0] = mv;
    self.pv[1..=self.len].copy_from_slice(&existing.pv[0..=existing.len]);
  }

  /// Return the PV moves as a slice
  pub fn moves(&self) -> &[Move] {
    &self.pv[..self.len]
  }

  /// Return "the" PV move, being the first move in the principal variation
  pub fn pv_move(&self) -> Move {
    self.moves()[0]
  }
}

impl Display for PVTable {
  /// Display the PV table as its UCI formatted string
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
