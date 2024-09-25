use chess::board::Board;

pub fn run_divide(fen: String, depth: usize) -> anyhow::Result<()> {
    let board: Board = fen.parse().unwrap();
    let perft_result = board.perft_divide(depth);
    let total: u64 = perft_result.iter().map(|(_, nodes)| nodes).sum();

    for (mv, nodes) in perft_result.iter() {
        println!("{mv}: {nodes}");
    }

    println!("\n{total}");

    Ok(())
}
