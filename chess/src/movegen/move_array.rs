use std::ops::Deref;
use std::ops::DerefMut;

use super::moves::Move;

#[derive(Debug, Copy, Clone)]
pub struct MoveArray {
  moves: [Move; Self::SIZE],
  len: usize,
}

impl MoveArray {
  pub const SIZE: usize = 218;

  pub fn new() -> Self {
    Self::default()
  }

  pub fn push(&mut self, mv: Move) {
    self.moves[self.len] = mv;
    self.len += 1;
  }

  pub fn clear(&mut self) {
    self.len = 0;
  }

  pub fn len(&self) -> usize {
    self.len
  }
}

impl Default for MoveArray {
  fn default() -> Self {
    Self {
      moves: [Move::default(); Self::SIZE],
      len: 0,
    }
  }
}

impl IntoIterator for MoveArray {
  type Item = Move;

  type IntoIter = IntoIter;

  fn into_iter(self) -> Self::IntoIter {
    Self::IntoIter {
      inner: self,
      idx: 0,
    }
  }
}

impl Deref for MoveArray {
  type Target = [Move];

  fn deref(&self) -> &Self::Target {
    &self.moves
  }
}

impl DerefMut for MoveArray {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.moves
  }
}

pub struct IntoIter {
  inner: MoveArray,
  idx: usize,
}

impl Iterator for IntoIter {
  type Item = Move;

  fn next(&mut self) -> Option<Self::Item> {
    if self.idx < self.inner.len {
      let mv = self.inner[self.idx];
      self.idx += 1;
      Some(mv)
    } else {
      None
    }
  }
}
