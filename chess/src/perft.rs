use crate::{board::Board, movegen::{legal_moves::All, moves::Move}};

impl Board {
    /// Count and return the number of leave nodes at a given depth
    pub fn perft(&self, depth: usize) -> u64 {
        if depth == 0 {
            return 1;
        };

        // OPTIMIZATION: If we're at the last step, we don't need to go through
        // playing every single move and returning back, just return the number of
        // legal moves directly.
        if depth == 1 {
            return self.pseudolegal_moves::<All>()
                .iter()
                .filter(|&&mv| self.is_legal(mv))
                .count() as u64;
        }

        self.pseudolegal_moves::<All>()
            .iter()
            .filter(|&&mv| self.is_legal(mv))
            .map(|&mv| self.play_move(mv).perft(depth - 1))
            .sum()
    }

    /// Count and return the number of leave nodes at a given depth, grouped 
    /// by the first move.
    pub fn perft_divide(&self, depth: usize) -> Vec<(Move, u64)> {
        self.pseudolegal_moves::<All>()
            .iter()
            .filter(|&&mv| self.is_legal(mv))
            .map(|&mv| {
                let nodes = self.play_move(mv).perft(depth - 1);
                (mv, nodes)
            })
            .collect()
    }
}
