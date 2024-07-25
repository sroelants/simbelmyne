use chess::piece::Color;
use crate::{evaluate::Score, search::params::hist_age_divisor, zobrist::ZHash};

const CORR_HIST_SIZE: usize = 16_384;
const CORR_HIST_GRAIN: Score = 256;
const CORR_HIST_WEIGHT_MAX: usize = 256;
const CORR_HIST_MAX_VALUE: Score = 32 * CORR_HIST_GRAIN;

#[derive(Debug)]
pub struct CorrHistTable {
    table: [[CorrHistEntry; CORR_HIST_SIZE]; 2]
}

impl CorrHistTable {
    pub fn boxed() -> Box<Self> {
        #![allow(clippy::cast_ptr_alignment)]
        // SAFETY: we're allocating a zeroed block of memory, and then casting 
        // it to a Box<Self>. This is fine! 
        // [[HistoryTable; Square::COUNT]; Piece::COUNT] is just a bunch of i16s
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

    pub fn age_entries(&mut self) {
        for color in 0..Color::COUNT  {
            for key in 0..CORR_HIST_SIZE {
                self.table[color][key].0 /= hist_age_divisor() as i32;
            }
        }
    }

    pub fn get(&self, side: Color, hash: ZHash) -> &CorrHistEntry {
        &self.table[side][hash.0 as usize % CORR_HIST_SIZE]
    }

    pub fn get_mut(&mut self, side: Color, hash: ZHash) -> &mut CorrHistEntry {
        &mut self.table[side][hash.0 as usize % CORR_HIST_SIZE]
    }
}

#[derive(Debug, Copy, Clone)]
pub struct CorrHistEntry(Score);

impl CorrHistEntry {
    /// Correct the provided eval score with the value stored in the entry
    pub fn correct(&self, eval: Score) -> Score {
        eval + self.0 / CORR_HIST_GRAIN

    }

    /// Update the entry with a given eval score delta
    ///
    /// Modify the old value to be a weighted sum of the old value and the
    /// delta of the best score and static eval.
    pub fn update(&mut self, best_score: Score, eval: Score, depth: usize) {
        // Scale the diff by the grain size
        let scaled_diff = (best_score - eval) * CORR_HIST_GRAIN;

        // The weights to give to the new and old entry, respectively
        let new_weight = usize::min(depth + 1, 16);
        let old_weight = CORR_HIST_WEIGHT_MAX - new_weight;

        // Take the weighted sum of the old value and the new
        let updated = (self.0 * old_weight as Score + scaled_diff * new_weight as Score) / CORR_HIST_WEIGHT_MAX as Score;

        self.0 = updated.clamp(-CORR_HIST_MAX_VALUE, CORR_HIST_MAX_VALUE);
    }
}
