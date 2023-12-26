use crate::evaluate::Eval;
use crate::position::Position;
use crate::transpositions::TTable;
use crate::search_tables::PVTable;

use super::Search;

impl Position {
    pub fn zero_window(
        &self, 
        ply: usize, 
        depth: usize, 
        value: Eval, 
        tt: &mut TTable, 
        pv: &mut PVTable,
        search: &mut Search,
        try_null: bool,
    ) -> Eval {
        self.negamax::<false>(ply, depth, value-1, value, tt, pv, search, try_null)
    }
}
