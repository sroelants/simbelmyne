//! Aspiration window search
//!
//! This is a slight adaptation to the vanilla alpha-beta search.
//! Instead of starting the root alpha-beta search with a maximal
//! window (alpha = -Infinity, beta = +Infinity), we use information from
//! previous iterations of our Iterative Deepening loop to make a guess at
//! what the expected PV score will be. We then choose a narrow window around
//! that guesstimate which will lead to much quicker cutoffs.
//!
//! The downside is that if we the search finds a PV move and score that falls
//! outside of our estimate window, we need to re-search with a bigger window,
//! because who knows what we missed by picking such a narrow window.
//!
//! The hope, as always in these things, is that the score is stable enough that
//! re-searches are minimal, and the time we save in the best-case scenario
//! more than compensates for the odd re-search.
use super::SearchRunner;
use crate::evaluate::tuner::NullTracer;
use crate::evaluate::Eval;
use crate::evaluate::Score;
use crate::evaluate::ScoreExt;
use crate::history_tables::pv::PVTable;
use crate::position::Position;
use crate::search::params::*;

impl<'a> SearchRunner<'a> {
  /// Perform an alpha-beta search with aspiration window centered on `guess`.
  pub fn aspiration_search(
    &mut self,
    pos: &mut Position,
    guess: Score,
    pv: &mut PVTable,
  ) -> Score {
    let mut alpha = Score::MINUS_INF;
    let mut beta = Score::PLUS_INF;
    let mut width = aspiration_base_window();
    let mut reduction = 0;

    if self.depth >= aspiration_min_depth() {
      alpha = Score::max(Score::MINUS_INF, guess - width);
      beta = Score::min(Score::PLUS_INF, guess + width);
    }

    loop {
      let score = self.negamax::<true>(
        pos,
        0,
        self.depth - reduction,
        alpha,
        beta,
        pv,
        Eval::new(&pos.board, &mut NullTracer),
        false,
        false,
      );

      // If we fail low or high, grow the bounds upward/downward
      if score <= alpha {
        alpha -= width;

        // Optimization: grow the window _downward_ by dropping beta
        // as well.
        beta = (alpha + beta) / 2;

        // Reset the search depth to the original value
        reduction = 0;
      } else if score >= beta {
        beta += width;

        // Research at reduced depth
        reduction += 1;
      } else {
        return score;
      }

      // Grow the window (exponentially)
      width *= 2;

      // If the window exceeds the max width, give up and open the window
      // up completely.
      if width > aspiration_max_window() {
        alpha = Score::MINUS_INF;
        beta = Score::PLUS_INF;
      }

      if self.aborted {
        return Score::MINUS_INF;
      }
    }
  }
}
