use crate::evaluate::Score;
use crate::position::Position;
use crate::transpositions::TTable;
use crate::search_tables::PVTable;

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
        try_null: bool,
    ) -> Score {
        self.negamax::<false>(ply, depth, value-1, value, tt, pv, search, try_null)
    }
}
