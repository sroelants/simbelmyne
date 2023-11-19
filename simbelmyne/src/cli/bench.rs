use colored::Colorize;

use crate::{position::Position, transpositions::TTable, search::SearchOpts};

use super::presets::Preset;
pub fn run_bench(depth: usize, fen: Option<String>) {
    if let Some(fen) = fen {
        run_single(&fen, depth);
    } else {
        for preset in Preset::all_presets() {
            run_single(preset.fen, depth);
        }
    }
}

pub fn run_single(fen: &str, depth: usize) {
    let board = fen.parse().unwrap();
    let position = Position::new(board);
    let mut tt = TTable::with_capacity(64);
    let opts = SearchOpts::new();

    let search = position.search(depth, &mut tt, opts);


    println!("{board}");
    println!("{:15}: {}", "FEN".green(), fen);
    println!("{:15}: {}", "Depth".green(), depth);
    println!("---");

    println!("{:15}: {}", "Best move".bright_cyan(), search.best_moves[0]);
    println!("{:15}: {}", "Score".bright_cyan(), search.scores[0]);
    println!("{:15}: {}", "Static eval".bright_cyan(), search.eval[0]);


    let nodes_visited: usize = search.nodes_visited.iter().sum();
    println!("{:15}: {}", "Nodes visited".blue(), nodes_visited);

    let leaf_nodes = search.nodes_visited[depth - 1];
    println!("{:15}: {}", "Leaf nodes".blue(), leaf_nodes);

    let beta_cutoffs: usize = search.beta_cutoffs.iter().sum();
    println!("{:15}: {}", "Beta cutoffs".blue(), beta_cutoffs);

    let time_spent = search.durations[0].as_millis();
    println!("{:15}: {}ms", "Duration".red(), time_spent);

    let knps = nodes_visited / time_spent as usize;
    println!("{:15}: {}knps", "knps".red(), knps);

    println!("\n");
}
