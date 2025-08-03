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

pub const CORRHIST_SIZE: usize = 16384;

impl History {
  pub fn eval_correction(&self, pos: &Position, ply: usize) -> Score {
    use Color::*;
    let us = pos.board.current;

    let pawn = self.pawn_corr[us][pos.pawn_hash].value();
    let w_nonpawn = self.w_nonpawn_corr[us][pos.nonpawn_hashes[White]].value();
    let b_nonpawn = self.b_nonpawn_corr[us][pos.nonpawn_hashes[Black]].value();
    let material = self.mat_corr[us][pos.material_hash].value();
    let minor = self.minor_corr[us][pos.minor_hash].value();

    let cont1 = self.indices.get(ply - 1)
      .map(|idx| self.contcorr_hist[us][*idx].value())
      .unwrap_or_default();

    let cont2 = self.indices.get(ply - 2)
      .map(|idx| self.contcorr_hist[us][*idx].value())
      .unwrap_or_default();

    let correction = pawn_corr_weight() * pawn
      + nonpawn_corr_weight() * w_nonpawn
      + nonpawn_corr_weight() * b_nonpawn
      + material_corr_weight() * material
      + minor_corr_weight() * minor
      + 256 * cont1
      + cont_corr_weight() * cont2;

    correction / (256 * CorrHistEntry::SCALE)
  }

  pub fn update_corrhist(
    &mut self,
    pos: &Position,
    ply: usize,
    depth: usize,
    diff: Score,
  ) {
    use Color::*;
    let us = pos.board.current;
    let corr = CorrHistEntry::new(diff);

    self.pawn_corr[us][pos.pawn_hash].update(corr, depth);
    self.w_nonpawn_corr[us][pos.nonpawn_hashes[White]].update(corr, depth);
    self.b_nonpawn_corr[us][pos.nonpawn_hashes[Black]].update(corr, depth);
    self.mat_corr[us][pos.material_hash].update(corr, depth);
    self.minor_corr[us][pos.minor_hash].update(corr, depth);

    if let Some(idx) = self.indices.get(ply - 1) {
      self.contcorr_hist[us][*idx].update(corr, depth);
    }

    if let Some(idx) = self.indices.get(ply - 2) {
      self.contcorr_hist[us][*idx].update(corr, depth);
    }
  }

  pub fn complexity(&mut self, pos: &Position, ply: usize) -> Score {
    use Color::*;
    let us = pos.board.current;

    let pawn = self.pawn_corr[us][pos.pawn_hash].value();
    let w_nonpawn = self.w_nonpawn_corr[us][pos.nonpawn_hashes[White]].value();
    let b_nonpawn = self.b_nonpawn_corr[us][pos.nonpawn_hashes[Black]].value();
    let material = self.mat_corr[us][pos.material_hash].value();
    let minor = self.minor_corr[us][pos.minor_hash].value();

    let cont1 = self.indices.get(ply - 1)
      .map(|idx| self.contcorr_hist[us][*idx].value())
      .unwrap_or_default();

    let cont2 = self.indices.get(ply - 2)
      .map(|idx| self.contcorr_hist[us][*idx].value())
      .unwrap_or_default();

    let squares = pawn * pawn +
        w_nonpawn * w_nonpawn +
        b_nonpawn * b_nonpawn +
        material * material +
        minor * minor +
        cont1 * cont1 +
        cont2 * cont2;

    let mean = squares / 7;
    let rms = Score::isqrt(mean);
    rms / (256 * CorrHistEntry::SCALE)
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
  const SCALE: Score = 256;
  const MAX: Score = 8192;
  const MAX_UPDATE: Score = 2048;

  pub fn new(corr: Score) -> Self {
    Self(Self::SCALE * corr)
  }

  pub fn value(&self) -> Score {
    self.0
  }

  pub fn update(&mut self, corr: Self, depth: usize) {
    let w = (depth + 1).min(16) as Score;

    let update = ((corr.0 - self.0) * w / 256)
        .clamp(-Self::MAX_UPDATE, Self::MAX_UPDATE);

    self.0 = (self.0 + update).clamp(-Self::MAX, Self::MAX);
  }
}
