use crate::history_tables::history::HistoryIndex;
use crate::history_tables::history::HistoryScore;
use crate::history_tables::pv::PVTable;
use crate::history_tables::threats::ThreatIndex;
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

use super::params::*;
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
        let excluded = search.stack[ply].excluded;

        // Carry over the current count of double extensions
        if ply > 0 {
            search.stack[ply].double_exts = search.stack[ply-1].double_exts;
        }

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

        let tt_entry = if excluded.is_none() { 
            tt.probe(self.hash) 
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
        
        let eval = if let Some(entry) = tt_entry {
            entry.get_eval()
        } else if excluded.is_some() {
            // In singular verification, we can re-use the eval already stored
            // in the stack.
            search.stack[ply].eval
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

        let futility = rfp_margin() * depth as Score
            + rfp_improving_margin() * !improving as Score;

        if !PV 
            && !in_root
            && !in_check
            && excluded.is_none()
            && depth <= rfp_threshold()
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
            && excluded.is_none()
            && eval + nmp_improving_margin() * improving as Score >= beta
            && self.board.zugzwang_unlikely();

        if should_null_prune {
            let reduction = (nmp_base_reduction() + depth / nmp_reduction_factor())
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
            .map(|ply| search.stack[ply].history_index);

        let twoply_hist_idx = ply
            .checked_sub(2)
            .map(|ply| search.stack[ply].history_index);

        let fourply_hist_idx = ply
            .checked_sub(4)
            .map(|ply| search.stack[ply].history_index);

        let countermove = oneply_hist_idx.and_then(|idx| search.countermoves[idx]);

        let mut legal_moves = MovePicker::<ALL_MOVES>::new(
            &self,  
            tt_move,
            search.killers[ply],
            countermove
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

        let oneply_conthist = oneply_hist_idx
            .map(|ply| search.conthist_table[ply]);

        let twoply_conthist = twoply_hist_idx
            .map(|ply| search.conthist_table[ply]);

        let fourply_conthist = fourply_hist_idx
            .map(|ply| search.conthist_table[ply]);

        while let Some(mv) = legal_moves.next(
            &search.history_table, 
            &search.tactical_history,
            oneply_conthist.as_ref(),
            twoply_conthist.as_ref(),
            fourply_conthist.as_ref(),
        ) {
            if Some(mv) == excluded {
                continue;
            }

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

            let futility = fp_base()
                + fp_margin() * (lmr_depth as Score)
                + 100 * improving as Score;

            if move_count > 0 
                && !PV
                && !in_check
                && lmr_depth <= fp_threshold()
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

            let see_margin = -see_quiet_margin() * depth as Score;

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

            let mut extension = 0;

            if se_candidate == Some(mv) {
                let mut local_pv = PVTable::new();
                let tt_score = tt_entry.unwrap().get_score();

                let se_depth = (depth - 1) / 2;
                let se_beta = Score::max(
                    tt_score - se_margin() * depth as Score,
                    -Score::MATE
                );

                // Do a verification search with the candidate move excluded.
                search.stack[ply].excluded = se_candidate;
                let value = self.zero_window(
                    ply, 
                    se_depth, 
                    se_beta, 
                    tt, 
                    &mut local_pv, 
                    search, 
                    try_null
                );
                search.stack[ply].excluded = None;

                if value < se_beta {
                    // Double extensions:
                    // If we're below the threshold by a lot, reduce by 2 ply
                    // Make sure to keep the total number of double extensions
                    // limited, though.
                    if !PV 
                    && value + double_ext_margin() < se_beta 
                    && search.stack[ply].double_exts <= double_ext_max() {
                        extension += 2;

                        search.stack[ply].double_exts += 1;
                    } else {
                        extension += 1;
                    }
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
            search.stack[ply].history_index = HistoryIndex::new(&self.board, mv);
            let next_position = self.play_move(mv);

            // Instruct the CPU to load the TT entry into the cache ahead of time
            tt.prefetch(next_position.hash);

            // PV Move
            if move_count == 0 {
                score = -next_position
                    .negamax::<PV>(
                        ply + 1, 
                        depth + extension - 1, 
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
                    depth - 1 + extension - reduction as usize, 
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
                        depth + extension - 1, 
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
                        depth + extension - 1, 
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

            if search.aborted {
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

            ////////////////////////////////////////////////////////////////////
            //
            // Upate the Quiet history tables
            //
            ////////////////////////////////////////////////////////////////////

            if best_move.is_quiet() {
                let idx = HistoryIndex::new(&self.board, best_move);
                let threat_idx = ThreatIndex::new(self.board.threats, best_move);

                search.history_table[threat_idx][idx] += bonus;
                search.killers[ply].add(best_move);

                if let Some(oneply) = oneply_hist_idx {
                    search.conthist_table[oneply][idx] += bonus;
                    search.countermoves[oneply] = Some(best_move);
                }

                if let Some(twoply) = twoply_hist_idx {
                    search.conthist_table[twoply][idx] += bonus;
                }

                if let Some(fourply) = fourply_hist_idx {
                    search.conthist_table[fourply][idx] += bonus;
                }

                // Deduct penalty for all tried quiets that didn't fail high
                for mv in quiets_tried {
                    let threat_idx = ThreatIndex::new(self.board.threats, mv);
                    let idx = HistoryIndex::new(&self.board, mv);

                    search.history_table[threat_idx][idx] -= bonus;

                    if let Some(oneply) = oneply_hist_idx {
                        search.conthist_table[oneply][idx] -= bonus;
                    }

                    if let Some(twoply) = twoply_hist_idx {
                        search.conthist_table[twoply][idx] -= bonus;
                    }

                    if let Some(fourply) = fourply_hist_idx {
                        search.conthist_table[fourply][idx] -= bonus;
                    }
                } 
            }

            ////////////////////////////////////////////////////////////////////
            //
            // Upate the Tactical history tables
            //
            ////////////////////////////////////////////////////////////////////

            // Add a bonus for the move that caused the cutoff
            else if best_move.is_tactical() {
                let victim = if let Some(piece) = self.board.get_at(best_move.tgt()) {
                    piece.piece_type()
                } else {
                    PieceType::Pawn
                };

                let idx = HistoryIndex::new(&self.board, best_move);
                search.tactical_history[victim][idx] += bonus;
            } 

            // Deduct a penalty from all tacticals that didn't cause a cutoff
            for mv in tacticals_tried {
                let idx = HistoryIndex::new(&self.board, mv);

                let victim = if let Some(piece) = self.board.get_at(mv.tgt()) {
                    piece.piece_type()
                } else {
                    PieceType::Pawn
                };

                search.tactical_history[victim][idx] -= bonus;
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

        if excluded.is_none() {
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
        }

        best_score
    }
}
