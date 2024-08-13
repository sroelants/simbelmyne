use crate::evaluate::{Eval, Score};
use crate::history_tables::pv::PVTable;
use crate::position::Position;
use crate::transpositions::TTable;

use super::Search;

impl Position {
    pub fn zero_window(
        &self, 
        ply: usize, 
        depth: usize, 
        value: Score, 
        tt: &mut TTable, 
        pv: &mut PVTable,
        search: &mut Search,
        eval_state: Eval,
        try_null: bool,
    ) -> Score {
        self.negamax::<false>(ply, depth, value-1, value, tt, pv, search, eval_state, try_null)
    }
}
