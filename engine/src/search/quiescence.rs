use chess::movegen::legal_moves::All;
use chess::movegen::moves::Move;

use super::params::*;
use super::SearchRunner;
use crate::evaluate::tuner::NullTracer;
use crate::evaluate::Eval;
use crate::evaluate::Score;
use crate::evaluate::ScoreExt;
use crate::move_picker::MovePicker;
use crate::position::Position;
use crate::transpositions::NodeType;
use crate::transpositions::TTEntry;

// Constants used for more readable const generics
const TACTICALS: bool = false;

impl<'a> SearchRunner<'a> {
  /// Perform a less intensive negamax search that only searches captures.
  ///
  /// This is to avoid horizon effects where we misjudge a position because
  /// we stopped the search abruptly at an inopportune time.
  ///
  /// The rough flow of this function is the same as `Position::negamax`, but
  /// we perform less pruning and hacks.
  pub fn quiescence_search<const PV: bool>(
    &mut self,
    pos: &Position,
    ply: usize,
    mut alpha: Score,
    beta: Score,
    mut eval_state: Eval,
  ) -> Score {
    if !self.tc.should_continue(self.nodes.local()) {
      self.aborted = true;
      return Score::MINUS_INF;
    }

    self.nodes.increment();
    self.seldepth = self.seldepth.max(ply);

    if pos.board.is_rule_draw() || pos.is_repetition() {
      return eval_state.draw_score(ply, self.nodes.local());
    }

    let in_check = pos.board.in_check();
    let tt_entry = self.tt.probe(pos.hash);
    let ttpv = PV || tt_entry.is_some_and(|entry| entry.get_ttpv());

    ////////////////////////////////////////////////////////////////////////
    //
    // Compute the static evaluation
    //
    // If the eval is _really_ good (>= beta), return it directly as a
    // "stand pat".
    //
    ////////////////////////////////////////////////////////////////////////

    let raw_eval = if in_check {
      // Precaution to make sure we don't miss mates
      -Score::MATE + ply as Score
    } else if let Some(entry) = tt_entry {
      entry.get_eval()
    } else {
      let eval = eval_state.total(&pos.board, &mut NullTracer);

      self.tt.insert(TTEntry::new(
        pos.hash,
        Move::NULL,
        Score::NO_SCORE,
        eval,
        0,
        NodeType::Upper,
        self.tt.get_age(),
        ttpv,
        ply,
      ));

      eval
    };

    let static_eval = if in_check {
      -Score::MATE + ply as Score
    } else {
      raw_eval + self.history.eval_correction(pos, ply)
    };

    if ply >= MAX_DEPTH {
      return static_eval;
    }

    if static_eval >= beta {
      return static_eval;
    }

    if alpha < static_eval {
      alpha = static_eval;
    }

    ////////////////////////////////////////////////////////////////////////
    //
    // Try and use the TT score
    //
    // Since _every_ score should technically stem from a QSearch (or a
    // draw/mate), we should be allowed to re-use TT scores.
    //
    ////////////////////////////////////////////////////////////////////////

    let tt_result =
      tt_entry.and_then(|entry| entry.try_score(0, alpha, beta, ply));

    if let Some(score) = tt_result {
      return score;
    }

    ////////////////////////////////////////////////////////////////////////
    //
    // Loop over all tacticals
    //
    ////////////////////////////////////////////////////////////////////////

    let tt_move = tt_entry.and_then(|entry| entry.get_move());

    let mut tacticals = MovePicker::new::<TACTICALS>(&pos, tt_move, ply);

    let mut best_move = tt_move;
    let mut best_score = static_eval;
    let mut node_type = NodeType::Upper;
    let mut move_count = 0;

    while let Some(mv) = tacticals.next(&self.history) {
      self.history.push_mv(mv, &pos.board);
      self.tt.prefetch(pos.approx_hash_after(mv));

      let next_position = pos.play_move(mv);

      let next_eval = eval_state.play_move(
        self.history.indices[ply],
        &next_position.board,
        next_position.kp_hash,
        &mut self.kp_cache,
      );

      let score = if move_count == 0 {
        -self.quiescence_search::<PV>(
          &next_position,
          ply + 1,
          -beta,
          -alpha,
          next_eval,
        )
      } else {
        -self.quiescence_search::<false>(
          &next_position,
          ply + 1,
          -beta,
          -alpha,
          next_eval,
        )
      };

      self.history.pop_mv();
      move_count += 1;

      if score > best_score {
        best_score = score;
      }

      if score >= beta {
        node_type = NodeType::Lower;
        best_move = Some(mv);
        break;
      }

      if score > alpha {
        alpha = score;
        best_move = Some(mv);
        node_type = NodeType::Exact;
      }

      if self.aborted {
        return Score::MINUS_INF;
      }
    }

    // If we're in check and there are no captures, we need to check
    // whether it might be mate!
    if in_check && move_count == 0 && pos.board.legal_moves::<All>().len() == 0
    {
      return -Score::MATE + ply as Score;
    }

    ////////////////////////////////////////////////////////////////////////
    //
    // Upate the search tables
    //
    // Store the best move and score, as well as whether or not the score
    // is an upper/lower bound, or exact.
    //
    ////////////////////////////////////////////////////////////////////////

    // Store in the TT
    self.tt.insert(TTEntry::new(
      pos.hash,
      best_move.unwrap_or(Move::NULL),
      best_score,
      raw_eval,
      0,
      node_type,
      self.tt.get_age(),
      ttpv,
      ply,
    ));

    best_score
  }
}
