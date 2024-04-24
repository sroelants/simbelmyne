use chess::movegen::moves::Move;
use chess::see::SEE_VALUES;

use crate::search_tables::Killers;
use crate::evaluate::ScoreExt;
use crate::move_picker::MovePicker;
use crate::position::Position;
use crate::evaluate::Score;
use crate::transpositions::NodeType;
use crate::transpositions::TTEntry;
use crate::transpositions::TTable;
use super::params::MAX_DEPTH;
use super::Search;
use super::params::USE_TT;
// Constants used for more readable const generics
const ALL: bool = true;
const CAPTURES: bool = false;


impl Position {
    /// Perform a less intensive negamax search that only searches captures.
    ///
    /// This is to avoid horizon effects where we misjudge a position because
    /// we stopped the search abruptly at an inopportune time.
    ///
    /// The rough flow of this function is the same as `Position::negamax`, but 
    /// we perform less pruning and hacks.
    pub fn quiescence_search(
        &self, 
        ply: usize,
        mut alpha: Score, 
        beta: Score, 
        tt: &mut TTable,
        search: &mut Search,
    ) -> Score {
        if !search.tc.should_continue() {
            search.aborted = true;
            return Score::MINUS_INF;
        }

        search.tc.add_node();

        search.seldepth = search.seldepth.max(ply);

        if self.board.is_rule_draw() || self.is_repetition() {
            return self.score.draw_score(search.tc.nodes());
        }

        let in_check = self.board.in_check();

        ////////////////////////////////////////////////////////////////////////
        //
        // Compute the static evaluation
        //
        // If the eval is _really_ good (>= beta), return it directly as a 
        // "stand pat".
        //
        ////////////////////////////////////////////////////////////////////////

        let eval = if in_check {
            // Precaution to make sure we don't miss mates
            -Score::MATE + ply as Score
        // } else if let Some(entry) = tt_entry {
        //     entry.get_eval()
        } else {
            self.score.total(&self.board)
        };

        if ply >= MAX_DEPTH {
            return eval;
        }

        if eval >= beta {
            return eval;
        }

        if alpha < eval {
            alpha = eval;
        }

        ////////////////////////////////////////////////////////////////////////
        //
        // Try and use the TT score
        //
        // Since _every_ score should technically stem from a QSearch (or a 
        // draw/mate), we should be allowed to re-use TT scores.
        //
        ////////////////////////////////////////////////////////////////////////

        let tt_entry = tt.probe(self.hash);
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

        let tt_move = tt_entry.map(|entry| entry.get_move());

        let mut tacticals = MovePicker::new(
            &self,
            self.board.legal_moves::<CAPTURES>(),
            tt_move,
            Killers::new(),
        );

        tacticals.only_good_tacticals = true;

        let mut best_move = Move::NULL;
        let mut best_score = eval;
        let mut node_type = NodeType::Upper;

        // If we're in check and there are no captures, we need to check
        // whether it might be mate!
        if in_check 
            && tacticals.len() == 0 
            && self.board.legal_moves::<ALL>().len() == 0 {
            return -Score::MATE + ply as Score;
        }

       while let Some(mv) = tacticals.next(&search.history_table) {
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

            let capture_value = self.board.get_at(mv.tgt())
                .map(|p| SEE_VALUES[p.piece_type() as usize])
                .unwrap_or(0);

            let futility = eval 
                + capture_value 
                + search.search_params.delta_pruning_margin;

            if !in_check && futility <= alpha {
                continue;
            }

            ////////////////////////////////////////////////////////////////////
            //
            // Play the move
            //
            // Play the move and recurse down the tree
            //
            ////////////////////////////////////////////////////////////////////
            let next_position = self.play_move(mv);
            tt.prefetch(next_position.hash);

            let score = -next_position
                .quiescence_search(
                    ply + 1, 
                    -beta, 
                    -alpha, 
                    tt,
                    search
                );

            if score > best_score {
                best_score = score;
                best_move = mv;
            }

            if score >= beta {
                node_type = NodeType::Lower;
                break;
            }

            if score > alpha {
                alpha = score;
                node_type = NodeType::Exact;
            }

            if search.aborted {
                return Score::MINUS_INF;
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

        // Store in the TT
        if USE_TT {
            tt.insert(TTEntry::new(
                self.hash,
                best_move,
                best_score,
                eval,
                0,
                node_type,
                tt.get_age(),
                ply
            ));
        }

        best_score
    }
}
