use super::SearchRunner;
use crate::evaluate::Score;
use crate::position::Position;
use crate::search::NonPv;

impl<'a> SearchRunner<'a> {
  pub fn zero_window(
    &mut self,
    pos: &Position,
    ply: usize,
    depth: usize,
    value: Score,
    try_null: bool,
    cutnode: bool,
  ) -> Score {
    self.negamax::<NonPv>(pos, ply, depth, value - 1, value, try_null, cutnode)
  }
}
