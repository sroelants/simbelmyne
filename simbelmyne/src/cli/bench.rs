use colored::Colorize;
use uci::time_control::TimeControl;

use crate::{position::Position, search::params::SearchParams, search_tables::HistoryTable, time_control::TimeController, transpositions::{PawnCache, TTable}};

const NO_DEBUG: bool = false;

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
    let (tc, _handle) = TimeController::new(TimeControl::Depth(depth), board);
    let mut history = HistoryTable::new();
    let mut pawn_cache = PawnCache::new();
    let search_params = SearchParams::default();
    let search = position.search::<NO_DEBUG>(
        &mut tt, 
        &mut history, 
        &mut pawn_cache,
        tc, 
        &search_params
    );

    println!("{board}");
    println!("{:17} {}", "FEN:".green(), fen);
    println!("{:17} {}", "Depth:".green(), depth);
    println!();

    println!("{:17} {}", "Best move:".bright_cyan(), search.pv[0]);
    println!("{:17} {}", "Score:".bright_cyan(), search.score);


    let nodes_visited: u32 = search.nodes;
    println!("{:17} {}", "Nodes visited:".blue(), nodes_visited);

    let time_spent = search.duration.as_millis();
    println!("{:17} {}ms", "Duration:".red(), time_spent);

    let knps = 1000 * search.nodes as u64 / time_spent as u64;
    println!("{:17} {}knps", "knps:".red(), knps);

    // Branching factors
    let root_bf = (nodes_visited as f32).powf(1.0 / (depth as f32));
    println!("{:17} {:.2}", "Branching factor:".red(), root_bf);

    // TT info
    println!("{:17} {}%", "TT occupancy".purple(), tt.occupancy());
    println!("{:17} {}", "TT inserts".purple(), tt.inserts());

    // Pawn cache info
    println!("{:17} {}%", "Pawn cache occupancy".purple(), pawn_cache.occupancy());
    println!("{:17} {}", "Pawn cache inserts".purple(), pawn_cache.inserts());
    println!("{:17} {}", "Pawn cache hits".purple(), pawn_cache.hits());
    println!("{:17} {}", "Pawn cache misses".purple(), pawn_cache.misses());

    println!("\n");
}
