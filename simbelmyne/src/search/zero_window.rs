use crate::evaluate::{Eval, Score};
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
        eval_state: Eval,
        try_null: bool,
    ) -> Score {
        self.negamax::<false>(
            pos,
            ply, 
            depth, 
            value-1, 
            value, 
            pv, 
            eval_state, 
            try_null
        )
    }
}
