use crate::evaluate::tuner::NullTrace;
use crate::evaluate::Eval;
use crate::history_tables::history::HistoryScore;
use crate::history_tables::pv::PVTable;
use crate::move_picker::Stage;
use crate::transpositions::NodeType;
use crate::transpositions::TTEntry;
use crate::evaluate::ScoreExt;
use crate::move_picker::MovePicker;
use crate::position::Position;
use crate::evaluate::Score;
use chess::movegen::legal_moves::MoveList;
use chess::movegen::moves::Move;
use chess::movegen::moves::MoveType;
use chess::piece::Color::*;

use super::params::*;
use super::params::lmr_reduction;
use super::params::MAX_DEPTH;
use super::SearchRunner;

const ALL_MOVES: bool = true;

impl<'a> SearchRunner<'a> {
    /// The main negamax function of the search routine.
    pub fn negamax<const PV: bool>(
        &mut self, 
        pos: &Position,
        ply: usize, 
        mut depth: usize,
        alpha: Score, 
        beta: Score, 
        pv: &mut PVTable,
        mut eval_state: Eval,
        try_null: bool,
        cutnode: bool,
    ) -> Score {
        if self.aborted {
            return Score::MINUS_INF;
        }

        let us = pos.board.current;
        let in_root = ply == 0;
        let excluded = self.stack[ply].excluded;

        // Carry over the current count of double extensions
        if ply > 0 {
            self.stack[ply].double_exts = self.stack[ply-1].double_exts;
        }

        ///////////////////////////////////////////////////////////////////////
        //
        // Check extension: 
        //
        // If we're in check, make sure we always search at least one extra ply
        //
        ///////////////////////////////////////////////////////////////////////

        let in_check = pos.board.in_check(); 

        if in_check {
            depth += 1;
        }

        ////////////////////////////////////////////////////////////////////////
        //
        // Quiescence search: 
        //
        // If we're in a leaf node, extend with a quiescence search
        //
        ////////////////////////////////////////////////////////////////////////

        if depth == 0 || ply >= MAX_DEPTH {
            return self.quiescence_search(
                &pos,
                ply, 
                alpha, 
                beta, 
                eval_state
            );
        }

        ////////////////////////////////////////////////////////////////////////
        //
        // Start processing node
        //
        ////////////////////////////////////////////////////////////////////////

        self.nodes.increment();

        // Do all the static evaluations first
        // That is, Check whether we can/should assign a score to this node
        // without recursing any deeper.

        // Rule-based draw? 
        // Don't return early when in the root node, because we won't have a PV 
        // move to play.
        if !in_root && (pos.board.is_rule_draw() || pos.is_repetition()) {
            return eval_state.draw_score(ply, self.nodes.local());
        }

        ////////////////////////////////////////////////////////////////////////
        //
        // TT cutoffs
        //
        // Check the TT table for a result that we can use, and return it.
        // Attempt to use the score from the TT entry. This may or may not
        // work, depending on the current alpha/beta, and whether the 
        // stored score is an upper/lower bound.
        //
        ////////////////////////////////////////////////////////////////////////

        let tt_entry = if excluded.is_none() { 
            self.tt.probe(pos.hash) 
        } else { 
            None 
        };

        let tt_move = tt_entry.and_then(|entry| entry.get_move());

        if !PV && !in_root && tt_entry.is_some() {
            let tt_entry = tt_entry.unwrap();

            // Can we use the stored score?
            if let Some(score) = tt_entry.try_score(depth, alpha, beta, ply) {
                return score;
            }
        }

        ////////////////////////////////////////////////////////////////////////
        //
        // Compute the static evaluation
        //
        // Try and get the static evaluation from the TT entry, if possible.
        //
        ////////////////////////////////////////////////////////////////////////

        let raw_eval = if excluded.is_some() {
            // In singular search, we're not going to be using/storing the
            // raw eval, so we can use whatever.
            Score::MINUS_INF
        } else if let Some(entry) = tt_entry {
            entry.get_eval()
        } else {
            let eval = eval_state.total(&pos.board, &mut NullTrace);

            self.tt.insert(TTEntry::new(
                pos.hash,
                Move::NULL,
                Score::NO_SCORE,
                eval,
                0,
                NodeType::Upper,
                self.tt.get_age(),
                ply
            ));

            eval
        };

        let static_eval = if excluded.is_some() {
            self.stack[ply].eval
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

            let minor_correction = self.history.corr_hist
                .get(us, pos.minor_hash)
                .corr();

            raw_eval 
                + pawn_correction 
                + (w_nonpawn_correction + b_nonpawn_correction) / 2
                + 4 * material_correction
                + minor_correction
        };

        // Store the eval in the search stack
        self.stack[ply].eval = static_eval;

        ////////////////////////////////////////////////////////////////////////
        //
        // Clear the next ply's killers table
        //
        // In order to make the killer moves stored in the killers table more
        // relevant, we clear the killers table for the upcoming ply, so we're
        // guaranteed that all of our child nodes will only see killers that
        // come directly from their siblings.
        //
        ////////////////////////////////////////////////////////////////////////

        self.history.clear_killers(ply + 1);

        ////////////////////////////////////////////////////////////////////////
        //
        // Improving heuristic:
        //
        // If our eval is better than two plies ago, we can
        // 1. More aggressively prune fail-high based pruning/reductions (rfp, 
        //    nmp, etc...)
        // 2. Be more cautious with fail-low based pruning/reductions 
        //    (fp, alpha-based reductions, etc...)
        //
        ////////////////////////////////////////////////////////////////////////

        let improving = !in_check 
            && ply >= 2 
            && self.stack[ply - 2].eval < static_eval;

        ////////////////////////////////////////////////////////////////////////
        //
        // Reverse futility pruning
        //
        // If we're close to the max depth of the search, and the static 
        // evaluation board is some margin above beta, assume it's highly 
        // unlikely for the search _not_ to end in a cutoff. Instead, just 
        // return a compromise value between the current eval and beta.
        //
        ////////////////////////////////////////////////////////////////////////

        let futility = rfp_margin() * depth as Score
            + rfp_improving_margin() * !improving as Score;

        if !PV 
            && !in_root
            && !in_check
            && excluded.is_none()
            && depth <= rfp_threshold()
            && static_eval - futility >= beta {
            return (static_eval + beta) / 2;
        }

        ////////////////////////////////////////////////////////////////////////
        //
        // Null move pruning
        //
        // Pretend to play a NULL move and do a search at reduced depth (so 
        // shouldn't be too expensive) and a really narrow window. If, after 
        // that, we _still_ get a beta cutoff, our position was so good we 
        // shouldn't bother searching it any further
        //
        ////////////////////////////////////////////////////////////////////////
        let nmp_margin = nmp_base_margin() 
            + nmp_margin_factor() * depth as Score 
            + nmp_improving_margin() * improving as Score;

        let should_null_prune = try_null
            && !PV
            && !in_root
            && !in_check
            && excluded.is_none()
            && static_eval + nmp_margin >= beta
            && pos.board.zugzwang_unlikely();

        if should_null_prune {
            let reduction = (nmp_base_reduction() + depth / nmp_reduction_factor())
                .min(depth);

            self.history.push_null_mv();

            let score = -self
                .zero_window(
                    &pos.play_null_move(),
                    ply + 1, 
                    depth - reduction,
                    -beta + 1, 
                    &mut PVTable::new(), 
                    eval_state,
                    false,
                    !cutnode
                );

            self.history.pop_mv();

            if score >= beta {
                return score;
            }
        }

        ////////////////////////////////////////////////////////////////////////
        //
        // Internal Iterative Reduction
        //
        // If we didn't get a TT hit, reduce the depth by one so we waste less
        // time in this iteration, and populate the TT for the next iteration
        // instead.
        //
        ////////////////////////////////////////////////////////////////////////

        if tt_move.is_none() && (PV || cutnode) && depth >= iir_threshold() {
            depth -= iir_reduction();
        }

        ////////////////////////////////////////////////////////////////////////
        //
        // Generate the legal moves and do some static checks to see whether 
        // we can prune, or bail altogether.
        //
        ////////////////////////////////////////////////////////////////////////

        let mut legal_moves = MovePicker::new::<ALL_MOVES>(
            &pos,  
            tt_move,
            ply
        );

        ////////////////////////////////////////////////////////////////////////
        //
        // Singular extensions (Part 1)
        //
        // If a move proves to be much better than all the other moves, we
        // extend the search depth for this move.
        //
        // We consider a move a candidate for singular extension when
        // 1. It is a TT-move
        // 2. The associated TT entry is an exact- or lower-bound entry
        // 3. The entry depth is not more than 3 ply shallower than our search
        //
        ////////////////////////////////////////////////////////////////////////

        let se_candidate = tt_entry.filter(|entry| {
            depth >= se_threshold()
            && !in_root 
            && excluded.is_none() 
            && entry.get_type() != NodeType::Upper
            && entry.get_depth() >= depth - se_tt_delta()
            && !entry.get_score().is_mate()
        }).and_then(|entry| entry.get_move());


        ////////////////////////////////////////////////////////////////////////
        //
        // Iterate over the remaining moves
        //
        ////////////////////////////////////////////////////////////////////////

        let mut move_count = 0;
        let mut quiets_tried = MoveList::new();
        let mut tacticals_tried = MoveList::new();
        let mut best_move = tt_move;
        let mut best_score = Score::MINUS_INF;
        let mut node_type = NodeType::Upper;
        let mut alpha = alpha;
        let mut local_pv = PVTable::new();

        while let Some(mv) = legal_moves.next(&self.history) {
            if Some(mv) == excluded {
                continue;
            }

            local_pv.clear();

            if !self.tc.should_continue(self.nodes.local()) {
                self.aborted = true;
                return Score::MINUS_INF;
            }

            let lmr_depth = usize::max(0, depth - lmr_reduction(depth, move_count));

            ////////////////////////////////////////////////////////////////////////
            //
            // Futility pruning
            // 
            // If we're near the end of the search, and the static evaluation of 
            // this node is lower than alpha by some margin, we prune away moves 
            // that are unlikely to be able to increase alpha. (i.e., quiet moves).
            //
            ////////////////////////////////////////////////////////////////////////

            let futility = fp_base()
                + fp_margin() * (lmr_depth as Score)
                + 100 * improving as Score;

            if move_count > 0 
                && !PV
                && !in_check
                && lmr_depth <= fp_threshold()
                && static_eval + futility < alpha {
                legal_moves.only_good_tacticals = true;
                continue;
            }

            ////////////////////////////////////////////////////////////////////
            //
            // SEE pruning
            //
            // For quiet moves and bad captures, if the Static Exchange Eval
            // comes out really bad, prune the move.
            //
            ////////////////////////////////////////////////////////////////////

            if legal_moves.stage() > Stage::GoodTacticals
                && (mv.is_tactical() || mv.get_type() == MoveType::Quiet)
                && move_count > 0
                && !in_root
                && !best_score.is_mate() {

                let margin = if mv.get_type() == MoveType::Quiet {
                    -see_quiet_margin() * depth as Score
                } else {
                    -see_tactical_margin() * depth as Score
                };

                if !pos.board.see(mv, margin) {
                    continue;
                }
            }

            ////////////////////////////////////////////////////////////////////
            //
            // Late move pruning
            //
            // Assuming good move ordering, the later moves in the list  are 
            // likely to be less interesting, especially as we approach the 
            // leaf nodes. After a (depth dependent) number of moves, start 
            // skipping quiet moves.
            //
            ////////////////////////////////////////////////////////////////////

            let lmp_moves = (lmp_base()
                + lmp_factor() * depth * depth) / (1 + !improving as usize);

            if depth <= lmp_threshold()
                && !PV
                && !in_check
                && move_count >= lmp_moves {
                legal_moves.only_good_tacticals = true;
            }

            ////////////////////////////////////////////////////////////////////
            //
            // History pruning
            //
            // We skip quiet moves with a sufficiently bad history score
            //
            ////////////////////////////////////////////////////////////////////

            let hp_margin = if mv.is_quiet() {
                quiet_hp_offset() + quiet_hp_margin() * depth as i32
            } else {
                tactical_hp_offset() + tactical_hp_margin() * depth as i32
            };

            if !in_check
                && !PV
                && !best_score.is_mate()
                && depth <= hp_threshold()
                && legal_moves.current_score() <= hp_margin {
                if mv.is_quiet() {
                    legal_moves.skip_quiets();
                }
                continue;
            }


            ////////////////////////////////////////////////////////////////////
            //
            // Singular extensions (Part 2)
            //
            // If there is a candidate SE move, we do a verification search,
            // where we perform a zero-window search on this same position with 
            // the candidate excluded, at reduced depth and centered around the
            // candidate move's TT score (minus a margin M, to make sure the 
            // candidate is _better_ by some margin M)
            //
            // NOTE: We're expecting/hoping that this ZW search will fail-low.
            // Because we're using fail-soft, we'll actually get an upper bound
            // score back, so we have an estimate of _by how much_ the move is
            // better than all the others. This will help up do fancy things 
            // like extend more if the fail-soft score is a lot lower.
            //
            ////////////////////////////////////////////////////////////////////

            let mut extension: i16 = 0;

            if se_candidate == Some(mv) {
                let mut local_pv = PVTable::new();
                let tt_score = tt_entry.unwrap().get_score();

                let se_depth = (depth - 1) / 2;
                let se_beta = Score::max(
                    tt_score - se_margin() * depth as Score,
                    -Score::MATE
                );

                // Do a verification search with the candidate move excluded.
                self.stack[ply].excluded = se_candidate;
                let value = self.zero_window(
                    &pos,
                    ply, 
                    se_depth, 
                    se_beta, 
                    &mut local_pv, 
                    eval_state,
                    try_null,
                    cutnode,
                );
                self.stack[ply].excluded = None;

                // If every other move is significantly less good, extend the 
                // SE Candidate move
                if value < se_beta {
                    extension += 1;

                    // Double extensions:
                    // If we're below the threshold by a lot, reduce by another 
                    // ply Make sure to keep the total number of double 
                    // extensions limited, though.
                    if !PV 
                    && value + double_ext_margin() < se_beta 
                    && self.stack[ply].double_exts <= double_ext_max() {
                        extension += 1;
                        self.stack[ply].double_exts += 1;

                        // Triple extensions:
                        // If the tt move is quiet (and otherwise unexpected to 
                        // be amazing), but beats se_beta by a _large_ margin,
                        // extend once more!
                        if mv.is_quiet() && value < se_beta - triple_ext_margin() {
                          extension += 1;
                        }
                    } 
                }


                ////////////////////////////////////////////////////////////////
                //
                // Multicut pruning:
                //
                // If the SE search failed high, there's more than one good 
                // move. If both it and the SE  candidate beat the search's 
                // `beta`, just assume this node will be a cutnode and return 
                // early.
                //
                // Note that this a guess, because both the TT score and the
                // SE search return scores from shallower depths, and `se_beta`
                // is _less_ than beta. Still, it's likely that both moves 
                // will produce a cutoff at the full search depth.
                //
                // NOTE: An alternative formulation would be:
                // if tt_score >= beta && value >= beta? That's slightly less
                // aggressive, though?
                //
                ////////////////////////////////////////////////////////////////
                else if se_beta >= beta {
                    return se_beta;
                }

                ////////////////////////////////////////////////////////////////
                //
                // Negative extensions
                //
                // A softer version of multicut:
                // If the TT score beats the search beta, and the SE search 
                // failed high, but not high enough to beat the search beta,
                // we assume that at full depth we'd probably find another
                // move that causes a cutoff and there's no point searching the
                // TT move quite as deeply.
                //
                ////////////////////////////////////////////////////////////////

                else if tt_score >= beta {
                    extension -= 1;
                }
            }

            ////////////////////////////////////////////////////////////////////
            //
            // Late move reductions
            //
            // Assuming good move ordering, we can search later moves at reduced
            // depth, reducing extra on less interesting moves, like quiets and
            // non-pv moves.
            //
            ////////////////////////////////////////////////////////////////////

            let mut score;
            self.history.push_mv(mv, &pos.board);
            let nodes_before = self.nodes.local();

            // Instruct the CPU to load the TT entry into the cache ahead of time
            self.tt.prefetch(pos.approx_hash_after(mv));

            let next_position = pos.play_move(mv);

            let next_eval = eval_state.play_move(
                self.history.indices[ply], 
                &next_position.board,
                next_position.pawn_hash,
                &mut self.pawn_cache
            );

            // PV Move
            if move_count == 0 {
                score = -self
                    .negamax::<PV>(
                        &next_position,
                        ply + 1, 
                        (depth as i16 + extension - 1) as usize, 
                        -beta, 
                        -alpha,
                        &mut local_pv, 
                        next_eval,
                        false,
                        !(PV || cutnode)
                    );

            // Search other moves with null-window, and open up window if a move
            // increases alpha
            } else {
                let mut reduction: i16 = 0;

                // Calculate LMR reduction
                if depth >= lmr_min_depth()
                    && move_count >= lmr_threshold() + PV as usize {
                    // Fetch the base LMR reduction value from the LMR table
                    reduction = lmr_reduction(depth, move_count) as i16;

                    // Reduce quiets and bad tacticals more
                    reduction += (legal_moves.stage() > Stage::GoodTacticals) as i16;

                    // Reduce bad captures even more
                    reduction += (legal_moves.stage() > Stage::Quiets) as i16;

                    // Reduce more if the TT move is a tactical
                    reduction += tt_move.is_some_and(|mv| mv.is_tactical()) as i16;

                    // Reduce more in expected cutnodes
                    reduction += cutnode as i16;

                    // Reduce non-pv nodes more
                    reduction -= PV as i16;

                    // Reduce less when the current position is in check
                    reduction -= in_check as i16;

                    // Reduce less when the move gives check
                    reduction -= next_position.board.in_check() as i16;


                    // Reduce moves with good history less, with bad history more
                    if mv.is_quiet() {
                        reduction -= (legal_moves.current_score() / hist_lmr_divisor()) as i16;
                    }

                    // Make sure we don't reduce below zero
                    reduction = reduction.clamp(0, depth as i16 - 1);
                }

                // Search with zero-window at reduced depth
                score = -self.zero_window(
                    &next_position,
                    ply + 1, 
                    (depth as i16 - 1 + extension - reduction).max(0) as usize, 
                    -alpha, 
                    &mut local_pv, 
                    next_eval,
                    true,
                    true,

                );

                // If score > alpha, but we were searching at reduced depth,
                // do a full-depth, zero-window search
                if score > alpha && reduction > 0 {
                    score = -self.zero_window(
                        &next_position,
                        ply + 1, 
                        (depth as i16 + extension - 1).max(0) as usize, 
                        -alpha, 
                        &mut local_pv, 
                        next_eval,
                        true,
                        !cutnode,
                    );
                }

                // If we still find score > alpha, re-search at full-depth *and*
                // full-window
                if score > alpha && score < beta {
                    score = -self.negamax::<PV>(
                        &next_position,
                        ply + 1, 
                        (depth as i16 + extension - 1).max(0) as usize, 
                        -beta, 
                        -alpha,
                        &mut local_pv, 
                        next_eval,
                        false,
                        !(PV || cutnode),
                    );
                }
            }

            self.history.pop_mv();
            move_count += 1;

            // Update the nodecount spent on this move
            if in_root {
                self.history.add_nodes(mv, self.nodes.local() - nodes_before);
            }

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
                node_type = NodeType::Exact;
                best_move = Some(mv);
                pv.add_to_front(mv, &local_pv);
            }

            // Fail-low moves get marked for history score penalty
            if score < alpha && mv.is_quiet() {
                quiets_tried.push(mv);
            }

            // Tacticals that don't cause a cutoff are always penalized
            if mv.is_tactical() {
                tacticals_tried.push(mv);
            }

            if self.aborted {
                return Score::MINUS_INF;
            }
        }

        // Checkmate?
        if move_count == 0 && excluded.is_some() {
            return alpha;
        }

        if move_count == 0 && in_check {
            return -Score::MATE + ply as Score;
        }

        // Stalemate?
        if move_count == 0 && !in_check {
            return eval_state.draw_score(ply, self.nodes.local());
        }


        ////////////////////////////////////////////////////////////////////////
        //
        // Upate the History tables
        //
        // If a quiet move exceeded beta, update the history tables:
        // - Store the move in the Killers table
        // - Store the move in the Countermove table
        // - Increment the move's score in the history and continuation history
        // - Decrement all preceding quiets that failed to beat beta in both 
        //   history tables
        //
        ////////////////////////////////////////////////////////////////////////

        if node_type == NodeType::Lower {
            let best_move = best_move.unwrap();
            let bonus = HistoryScore::bonus(depth);

            ////////////////////////////////////////////////////////////////////
            //
            // Upate the Quiet history tables
            //
            ////////////////////////////////////////////////////////////////////

            if best_move.is_quiet() {
                // New history table
                self.history.add_hist_bonus(best_move, &pos.board, bonus);
                self.history.add_killer(ply, best_move);
                self.history.add_countermove(best_move);

                // Deduct penalty for all tried quiets that didn't fail high
                for mv in quiets_tried {
                    self.history.add_hist_bonus(mv, &pos.board, -bonus);
                } 
            }

            ////////////////////////////////////////////////////////////////////
            //
            // Upate the Tactical history tables
            //
            ////////////////////////////////////////////////////////////////////

            // Add a bonus for the move that caused the cutoff
            else {
                self.history.add_hist_bonus(best_move, &pos.board, bonus);
            } 

            // Deduct a penalty from all tacticals that didn't cause a cutoff
            for mv in tacticals_tried {
                self.history.add_hist_bonus(mv, &pos.board, -bonus);
            }
        }

        if excluded.is_none() {
            ///////////////////////////////////////////////////////////////////
            //
            // Upate the Correction history
            //
            // Keep track of how big the difference between static eval and 
            // search score is if:
            // 1. We're not in check (so we have a valid static eval)
            // 2. We have no best move (fail-low), or the best move is a quiet 
            //    move
            // 3. The score is valid to use when it comes to the node bounds:
            //    3.a) If the score is >= eval and the score is _not_ an upper 
            //         bound
            //    3.b) If the score is <= eval and the score is _not_ a lower 
            //         bound
            //
            ///////////////////////////////////////////////////////////////////

            if !in_check
                && !best_move.is_some_and(|mv| mv.is_tactical())
                && !(node_type == NodeType::Lower && best_score <= static_eval)
                && !(node_type == NodeType::Upper && best_score >= static_eval) 
            {
                // Update the pawn corrhist
                self.history.corr_hist
                    .get_mut(us, pos.pawn_hash)
                    .update(best_score, static_eval, depth);

                // Update the non-pawn corrhist
                self.history.corr_hist
                    .get_mut(us, pos.nonpawn_hashes[White])
                    .update(best_score, static_eval, depth);

                self.history.corr_hist
                    .get_mut(us, pos.nonpawn_hashes[Black])
                    .update(best_score, static_eval, depth);

                // Update the material corrhist
                self.history.corr_hist
                    .get_mut(us, pos.material_hash)
                    .update(best_score, static_eval, depth);

                // Update the minor corrhist
                self.history.corr_hist
                    .get_mut(us, pos.minor_hash)
                    .update(best_score, static_eval, depth);
            }

            ///////////////////////////////////////////////////////////////////
            //
            // Upate the TT
            //
            // Store the best move and score, as well as whether or not the 
            // score is an upper/lower bound, or exact.
            //
            ///////////////////////////////////////////////////////////////////

            self.tt.insert(TTEntry::new(
                pos.hash,
                best_move.unwrap_or(Move::NULL),
                best_score,
                raw_eval,
                depth,
                node_type,
                self.tt.get_age(),
                ply
            ));
        }

        best_score
    }
}
