use crate::transpositions::NodeType;
use crate::transpositions::TTEntry;
use crate::search_tables::PVTable;
use crate::evaluate::Eval;
use crate::transpositions::TTable;
use crate::move_picker::MovePicker;
use crate::position::Position;
use crate::evaluate::Score;
use chess::movegen::legal_moves::MoveList;
use chess::movegen::moves::Move;

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
            return Eval::MIN;
        }

        let in_root = ply == 0;

        search.tc.add_node();
        pv.clear();

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
                return self.quiescence_search(ply, alpha, beta, pv, search);
            } else {
                search.tc.add_node();
                return self.score.total(self.board.current);
            }
        }

        ////////////////////////////////////////////////////////////////////////
        //
        // Start processing node
        //
        ////////////////////////////////////////////////////////////////////////

        let mut best_move = Move::NULL;
        let mut best_score = Eval::MIN;
        let mut node_type = NodeType::Upper;
        let mut alpha = alpha;
        let tt_entry = tt.probe(self.hash);
        let tt_move = tt_entry.map(|entry| entry.get_move());
        let mut local_pv = PVTable::new();

        // Rule-based draw? 
        // Don't return early when in the root node, because we won't have a PV 
        // move to play.
        if !in_root && (self.board.is_rule_draw() || self.is_repetition()) {
            return Eval::DRAW;
        }

        // Do all the static evaluations first
        // That is, Check whether we can/should assign a score to this node
        // without recursing any deeper.

        // Rule-based draw? 
        // Don't return early when in the root node, because we won't have a PV 
        // move to play.
        if (self.board.is_rule_draw() || self.is_repetition()) && !in_root {
            return Eval::DRAW;
        }

        if ply >= MAX_DEPTH {
            return self.score.total(self.board.current);
        }

        ////////////////////////////////////////////////////////////////////////
        //
        // TT cutoffs
        //
        // Check the TT table for a result that we can use, and return it
        //
        ////////////////////////////////////////////////////////////////////////

        if !in_root {
            let tt_result = tt_entry.and_then(|entry| {
                entry.try_use(depth, alpha, beta)
            });

            if let Some((mv, score)) = tt_result {
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
            self.score.total(self.board.current)
        };

        ////////////////////////////////////////////////////////////////////////
        //
        // Reverse futility pruning
        //
        // If we're close to the max depth of the search, and the static 
        // evaluation board is some margin above beta, assume it's highly 
        // unlikely for the search _not_ to end in a cutoff, and just return
        // beta instead.
        //
        ////////////////////////////////////////////////////////////////////////

        if depth <= search.search_params.rfp_threshold 
            && eval >= beta.saturating_add(search.search_params.rfp_margin * depth as Score)
            && !in_root
            && !in_check
            && !PV
        {
            return beta;
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
                return beta;
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
            return -Eval::MATE + ply as Score;
        }

        // Stalemate?
        if legal_moves.len() == 0 && !in_check {
            return Eval::DRAW;
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
            && !Eval::is_mate_score(alpha)
            && !Eval::is_mate_score(beta) 
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
            if !search.tc.should_continue() {
                search.aborted = true;
                return Eval::MIN;
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

            // PV Move
            if move_count == 0 {
            score = -self
                .play_move(mv)
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
                    let move_count = move_count.clamp(0, search.search_params.lmr_max_moves);

                    reduction = search.search_params.lmr_table[depth][move_count];

                    reduction += !PV as usize;

                    reduction += mv.is_quiet() as usize;

                    reduction = reduction.clamp(0, depth - 2);
                }

                // Search with zero-window at reduced depth
                score = -self.play_move(mv).zero_window(
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
                    score = -self.play_move(mv).zero_window(
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
                    score = -self.play_move(mv).negamax::<true>(
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

            if alpha < score {
                alpha = score;
                node_type = NodeType::Exact;
                pv.add_to_front(mv, &local_pv);
            } else {
                // Fail-low moves get marked for history score penalty
                if mv.is_quiet() {
                    quiets_tried.push(mv);
                }
            }

            if beta <= score {
                node_type = NodeType::Lower;
                break;
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
        
        // Fail-hard semantics: the score we return is clamped to the
        // `alpha`-`beta` window.
        let score = match node_type {
            NodeType::Upper => alpha,
            NodeType::Exact => best_score,
            NodeType::Lower => beta,
        };

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
                score,
                eval,
                depth,
                node_type,
                tt.get_age()
            ));
        }

        score
    }
}


