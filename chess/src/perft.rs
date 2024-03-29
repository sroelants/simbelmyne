use crate::{board::Board, movegen::moves::Move};

const QUIETS: bool = true;

pub fn perft(board: Board, depth: usize) -> usize {
    if depth == 0 {
        return 1;
    };

    let moves = board.legal_moves::<QUIETS>();

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
    let moves = board.legal_moves::<QUIETS>();

    moves
        .iter()
        .map(|&mv| {
            let new_board = board.play_move(mv);
            let nodes = perft(new_board, depth - 1);
            (mv, nodes)
        })
        .collect()
}


