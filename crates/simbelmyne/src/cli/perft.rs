use anyhow::*;
use chess::board::Board;
use colored::*;
use std::time::Instant;

use engine::tests::PERFT_RESULTS;

pub struct PerftResult {
  pub nodes: u64,
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

pub fn perform_perft<const BULK: bool>(
  board: Board,
  depth: usize,
) -> PerftResult {
  let start = Instant::now();
  let nodes = board.perft(depth);
  let duration = start.elapsed();

  return PerftResult {
    nodes,
    duration: duration.as_micros(),
  };
}

const BULK: bool = true;

pub fn run_perft(
  depth: usize,
  fen: Option<String>,
  all: bool,
) -> anyhow::Result<()> {
  if all {
    run_suite();
  } else if let Some(fen) = fen {
    run_fen(fen, depth)?;
  }

  Ok(())
}

fn run_suite() {
  for entry in PERFT_RESULTS {
    let mut parts = entry.split(',');
    let fen = parts.next().unwrap();
    let results: Vec<u64> = parts.map(|p| p.trim().parse().unwrap()).collect();
    let board: Board = fen.parse().unwrap();

    print!("{:<100} ", fen.blue());

    for (i, &expected) in results.iter().enumerate() {
      let found = board.perft(i + 1);
      if found == expected {
        print!("{} ", found.to_string().green());
      } else {
        print!("{} ", found.to_string().red());
      }
    }

    println!("");
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
