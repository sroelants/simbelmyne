use crate::evaluate::Score;
use crate::position::Position;
use crate::transpositions::{PawnCache, TTable};
use crate::search_tables::PVTable;

use super::Search;

impl Position {
    pub fn zero_window(
        &self, 
        ply: usize, 
        depth: usize, 
        value: Score, 
        tt: &mut TTable, 
        pawn_cache: &mut PawnCache,
        pv: &mut PVTable,
        search: &mut Search,
        try_null: bool,
    ) -> Score {
        self.negamax::<false>(
            ply, 
            depth, 
            value-1, 
            value, 
            tt, 
            pawn_cache,
            pv, 
            search, 
            try_null
        )
    }
}
