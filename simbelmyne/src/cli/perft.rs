use chess::{board::Board, movegen::moves::Move};
use rayon::prelude::*;
use std::time::Instant;
use crate::cli::presets::{Preset, PerftPreset};
use anyhow::*;
use colored::*;

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
    if depth == 0 {
        return 1;
    };

    let moves = board.legal_moves();

    // OPTIMIZATION: If we're at the last step, we don't need to go through
    // playing every single move and returning back, just return the number of
    // legal moves directly.
    if BULK && depth == 1 {
        return moves.len();
    }

    moves
        .par_iter()
        .map(|mv| {
            let new_board = board.play_move(*mv);
            let nodes = perft::<BULK>(new_board, depth - 1);
            nodes
        })
        .sum()
}

pub fn perform_perft<const BULK: bool>(board: Board, depth: usize) -> PerftResult {
    let start = Instant::now();
    let nodes = perft::<BULK>(board, depth);
    let duration = start.elapsed();

    return PerftResult {
        nodes,
        duration: duration.as_micros(),
    };
}

pub fn perft_divide<const BULK: bool>(board: Board, depth: usize) -> Vec<(Move, usize)> {
    let moves = board.legal_moves();

    moves
        .par_iter()
        .map(|&mv| {
            let new_board = board.play_move(mv);
            let nodes = perft::<BULK>(new_board, depth - 1);
            (mv, nodes)
        })
        .collect()
}

const BULK: bool = true;

pub fn run_perft(depth: usize, fen: Option<String>, preset: Option<Preset>, all: bool) -> anyhow::Result<()> {
    println!(
        "🏃 {}",
        "Running Perft test\n----------------------------"
            .blue()
            .italic()
    );

    if all {
        for preset in Preset::all_presets() {
            run_preset(preset, depth)?;
        }
    } else if let Some(fen) = fen {
        run_fen(fen, depth)?;
    } else if let Some(preset) = preset {
        let preset = Preset::load_preset(preset);
        run_preset(preset, depth)?;
    }

    Ok(())
}

fn run_preset(preset: &PerftPreset, depth: usize) -> anyhow::Result<()>{
    let mut all_passed = true;
    let board: Board = preset.fen.parse().unwrap();

    println!("{}: {}", "Preset".green(), preset.name);
    println!("{}: {}", "Description".green(), preset.description);
    println!("{}: {}", "FEN".green(), preset.fen.italic());
    println!("{}:\n\n{board}\n\n", "Board".green());

    for depth in 0..=depth {
        let result = perform_perft::<BULK>(board, depth);
        let is_match = match preset.expected.get(depth) {
            Some(&expected) => result.nodes == expected,
            None => true,
        };

        all_passed &= is_match;

        print!("Depth {}: ", depth.to_string().blue());

        let expected = preset
            .expected
            .get(depth)
            .map(|&n| n.to_string())
            .unwrap_or("".to_string())
            .green();

        print!("expected {expected:>12} ");

        let found = if is_match {
            result.nodes.to_string().green()
        } else {
            result.nodes.to_string().red()
        };

        print!("found {found:>12} ");

        let mnps_str = format!("({:.3}Mnps)",result.mega_nps());
        print!("in {:5}ms {:>15}", result.millis(), mnps_str);

        if is_match {
            print!("{:>3}","🎉");
        } else {
            print!("{:>3}", "💥");
        }

        println!("");

    }
        println!("\n\n");

    if all_passed {
        Ok(())
    } else {
        return Err(anyhow!("Some perft results didn't match the expected value."));
    }
}

fn run_fen(fen: String, depth: usize) -> anyhow::Result<()> {
    let board: Board = fen.parse().unwrap();

    println!("{}: {}", "FEN".green(), fen.italic());
    println!("{}:\n\n{board}\n\n", "Board".green());

    for depth in 0..=depth {
        let result = perform_perft::<BULK>(board, depth);

        print!("Depth {}: ", depth.to_string().blue());

        print!("found {:>12} ", result.nodes.to_string().green());

        print!("in {:5}ms ({:.3}Mnps)", result.millis(), result.mega_nps());

        println!("");
    }
        println!("\n\n");

    Ok(())
}
