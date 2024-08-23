use chess::movegen::legal_moves::All;
use chess::movegen::moves::Move;
use chess::piece::Color::*;

use crate::evaluate::tuner::NullTrace;
use crate::evaluate::Eval;
use crate::evaluate::ScoreExt;
use crate::move_picker::MovePicker;
use crate::position::Position;
use crate::evaluate::Score;
use crate::transpositions::NodeType;
use crate::transpositions::TTEntry;
use super::params::*;
use super::SearchThread;

// Constants used for more readable const generics
const TACTICALS: bool = false;

impl<'a> SearchThread<'a> {
    /// Perform a less intensive negamax search that only searches captures.
    ///
    /// This is to avoid horizon effects where we misjudge a position because
    /// we stopped the search abruptly at an inopportune time.
    ///
    /// The rough flow of this function is the same as `Position::negamax`, but 
    /// we perform less pruning and hacks.
    pub fn quiescence_search(
        &mut self, 
        pos: &Position,
        ply: usize,
        mut alpha: Score, 
        beta: Score, 
        mut eval_state: Eval,
    ) -> Score {
        if !self.tc.should_continue(self.nodes) {
            self.aborted = true;
            return Score::MINUS_INF;
        }

        self.nodes += 1;
        self.seldepth = self.seldepth.max(ply);

        if pos.board.is_rule_draw() || pos.is_repetition() {
            return eval_state.draw_score(ply, self.nodes);
        }

        let us = pos.board.current;
        let in_check = pos.board.in_check();
        let tt_entry = self.tt.probe(pos.hash);

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
            -Score::MATE + ply as Score } else if let Some(entry) = tt_entry {
            entry.get_eval()
        } else {
            // let idx = search.history.indices[ply-1];
            // let new_eval = eval_state.play_move(idx, &self.board);
            // search.stack[ply].incremental_eval = Some(new_eval);
            eval_state.total(&pos.board, &mut NullTrace)
        };

        let static_eval = if in_check {
            -Score::MATE + ply as Score
        } else {
            let pawn_correction = self.history.corr_hist
                .get(us, pos.pawn_hash)
                .corr();

            let w_nonpawn_correction = self.history.corr_hist
                .get(us, pos.nonpawn_hashes[White])
                .corr();

            let b_nonpawn_correction = self.history.corr_hist
                .get(us, pos.nonpawn_hashes[Black])
                .corr();

            let material_correction = self.history.corr_hist
                .get(us, pos.material_hash)
                .corr();

            raw_eval 
                + pawn_correction 
                + (w_nonpawn_correction + b_nonpawn_correction) / 2
                + 4 * material_correction
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

        let tt_result = tt_entry.and_then(|entry| {
            entry.try_score(0, alpha, beta, ply)
        });

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
            ////////////////////////////////////////////////////////////////////
            //
            // Delta/Futility pruning
            //
            // Take the current evaluation, add the material score of the 
            // would-be capture and an additional margin to account for any
            // positional gains. If this total score still can't beat alpha, 
            // don't even bother searching the move.
            //
            // ("If we're down a rook, don't bother trying to capture a pawn")
            //
            ////////////////////////////////////////////////////////////////////

            // let capture_value = self.board.get_at(mv.tgt())
            //     .map(|p| SEE_VALUES[p.piece_type()])
            //     .unwrap_or(0);
            //
            // let futility = static_eval 
            //     + capture_value 
            //     + delta_pruning_margin();
            //
            // if !in_check && futility <= alpha {
            //     continue;
            // }

            ////////////////////////////////////////////////////////////////////
            //
            // Play the move
            //
            // Play the move and recurse down the tree
            //
            ////////////////////////////////////////////////////////////////////
            self.history.push_mv(mv, &pos.board);
            self.tt.prefetch(pos.approx_hash_after(mv));

            let next_position = pos.play_move(mv);

            let next_eval = eval_state.play_move(
                self.history.indices[ply], 
                &next_position.board,
                next_position.pawn_hash,
                &mut self.pawn_cache
            );

            let score = -self
                .quiescence_search(
                    &next_position,
                    ply + 1, 
                    -beta, 
                    -alpha, 
                    next_eval,
                );

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
        if in_check 
            && move_count == 0
            && pos.board.legal_moves::<All>().len() == 0 {
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
            ply
        ));

        best_score
    }
}
