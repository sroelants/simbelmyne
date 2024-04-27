use crate::move_picker::Stage;
use crate::transpositions::NodeType;
use crate::transpositions::TTEntry;
use crate::search_tables::PVTable;
use crate::evaluate::ScoreExt;
use crate::transpositions::TTable;
use crate::move_picker::MovePicker;
use crate::position::Position;
use crate::evaluate::Score;
use chess::movegen::legal_moves::MoveList;
use chess::movegen::moves::Move;

use super::params::LMR_MAX_MOVES;
use super::params::LMR_TABLE;
use super::Search;
use super::params::HISTORY_TABLE;
use super::params::IIR_THRESHOLD;
use super::params::KILLER_MOVES;
use super::params::MAX_DEPTH;
use super::params::NULL_MOVE_PRUNING;
use super::params::QUIESCENCE_SEARCH;
use super::params::USE_TT;

// Constants used for more readable const generics
const QUIETS: bool = true;

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
            if QUIESCENCE_SEARCH {
                return self.quiescence_search(ply, alpha, beta, tt, search);
            } else {
                return self.score.total(&self.board);
            }
        }

        ////////////////////////////////////////////////////////////////////////
        //
        // Start processing node
        //
        ////////////////////////////////////////////////////////////////////////

        search.tc.add_node();

        let mut best_move = Move::NULL;
        let mut best_score = Score::MINUS_INF;
        let mut node_type = NodeType::Upper;
        let mut alpha = alpha;
        let mut local_pv = PVTable::new();

        // Do all the static evaluations first
        // That is, Check whether we can/should assign a score to this node
        // without recursing any deeper.

        // Rule-based draw? 
        // Don't return early when in the root node, because we won't have a PV 
        // move to play.
        if !in_root && (self.board.is_rule_draw() || self.is_repetition()) {
            return self.score.draw_score(search.tc.nodes());
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
        let tt_move = tt_entry.map(|entry| entry.get_move());

        if !PV && !in_root && tt_entry.is_some() {
            let tt_entry = tt_entry.unwrap();

            // Can we use the stored score?
            let tt_score = tt_entry.try_score(depth, alpha, beta, ply);

            if let Some(score) = tt_score {
                let mv = tt_entry.get_move();

                // Should we store the move as a killer?
                let is_killer = node_type == NodeType::Lower && mv.is_quiet();
                if is_killer && KILLER_MOVES { 
                    search.killers[ply].add(best_move);
                }

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

        ////////////////////////////////////////////////////////////////////////
        //
        // Reverse futility pruning
        //
        // If we're close to the max depth of the search, and the static 
        // evaluation board is some margin above beta, assume it's highly 
        // unlikely for the search _not_ to end in a cutoff. Instead, just 
        // return a compromise value between the current eval and beta.
        //
        // TODO: Other options to try here are: 
        // - return eval
        // - return the "Worst case scenario" score (eval - futility)
        //
        ////////////////////////////////////////////////////////////////////////

        let futility = search.search_params.rfp_margin * depth as Score;

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

        let should_null_prune = NULL_MOVE_PRUNING 
            && try_null
            && !PV
            && !in_root
            && !in_check
            && self.board.zugzwang_unlikely();

        if should_null_prune {
            let reduction = (search.search_params.nmp_base_reduction + depth / search.search_params.nmp_reduction_factor)
                .min(depth);

            let score = -self
                .play_move(Move::NULL)
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

        if tt_entry.is_none() && !in_root && depth >= IIR_THRESHOLD {
            depth -= 1;
        }

        ////////////////////////////////////////////////////////////////////////
        //
        // Generate the legal moves and do some static checks to see whether 
        // we can prune, or bail altogether.
        //
        ////////////////////////////////////////////////////////////////////////

        let mut legal_moves = MovePicker::new(
            &self,  
            self.board.legal_moves::<QUIETS>(),
            tt_move,
            search.killers[ply],
        );

        // Checkmate?
        if legal_moves.len() == 0 && in_check {
            return -Score::MATE + ply as Score;
        }

        // Stalemate?
        if legal_moves.len() == 0 && !in_check {
            return self.score.draw_score(search.tc.nodes());
        }

        ////////////////////////////////////////////////////////////////////////
        //
        // Futility pruning
        // 
        // If we're near the end of the search, and the static evaluation of 
        // this node is lower than alpha by some margin, we prune away moves 
        // that are unlikely to be able to increase alpha. (i.e., quiet moves).
        //
        ////////////////////////////////////////////////////////////////////////

        if depth <= search.search_params.fp_threshold
            && eval + search.search_params.fp_margins[depth] <= alpha
            && !PV
            && !in_check
            && legal_moves.count_tacticals() > 0
            && !alpha.is_mate()
            && !beta.is_mate()
        {
            legal_moves.only_good_tacticals = true;
        }

        ////////////////////////////////////////////////////////////////////////
        //
        // Iterate over the remaining moves
        //
        ////////////////////////////////////////////////////////////////////////

        let mut move_count = 0;
        let mut quiets_tried = MoveList::new();

        while let Some(mv) = legal_moves.next(&search.history_table) {
            local_pv.clear();

            if !search.tc.should_continue() {
                search.aborted = true;
                return Score::MINUS_INF;
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

            if depth <= search.search_params.lmp_threshold
                && !PV
                && !in_check
                && move_count >= search.search_params.lmp_move_thresholds[depth] {
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
            let next_position = self.play_move(mv);

            // Instruct the CPU to load the TT entry into the cache ahead of time
            tt.prefetch(next_position.hash);

            // PV Move
            if move_count == 0 {
                score = -next_position
                    .negamax::<true>(
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
                let mut reduction: usize = 0;

                // Calculate LMR reduction
                if depth >= search.search_params.lmr_min_depth
                && move_count >= search.search_params.lmr_threshold
                && !in_check {
                    let move_count = move_count.clamp(0, LMR_MAX_MOVES);

                    reduction = LMR_TABLE[depth][move_count];

                    // Reduce non-pv nodes more
                    reduction += !PV as usize;

                    // Reduce quiets and bad tacticals more
                    reduction += (legal_moves.stage() > Stage::GoodTacticals) as usize;

                    // Reduce bad captures even more
                    reduction += (legal_moves.stage() > Stage::Quiets) as usize;

                    // Make sure we don't reduce below zero
                    reduction = reduction.clamp(0, depth - 2);
                }

                // Search with zero-window at reduced depth
                score = -next_position.zero_window(
                    ply + 1, 
                    depth - 1 - reduction, 
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
                if score > alpha {
                    score = -next_position.negamax::<true>(
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

            if score > best_score {
                best_score = score;
                best_move = mv;
            }

            if score < alpha && mv.is_quiet() {
                // Fail-low moves get marked for history score penalty
                quiets_tried.push(mv);
            }

            if score >= beta {
                node_type = NodeType::Lower;
                break;
            }

            if score > alpha {
                alpha = score;
                node_type = NodeType::Exact;
                pv.add_to_front(mv, &local_pv);
            }

            if search.aborted {
                return Score::MINUS_INF;
            }

            move_count += 1;
        }

        ////////////////////////////////////////////////////////////////////////
        //
        // Upate the search tables
        //
        // Store the best move and score, as well as whether or not the score
        // is an upper/lower bound, or exact.
        //
        ////////////////////////////////////////////////////////////////////////

        // If we had a cutoff, update the Killers and History
        if node_type == NodeType::Lower && best_move.is_quiet() {
            if HISTORY_TABLE {
                let piece = self.board.get_at(best_move.src()).unwrap();
                search.history_table.increment(&best_move, piece, depth);

                for mv in quiets_tried {
                    let piece = self.board.get_at(mv.src()).unwrap();
                    search.history_table.decrement(&mv, piece, depth);
                }
            }

            if KILLER_MOVES {
                search.killers[ply].add(best_move);
            }
        }

        // Store in the TT
        if USE_TT {
            tt.insert(TTEntry::new(
                self.hash,
                best_move,
                best_score,
                eval,
                depth,
                node_type,
                tt.get_age(),
                ply
            ));
        }

        best_score
    }
}


