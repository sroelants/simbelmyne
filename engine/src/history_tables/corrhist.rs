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

    let pawn_correction = self.corr_hist[us][pos.pawn_hash].corr();
    let w_nonpawn_correction =
      self.corr_hist[us][pos.nonpawn_hashes[White]].corr();
    let b_nonpawn_correction =
      self.corr_hist[us][pos.nonpawn_hashes[Black]].corr();
    let material_correction = self.corr_hist[us][pos.material_hash].corr();
    let minor_correction = self.corr_hist[us][pos.minor_hash].corr();
    let cont_correction = self
      .indices
      .get(ply - 2)
      .map(|idx| self.contcorr_hist[*idx].corr())
      .unwrap_or_default();

    let correction = pawn_corr_weight() * pawn_correction
      + nonpawn_corr_weight() * w_nonpawn_correction
      + nonpawn_corr_weight() * b_nonpawn_correction
      + material_corr_weight() * material_correction
      + minor_corr_weight() * minor_correction
      + cont_corr_weight() * cont_correction;

    correction / 256
  }

  pub fn update_corrhist(
    &mut self,
    pos: &Position,
    ply: usize,
    depth: usize,
    score: Score,
    eval: Score,
  ) {
    use Color::*;
    let us = pos.board.current;

    // Update the pawn corrhist
    self.corr_hist[us][pos.pawn_hash].update(score, eval, depth);

    // Update the non-pawn corrhist
    self.corr_hist[us][pos.nonpawn_hashes[White]].update(score, eval, depth);
    self.corr_hist[us][pos.nonpawn_hashes[Black]].update(score, eval, depth);

    // Update the material corrhist
    self.corr_hist[us][pos.material_hash].update(score, eval, depth);

    // Update the minor corrhist
    self.corr_hist[us][pos.minor_hash].update(score, eval, depth);

    // Update the cont corrhist
    if let Some(idx) = self.indices.get(ply - 2) {
      self.contcorr_hist[*idx].update(score, eval, depth);
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
  /// The granularity scale of the runnig average weighting.
  ///
  /// Any differences smaller than [GRAIN] due to the lerping will be
  /// indistinguishable.
  const GRAIN: Score = 256;

  /// The weight scale used for lerping (1 -> MAX_WEIGHT)
  const MAX_WEIGHT: Score = 256;

  /// The maximum value stored in a CorrHistEntry.
  ///
  /// Entries are clamped to lie between [-MAX_VALUE, MAX_VALUE].
  const MAX_VALUE: Score = 32 * Self::GRAIN;

  /// The maximal value by which we allow the corrhist entry to change in
  /// a single update
  const MAX_UPDATE: Score = Self::MAX_VALUE / 4;

  /// Correct the provided eval score with the value stored in the entry
  pub fn corr(&self) -> Score {
    self.0 / Self::GRAIN
  }

  /// Update the entry with a given eval score delta
  ///
  /// Modify the old value to be a weighted sum of the old value and the
  /// new delta of the best score and static eval.
  ///
  /// We artificially grow the diff by [GRAIN], and undo this scaling when
  /// applying the correction. This means there is a granularity to the
  /// mixing.
  pub fn update(&mut self, best_score: Score, eval: Score, depth: usize) {
    // Scale the diff by the grain size
    let scaled_diff = (best_score - eval) * Self::GRAIN;

    // The weights to give to the new and old entry, respectively
    let new_weight = (depth + 1).min(16) as Score;
    let old_weight = Self::MAX_WEIGHT - new_weight;

    // Take the weighted sum of the old value and the new
    let updated =
      (self.0 * old_weight + scaled_diff * new_weight) / Self::MAX_WEIGHT;

    self.0 = updated
      // Temper excessive updates by clamping to a reasonable range
      .clamp(self.0 - Self::MAX_UPDATE, self.0 + Self::MAX_UPDATE)
      // Clamp to max allowed value
      .clamp(-Self::MAX_VALUE, Self::MAX_VALUE);
  }
}
