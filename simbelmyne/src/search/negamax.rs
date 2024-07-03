use crate::history_tables::history::HistoryIndex;
use crate::history_tables::history::HistoryScore;
use crate::history_tables::pv::PVTable;
use crate::move_picker::Stage;
use crate::transpositions::NodeType;
use crate::transpositions::TTEntry;
use crate::evaluate::ScoreExt;
use crate::transpositions::TTable;
use crate::move_picker::MovePicker;
use crate::position::Position;
use crate::evaluate::Score;
use chess::movegen::legal_moves::MoveList;
use chess::movegen::moves::Move;
use chess::piece::PieceType;

use super::params::lmr_reduction;
use super::Search;
use super::params::IIR_THRESHOLD;
use super::params::MAX_DEPTH;

const ALL_MOVES: bool = true;

impl Position {
    /// The main negamax function of the search routine.
    pub fn negamax<const PV: bool>(
        &self, 
        ply: usize, 
        mut depth: usize,
        alpha: Score, 
        beta: Score, 
        tt: &mut TTable, 
        pv: &mut PVTable,
        search: &mut Search,
        try_null: bool,
    ) -> Score {
        if search.aborted {
            return Score::MINUS_INF;
        }

        let in_root = ply == 0;

        ///////////////////////////////////////////////////////////////////////
        //
        // Check extension: 
        //
        // If we're in check, make sure we always search at least one extra ply
        //
        ///////////////////////////////////////////////////////////////////////

        let in_check = self.board.in_check();

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
            return self.quiescence_search(ply, alpha, beta, tt, search);
        }

        ////////////////////////////////////////////////////////////////////////
        //
        // Start processing node
        //
        ////////////////////////////////////////////////////////////////////////

        search.tc.add_node();

        // Do all the static evaluations first
        // That is, Check whether we can/should assign a score to this node
        // without recursing any deeper.

        // Rule-based draw? 
        // Don't return early when in the root node, because we won't have a PV 
        // move to play.
        if !in_root && (self.board.is_rule_draw() || self.is_repetition()) {
            return self.score.draw_score(ply, search.tc.nodes());
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

        let tt_entry = tt.probe(self.hash);
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
        
        let eval = if let Some(entry) = tt_entry {
            entry.get_eval()
        } else {
            self.score.total(&self.board)
        };

        // Store the eval in the search stack
        search.stack[ply].eval = eval;

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

        search.killers[ply + 1].clear();


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
            && search.stack[ply - 2].eval < eval;

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

        let futility = search.search_params.rfp_margin * depth as Score
            + search.search_params.rfp_improving_margin* !improving as Score;

        if !PV 
            && !in_root
            && !in_check
            && depth <= search.search_params.rfp_threshold
            && eval - futility >= beta {
            return (eval + beta) / 2;
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

        let should_null_prune = try_null
            && !PV
            && !in_root
            && !in_check
            && eval + search.search_params.nmp_improving_margin * improving as Score >= beta
            && self.board.zugzwang_unlikely();

        if should_null_prune {
            let reduction = (search.search_params.nmp_base_reduction + depth / search.search_params.nmp_reduction_factor)
                .min(depth);

            let score = -self
                .play_null_move()
                .zero_window(
                    ply + 1, 
                    depth - reduction,
                    -beta + 1, 
                    tt, 
                    &mut PVTable::new(), 
                    search, 
                    false
                );

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

        if tt_move.is_none() && !in_root && depth >= IIR_THRESHOLD {
            depth -= 1;
        }

        ////////////////////////////////////////////////////////////////////////
        //
        // Generate the legal moves and do some static checks to see whether 
        // we can prune, or bail altogether.
        //
        ////////////////////////////////////////////////////////////////////////
        let oneply_hist_idx = ply
            .checked_sub(1)
            .map(|prev_ply| search.stack[prev_ply].history_index);

        let twoply_hist_idx = ply
            .checked_sub(2)
            .map(|pprev_ply| search.stack[pprev_ply].history_index);

        let countermove = oneply_hist_idx.and_then(|idx| search.countermoves[idx]);

        let mut legal_moves = MovePicker::<ALL_MOVES>::new(
            &self,  
            tt_move,
            search.killers[ply],
            countermove
        );

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

        let oneply_conthist = oneply_hist_idx
            .map(|prev_idx| search.conthist_table[prev_idx]);

        let twoply_conthist = twoply_hist_idx
            .map(|pprev_idx| search.conthist_table[pprev_idx]);

        while let Some(mv) = legal_moves.next(
            &search.history_table, 
            &search.tactical_history,
            oneply_conthist.as_ref(),
            twoply_conthist.as_ref()
        ) {
            local_pv.clear();
            let is_quiet = mv.is_quiet();

            if !search.tc.should_continue() {
                search.aborted = true;
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
            let futility = search.search_params.fp_base 
                + search.search_params.fp_margin * lmr_depth as Score;

            if move_count > 0 
                && !PV
                && !in_check
                && lmr_depth <= search.search_params.fp_threshold
                && eval + futility < alpha {
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

            let see_margin = search.search_params.see_quiet_margin * depth as Score;

            if legal_moves.stage() > Stage::GoodTacticals
                && is_quiet
                && move_count > 0
                && !in_root
                && !best_score.is_mate()
                && !self.board.see(mv, see_margin) {
                continue;
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

            let lmp_moves = (search.search_params.lmp_base 
                + search.search_params.lmp_factor * depth * depth) / (1 + !improving as usize);

            if depth <= search.search_params.lmp_threshold
                && !PV
                && !in_check
                && move_count >= lmp_moves {
                legal_moves.only_good_tacticals = true;
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
            search.stack[ply].history_index = HistoryIndex::new(&self.board, mv);
            let next_position = self.play_move(mv);

            // Instruct the CPU to load the TT entry into the cache ahead of time
            tt.prefetch(next_position.hash);

            // PV Move
            if move_count == 0 {
                score = -next_position
                    .negamax::<PV>(
                        ply + 1, 
                        depth - 1, 
                        -beta, 
                        -alpha,
                        tt, 
                        &mut local_pv, 
                        search, 
                        false
                    );

            // Search other moves with null-window, and open up window if a move
            // increases alpha
            } else {
                let mut reduction: i16 = 0;

                // Calculate LMR reduction
                if depth >= search.search_params.lmr_min_depth
                    && move_count >= search.search_params.lmr_threshold + PV as usize {
                    // Fetch the base LMR reduction value from the LMR table
                    reduction = lmr_reduction(depth, move_count) as i16;

                    // Reduce quiets and bad tacticals more
                    reduction += (legal_moves.stage() > Stage::GoodTacticals) as i16;

                    // Reduce bad captures even more
                    reduction += (legal_moves.stage() > Stage::Quiets) as i16;

                    // Reduce more if the TT move is a tactical
                    reduction += tt_move.is_some_and(|mv| mv.is_tactical()) as i16;

                    // Reduce non-pv nodes more
                    reduction -= PV as i16;

                    // Reduce less when the current position is in check
                    reduction -= in_check as i16;

                    // Reduce less when the move gives check
                    reduction -= next_position.board.in_check() as i16;

                    // Reduce moves with good history less, with bad history more
                    if mv.is_quiet() {
                        reduction -= (legal_moves.current_score() / 8191) as i16;
                    }

                    // Make sure we don't reduce below zero
                    reduction = reduction.clamp(0, depth as i16 - 1);
                }

                // Search with zero-window at reduced depth
                score = -next_position.zero_window(
                    ply + 1, 
                    depth - 1 - reduction as usize, 
                    -alpha, 
                    tt, 
                    &mut local_pv, 
                    search, 
                    true
                );

                // If score > alpha, but we were searching at reduced depth,
                // do a full-depth, zero-window search
                if score > alpha && reduction > 0 {
                    score = -next_position.zero_window(
                        ply + 1, 
                        depth - 1, 
                        -alpha, 
                        tt, 
                        &mut local_pv, 
                        search, 
                        true
                    );
                }

                // If we still find score > alpha, re-search at full-depth *and*
                // full-window
                if score > alpha && score < beta {
                    score = -next_position.negamax::<PV>(
                        ply + 1, 
                        depth - 1, 
                        -beta, 
                        -alpha,
                        tt, 
                        &mut local_pv, 
                        search, 
                        false
                    );
                }
            }

            move_count += 1;

            if score > best_score {
                best_score = score;
            }

            // Fail-low moves get marked for history score penalty
            if score < alpha && mv.is_quiet() {
                quiets_tried.push(mv);
            }

            // Tacticals that don't cause a cutoff are always penalized
            if mv.is_tactical() {
                tacticals_tried.push(mv);
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

            if search.aborted {
                return Score::MINUS_INF;
            }
        }

        // Checkmate?
        if move_count == 0 && in_check {
            return -Score::MATE + ply as Score;
        }

        // Stalemate?
        if move_count == 0 && !in_check {
            return self.score.draw_score(ply, search.tc.nodes());
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
            let idx = HistoryIndex::new(&self.board, best_move);

            ////////////////////////////////////////////////////////////////////////
            //
            // Upate the Quiet history tables
            //
            ////////////////////////////////////////////////////////////////////////

            if best_move.is_quiet() {
                search.history_table[idx] += bonus;
                search.killers[ply].add(best_move);

                if let Some(oneply) = oneply_hist_idx {
                    search.conthist_table[oneply][idx] += bonus;
                    search.countermoves[oneply] = Some(best_move);
                }

                if let Some(twoply) = twoply_hist_idx {
                    search.conthist_table[twoply][idx] += bonus;
                }

                // Deduct penalty for all tried quiets that didn't fail high
                for mv in quiets_tried {
                    let idx = HistoryIndex::new(&self.board, mv);
                    search.history_table[idx] -= bonus;

                    if let Some(oneply) = oneply_hist_idx {
                        search.conthist_table[oneply][idx] -= bonus;
                    }

                    if let Some(twoply) = twoply_hist_idx {
                        search.conthist_table[twoply][idx] -= bonus;
                    }
                } 
            }

            ////////////////////////////////////////////////////////////////////////
            //
            // Upate the Tactical history tables
            //
            ////////////////////////////////////////////////////////////////////////

            else if best_move.is_tactical() {
                if best_move.is_capture() {
                    // If the move is a capture, index the history table with
                    // the captured piece
                    let victim = self.board
                        .get_at(best_move.get_capture_sq())
                        .unwrap()
                        .piece_type();

                    search.tactical_history[victim][idx] -= bonus;
                } 

                // If the move is a promotion, index the history table with
                // a `Pawn` capture
                else {
                    use PieceType::*;
                    search.tactical_history[Pawn][idx] += bonus;
                }

                for mv in tacticals_tried {
                    // If the move is a capture, index the history table with
                    // the captured piece
                    if mv.is_capture() {
                        let victim = self.board
                            .get_at(mv.get_capture_sq())
                            .unwrap()
                            .piece_type();

                        search.tactical_history[victim][idx] -= bonus;
                    } 

                    // If the move is a promotion, index the history table with
                    // a `Pawn` capture
                    else {
                        use PieceType::*;
                        search.tactical_history[Pawn][idx] += bonus;
                    }
                }
            }
        }

        ////////////////////////////////////////////////////////////////////////
        //
        // Upate the TT
        //
        // Store the best move and score, as well as whether or not the score
        // is an upper/lower bound, or exact.
        //
        ////////////////////////////////////////////////////////////////////////

        tt.insert(TTEntry::new(
            self.hash,
            best_move.unwrap_or(Move::NULL),
            best_score,
            eval,
            depth,
            node_type,
            tt.get_age(),
            ply
        ));

        best_score
    }
}
