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
//! search score, since there's too many positions that share the same pawn hash,
//! and the value would just get overwritten constantly and result in a bunch of
//! noise.
//!
//! Instead, we keep a running average: We take a weighted sum of the current
//! value and the newly provided delta, where we give more weight to the new 
//! delta if it corresponded to a higher depth search.
//!
//! NOTE: Would it make more sense to give higher weight to shallow searches? 
//! Those are clearly the ones that need more correction, because the eval got
//! it _very_ wrong.
use chess::piece::Color;
use crate::{evaluate::Score, zobrist::ZHash};

#[derive(Debug)]
pub struct CorrHistTable {
    table: [[CorrHistEntry; Self::SIZE]; Color::COUNT]
}

impl CorrHistTable {
    const SIZE: usize = 16_384;

    pub fn boxed() -> Box<Self> {
        #![allow(clippy::cast_ptr_alignment)]
        // SAFETY: we're allocating a zeroed block of memory, and then casting 
        // it to a Box<Self>. This is fine! 
        // [[CorrHistEntry; CORR_HIST_SIZE]; Color::COUNT] is just a bunch of i32s
        // in disguise, which are fine to zero-out.
        unsafe {
            let layout = std::alloc::Layout::new::<Self>();
            let ptr = std::alloc::alloc_zeroed(layout);
            if ptr.is_null() {
                std::alloc::handle_alloc_error(layout);
            }
            Box::from_raw(ptr.cast())
        }
    }

    /// Get a reference to the correction history entry for a given STM and
    /// pawn hash.
    pub fn get(&self, side: Color, hash: ZHash) -> &CorrHistEntry {
        &self.table[side][hash.0 as usize % Self::SIZE]
    }

    /// Get an exclusive reference to the correction history entry for a given 
    /// STM and pawn hash.
    pub fn get_mut(&mut self, side: Color, hash: ZHash) -> &mut CorrHistEntry {
        &mut self.table[side][hash.0 as usize % Self::SIZE]
    }
}

#[derive(Debug, Copy, Clone)]
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

    /// Correct the provided eval score with the value stored in the entry
    pub fn correct(&self, eval: Score) -> Score {
        eval + self.0 / Self::GRAIN
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
        let updated = (self.0 * old_weight + scaled_diff * new_weight) / Self::MAX_WEIGHT;

        self.0 = updated.clamp(-Self::MAX_VALUE, Self::MAX_VALUE);
    }
}
