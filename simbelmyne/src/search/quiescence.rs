use crate::search_tables::HistoryTable;
use crate::search_tables::Killers;
use crate::search_tables::PVTable;
use crate::evaluate::Score;
use crate::move_picker::MovePicker;
use crate::position::Position;
use crate::evaluate::Eval;
use super::params::MAX_DEPTH;
use super::Search;

// Constants used for more readable const generics
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
        mut alpha: Eval, 
        beta: Eval, 
        pv: &mut PVTable,
        search: &mut Search,
    ) -> Eval {
        if !search.should_continue() {
            return Score::MIN;
        }

        search.tc.add_node();

        search.seldepth = search.seldepth.max(ply);

        if self.board.is_rule_draw() || self.is_repetition() {
            return Score::DRAW;
        }

        let mut local_pv = PVTable::new();

        let eval = self.score.total(self.board.current);

        if ply >= MAX_DEPTH {
            return eval;
        }

        if eval >= beta {
            return beta
        }

        if alpha < eval {
            alpha = eval;
        }

        let mut tacticals = MovePicker::new(
            &self,
            self.board.legal_moves::<CAPTURES>(),
            None,
            Killers::new(),
            HistoryTable::new(),
        );

        tacticals.only_good_tacticals = true;

        for mv in tacticals {
            let score = -self
                .play_move(mv)
                .quiescence_search(
                    ply + 1, 
                    -beta, 
                    -alpha, 
                    &mut local_pv, 
                    search
                );

            if alpha < score {
                alpha = score;
                pv.add_to_front(mv, &local_pv);
            }

            if score >= beta {
                return beta;
            }
        }

        alpha
    }
}
