use chess::{board::Board, movegen::{legal_moves::All, moves::Move}};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

pub fn run_divide(fen: String, depth: usize) -> anyhow::Result<()> {
    let board = fen.parse().unwrap();
    let perft_result = perft_divide(board, depth);
    let total: usize = perft_result.iter().map(|(_, nodes)| nodes).sum();

    for (mv, nodes) in perft_result.iter() {
        println!("{mv}: {nodes}");
    }

    println!("\n{total}");

    Ok(())
}

pub fn perft(board: Board, depth: usize) -> usize {
    if depth == 0 {
        return 1;
    };

    let moves = board.legal_moves::<All>();

    // OPTIMIZATION: If we're at the last step, we don't need to go through
    // playing every single move and returning back, just return the number of
    // legal moves directly.
    if depth == 1 {
        return moves.len();
    }

    moves
        .iter()
        .map(|mv| {
            let new_board = board.play_move(*mv);
            let nodes = perft(new_board, depth - 1);
            nodes
        })
        .sum()
}

pub fn perft_divide(board: Board, depth: usize) -> Vec<(Move, usize)> {
    let moves = board.legal_moves::<All>();

    moves
        .par_iter()
        .map(|&mv| {
            let new_board = board.play_move(mv);
            let nodes = perft(new_board, depth - 1);
            (mv, nodes)
        })
        .collect()
}


