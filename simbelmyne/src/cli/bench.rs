use colored::Colorize;

use crate::{position::Position, transpositions::TTable, search::SearchOpts, time_control::TimeControl};

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
    let mut opts = SearchOpts::ALL;
    opts.tt_move = true;
    opts.mvv_lva = true;
    opts.killers = true;
    opts.history_table = true;
    opts.debug = true;
    let (tc, _handle) = TimeControl::fixed_depth(depth);
    let search = position.search(&mut tt, opts, tc);

    println!("{board}");
    println!("{:17} {}", "FEN:".green(), fen);
    println!("{:17} {}", "Depth:".green(), depth);
    println!();

    println!("{:17} {}", "Best move:".bright_cyan(), search.pv.pv_move());
    println!("{:17} {}", "Score:".bright_cyan(), search.score);


    let nodes_visited: usize = search.nodes_visited;
    println!("{:17} {}", "Nodes visited:".blue(), nodes_visited);

    let leaf_nodes = search.leaf_nodes;
    println!("{:17} {}", "Leaf nodes:".blue(), leaf_nodes);

    let beta_cutoffs: usize = search.beta_cutoffs.iter().sum();
    println!("{:17} {}", "Beta cutoffs:".blue(), beta_cutoffs);

    let time_spent = search.duration.as_millis();
    println!("{:17} {}ms", "Duration:".red(), time_spent);

    let knps = nodes_visited / if time_spent > 0 { time_spent as usize } else { 1 };
    println!("{:17} {}knps", "knps:".red(), knps);

    // Branching factors
    let root_bf = (nodes_visited as f32).powf(1.0 / (depth as f32));
    println!("{:17} {:.2}", "Branching factor:".red(), root_bf);

    // TT info
    println!("{:17} {}%", "TT occupancy".purple(), tt.occupancy());
    println!("{:17} {}", "TT inserts".purple(), tt.inserts());
    println!("{:17} {}", "TT hits".purple(), search.tt_hits);

    println!("\n");
}
