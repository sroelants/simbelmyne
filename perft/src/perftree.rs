use std::str::FromStr;
use chess::{board::Board, movegen::moves::Move};
use crate::{perft::perft_divide, BULK};

pub fn run_perftree(depth: usize, fen: String, moves: Vec<String>) -> anyhow::Result<()> {
    let mut board: Board = fen.parse().unwrap();
    let moves: Vec<Move> = moves
        .join(" ")
        .split(' ')
        .filter(|s| !s.is_empty())
        .map(|mv| Move::from_str(mv).unwrap())
        .collect();

    for mv in moves.iter() {
        board = board.play_move(*mv);
    }

    let results = perft_divide::<BULK>(board, depth);

    results.iter().for_each(|(mv, nodes)| println!("{mv} {nodes}"));
    println!("");
    println!("{}", results.iter().map(|(_, nodes)| nodes).sum::<usize>());

    Ok(())
}
