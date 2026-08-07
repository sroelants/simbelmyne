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
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PVTable {
  /// The principal variation moves collected so far
  pv: [Move; MAX_DEPTH],

  /// The length, being the index of the last move stored in the array (+ 1).
  len: usize,
}

impl Default for PVTable {
  fn default() -> Self {
    Self {
      pv: [Default::default(); MAX_DEPTH],
      len: Default::default(),
    }
  }
}

impl PVTable {
  /// Create a new PV table
  pub fn new() -> Self {
    Self {
      pv: [Move::NULL; MAX_DEPTH],
      len: 0,
    }
  }

  pub fn from_parts(head: Move, tail: &Self) -> Self {
    let mut pv = Self::new();
    pv.len = tail.len + 1;
    pv.pv[0] = head;
    pv.pv[1..=pv.len].copy_from_slice(&tail.pv[0..=tail.len]);
    pv
  }

  /// Clear the PV table by re-setting its index.
  /// Note that we're not actually clearing any data here.
  pub fn clear(&mut self) {
    self.len = 0;
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
