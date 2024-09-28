use crate::evaluate::Score;
use crate::history_tables::pv::PVTable;
use crate::position::Position;
use super::SearchRunner;

impl<'a> SearchRunner<'a> {
    pub fn zero_window(
        &mut self, 
        pos: &Position,
        ply: usize, 
        depth: usize, 
        value: Score, 
        pv: &mut PVTable,
        try_null: bool,
    ) -> Score {
        self.negamax::<false>(
            pos,
            ply, 
            depth, 
            value-1, 
            value, 
            pv, 
            try_null
        )
    }
}
