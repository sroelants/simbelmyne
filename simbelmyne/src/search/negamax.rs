use crate::transpositions::NodeType;
use crate::transpositions::TTEntry;
use crate::search_tables::PVTable;
use crate::evaluate::Score;
use crate::transpositions::TTable;
use crate::move_picker::MovePicker;
use crate::position::Position;
use crate::evaluate::Eval;
use chess::movegen::moves::Move;

use super::Search;
use super::params::FP_MARGINS;
use super::params::FP_THRESHOLD;
use super::params::HISTORY_TABLE;
use super::params::KILLER_MOVES;
use super::params::MAX_DEPTH;
use super::params::NULL_MOVE_PRUNING;
use super::params::NULL_MOVE_REDUCTION;
use super::params::QUIESCENCE_SEARCH;
use super::params::RFP_MARGIN;
use super::params::RFP_THRESHOLD;
use super::params::USE_TT;

// Constants used for more readable const generics
const QUIETS: bool = true;

impl Position {
    /// The main negamax function of the search routine.
    pub fn negamax<const PV: bool>(
        &self, 
        ply: usize, 
        mut depth: usize,
        alpha: Eval, 
        beta: Eval, 
        tt: &mut TTable, 
        pv: &mut PVTable,
        search: &mut Search,
        try_null: bool,
    ) -> Eval {
        if !search.should_continue() {
            return Score::MIN;
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
        let mut best_score = Score::MIN;
        let mut node_type = NodeType::Upper;
        let mut alpha = alpha;
        let tt_entry = tt.probe(self.hash);
        let mut local_pv = PVTable::new();

        pv.clear();
        search.tc.add_node();

        // Rule-based draw? 
        // Don't return early when in the root node, because we won't have a PV 
        // move to play.
        if !in_root && (self.board.is_rule_draw() || self.is_repetition()) {
            return Score::DRAW;
        }

        // Do all the static evaluations first
        // That is, Check whether we can/should assign a score to this node
        // without recursing any deeper.

        // Rule-based draw? 
        // Don't return early when in the root node, because we won't have a PV 
        // move to play.
        if (self.board.is_rule_draw() || self.is_repetition()) && !in_root {
            return Score::DRAW;
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
        // Reverse futility pruning
        //
        // If we're close to the max depth of the search, and the static 
        // evaluation board is some margin above beta, assume it's highly 
        // unlikely for the search _not_ to end in a cutoff, and just return
        // beta instead.
        //
        ////////////////////////////////////////////////////////////////////////
        let eval = self.score.total(self.board.current);

        if depth <= RFP_THRESHOLD 
            && eval >= beta + RFP_MARGIN * depth as i32
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
            && !PV
            && depth >= NULL_MOVE_REDUCTION + 1;

        if should_null_prune {
            let score = -self
                .play_move(Move::NULL)
                .zero_window(
                    ply + 1, 
                    depth - 1 - NULL_MOVE_REDUCTION, 
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
        // Generate the legal moves and do some static checks to see whether 
        // we can prune, or bail altogether.
        //
        ////////////////////////////////////////////////////////////////////////

        let mut legal_moves = MovePicker::new(
            &self,  
            self.board.legal_moves::<QUIETS>(),
            tt_entry.map(|entry| entry.get_move()),
            search.killers[ply],
            search.history_table,
        );


        // Checkmate?
        if legal_moves.len() == 0 && in_check {
            return -Score::MATE + ply as Eval;
        }

        // Stalemate?
        if legal_moves.len() == 0 && !in_check {
            return Score::DRAW;
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

        if depth <= FP_THRESHOLD
            && eval + FP_MARGINS[depth] <= alpha
            && !PV
            && !in_check
            && legal_moves.count_tacticals() > 0
            && !Score::is_mate_score(alpha)
            && !Score::is_mate_score(beta) 
        {
            legal_moves.only_good_tacticals = true;
        }

        ////////////////////////////////////////////////////////////////////////
        //
        // Iterate over the remaining moves
        //
        ////////////////////////////////////////////////////////////////////////

        for (move_count, mv) in legal_moves.enumerate() {
            if !search.should_continue() {
                return Score::MIN;
            }

            let mut score;

            // PV Move
            if move_count == 0 {
            score = -self
                .play_move(mv)
                .negamax::<true>(ply + 1, 
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
                score = -self.play_move(mv).zero_window(
                    ply + 1, 
                    depth - 1, 
                    -alpha, 
                    tt, 
                    &mut local_pv, 
                    search, 
                    true
                );

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
            }

            if beta <= score {
                node_type = NodeType::Lower;
                break;
            }
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
                depth,
                node_type,
            ));
        }

        score
    }
}


