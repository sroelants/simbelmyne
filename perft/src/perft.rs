use std::time::Instant;
use chess::{board::Board, movegen::moves::Move};

pub struct PerftResult {
    pub nodes: usize,
    pub duration: u128,
}

impl PerftResult {
    /// Return Nodes per second in units of Meganodes (1m nodes) per second
    pub fn mega_nps(&self) -> f64 {
        if self.duration > 0 {
            self.nodes as f64 / self.duration as f64
        } else {
            0f64
        }
    }

    /// Return the run duration in milliseconds
    pub fn millis(&self) -> u128 {
        self.duration / 1000
    }
}

pub fn perft<const BULK: bool>(board: Board, depth: usize) -> usize {
    if depth == 0 { return 1 };

    let mut nodes = 0;
    let moves = board.legal_moves();

    // OPTIMIZATION: If we're at the last step, we don't need to go through 
    // playing every single move and returning back, just return the number of 
    // legal moves directly.
    if BULK && depth == 1 { return moves.len() }

    for mv in board.legal_moves() {
        let new_board = board.play_move(mv);
        nodes += perft::<BULK>(new_board, depth - 1);
    }

    nodes
}

pub fn run_perft<const BULK: bool>(board: Board, depth: usize) -> PerftResult {
    let start = Instant::now();
    let nodes = perft::<BULK>(board, depth);
    let duration = start.elapsed();

    return PerftResult { nodes, duration: duration.as_micros() }
}

pub fn perft_divide<const BULK: bool>(board: Board, depth: usize) -> Vec<(Move, usize)> {
    let moves = board.legal_moves();

    moves
        .into_iter()
        .map(|mv| {
            let new_board = board.play_move(mv);
            let nodes = perft::<BULK>(new_board, depth - 1);
            (mv, nodes)
        })
        .collect()
}
