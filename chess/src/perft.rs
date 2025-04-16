use crate::board::Board;
use crate::movegen::legal_moves::All;
use crate::movegen::moves::Move;

impl Board {
  /// Count and return the number of leave nodes at a given depth
  pub fn perft(&self, depth: usize) -> u64 {
    if depth == 0 {
      return 1;
    };

    // OPTIMIZATION: If we're at the last step, we don't need to go through
    // playing every single move and returning back, just return the number
    // of legal moves directly.
    if depth == 1 {
      return self.legal_moves::<All>().len() as u64;
    }

    self
      .legal_moves::<All>()
      .iter()
      .map(|&mv| self.play_move(mv).perft(depth - 1))
      .sum()
  }

  /// Count and return the number of leave nodes at a given depth, grouped
  /// by the first move.
  pub fn perft_divide(&self, depth: usize) -> Vec<(Move, u64)> {
    self
      .legal_moves::<All>()
      .iter()
      .map(|&mv| {
        let nodes = self.play_move(mv).perft(depth - 1);
        (mv, nodes)
      })
      .collect()
  }
}
