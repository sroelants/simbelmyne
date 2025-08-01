//! Correction history
//!
//! Correction history keeps track of how well the static eval of a position
//! matched the returned search score. We can use this historic information to
//! correct the static eval in positions where the eval has been known to
//! approximate the search score poorly.
//!
//! In a sense, this is a softer variation on the idea of "use the search score
//! as an improved static eval". Using the search score as an improved eval
//! requires us to have a usable search score for the _exact_ position. The
//! correction table, however, is indexed by _pawn hash_, and so the entries
//! are shared across many different positions (since many different positions
//! share the same pawn hash).
//!
//! The value we store is not just the delta between the static eval and the
//! search score, since there's too many positions that share the same pawn
//! hash, and the value would just get overwritten constantly and result in a
//! bunch of noise.
//!
//! Instead, we keep a running average: We take a weighted sum of the current
//! value and the newly provided delta, where we give more weight to the new
//! delta if it corresponded to a higher depth search.
//!
//! NOTE: Would it make more sense to give higher weight to shallow searches?
//! Those are clearly the ones that need more correction, because the eval got
//! it _very_ wrong.

use chess::piece::Color;
use std::ops::Index;
use std::ops::IndexMut;

use crate::evaluate::Score;
use crate::position::Position;
use crate::search::params::cont_corr_weight;
use crate::search::params::material_corr_weight;
use crate::search::params::minor_corr_weight;
use crate::search::params::nonpawn_corr_weight;
use crate::search::params::pawn_corr_weight;
use crate::zobrist::ZHash;

use super::History;

pub const CORRHIST_SIZE: usize = 65536;

impl History {
  pub fn eval_correction(&self, pos: &Position, ply: usize) -> Score {
    use Color::*;
    let us = pos.board.current;

    let pawn_corr = self.corr_hist[us][pos.pawn_hash].value();
    let w_nonpawn_corr = self.corr_hist[us][pos.nonpawn_hashes[White]].value();
    let b_nonpawn_corr = self.corr_hist[us][pos.nonpawn_hashes[Black]].value();
    let material_corr = self.corr_hist[us][pos.material_hash].value();
    let minor_corr = self.corr_hist[us][pos.minor_hash].value();

    let cont_correction = self
      .indices
      .get(ply - 2)
      .map(|idx| self.contcorr_hist[*idx].value())
      .unwrap_or_default();

    let correction = pawn_corr_weight() * pawn_corr
      + nonpawn_corr_weight() * w_nonpawn_corr
      + nonpawn_corr_weight() * b_nonpawn_corr
      + material_corr_weight() * material_corr
      + minor_corr_weight() * minor_corr
      + cont_corr_weight() * cont_correction;

    correction / 4096
  }

  pub fn update_corrhist(
    &mut self,
    pos: &Position,
    ply: usize,
    depth: usize,
    delta: Score
  ) {
    use Color::*;
    let us = pos.board.current;
    let delta = CorrHistEntry::get_bonus(delta, depth);

    self.corr_hist[us][pos.pawn_hash].update(delta);
    self.corr_hist[us][pos.nonpawn_hashes[White]].update(delta);
    self.corr_hist[us][pos.nonpawn_hashes[Black]].update(delta);
    self.corr_hist[us][pos.material_hash].update(delta);
    self.corr_hist[us][pos.minor_hash].update(delta);

    if let Some(idx) = self.indices.get(ply - 2) {
      self.contcorr_hist[*idx].update(delta);
    }
  }
}

#[derive(Debug)]
pub struct Hash<T, const SIZE: usize> {
  table: [T; SIZE],
}

impl<T, const SIZE: usize> Index<ZHash> for Hash<T, SIZE> {
  type Output = T;

  fn index(&self, index: ZHash) -> &Self::Output {
    &self.table[index.0 as usize % SIZE]
  }
}

impl<T, const SIZE: usize> IndexMut<ZHash> for Hash<T, SIZE> {
  fn index_mut(&mut self, index: ZHash) -> &mut Self::Output {
    &mut self.table[index.0 as usize % SIZE]
  }
}

#[derive(Debug, Copy, Clone, Default)]
pub struct CorrHistEntry(Score);

impl CorrHistEntry {
  const MAX_VALUE: Score = 2048;
  const MAX_DELTA: Score = 256;

  pub fn get_bonus(diff: Score, depth: usize) -> Score {
    (diff * depth as Score / 8).clamp(-Self::MAX_DELTA, Self::MAX_DELTA)
  }

  pub fn update(&mut self, delta: Score) {
    self.0 += delta - self.0 * delta.abs() / Self::MAX_VALUE;
  }

  pub fn value(&self) -> Score {
    self.0
  }
}
