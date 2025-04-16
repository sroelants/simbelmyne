use chess::board::Board;
use colored::Colorize;
use engine::evaluate::params::PARAMS;
use engine::evaluate::tuner::EvalTrace;
use engine::evaluate::tuner::EvalWeights;
use rayon::iter::ParallelBridge;
use rayon::iter::ParallelIterator;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;
use std::time::Instant;
use tuner::Activation;
use tuner::DataEntry;
use tuner::Tuner;

pub fn run_tune(
  file: PathBuf,
  positions: Option<usize>,
  epochs: usize,
  output: Option<PathBuf>,
  interval: usize,
  zero: bool,
) {
  // Set a custom stack size for each thread in rayon's thread pool
  rayon::ThreadPoolBuilder::new()
    .stack_size(8_000_000) // 8mb
    .build_global()
    .unwrap();

  let start = Instant::now();
  eprintln!("Loading input from {}... ", file.to_str().unwrap().blue());

  // Load the training data from the input file, and parse them into
  // `tuner::DataEntry`s that we can pass into `tuner::Tuner`.
  let training_data =
    BufReader::new(File::open(file).expect("Failed to open file: {file}"))
      .lines()
      .take(positions.unwrap_or(usize::MAX))
      .filter_map(|line| line.ok())
      .par_bridge()
      .map(|line| parse_line(&line))
      .map(|(board, result)| create_data_entry(board, result))
      .collect();

  let mut tuner = Tuner::new(
    if zero { EvalWeights::default() } else { PARAMS },
    training_data,
  );

  eprintln!(
    "{} Loaded {} entries",
    start.elapsed().pretty(),
    tuner.training_data().len().to_string().blue()
  );

  // Start tuning!
  for epoch in 0..=epochs {
    tuner.tune();

    // Print progress and output weights to file every `interval` epochs
    if epoch % interval == 0 {
      eprintln!(
        "{} Epoch {epoch: <4} - MSE: {}",
        start.elapsed().pretty(),
        tuner.mse()
      );

      if let Some(ref path) = output {
        write_output(path, &tuner);
      }
    }
  }
}

////////////////////////////////////////////////////////////////////////////////
//
// Parsing
//
////////////////////////////////////////////////////////////////////////////////

/// Parse an input line into a (Board, GameResult) pair
///
/// TODO: Make this more robust towards other input formats?
fn parse_line(line: &str) -> (Board, GameResult) {
  let mut parts = line.split(' ');
  let fen = parts.by_ref().take(6).collect::<Vec<_>>().join(" ");
  let result = parts.by_ref().collect::<String>();

  let board: Board = fen.parse().expect("Invalid FEN");
  let result: GameResult = result.parse().expect("Invalid WLD");

  (board, result)
}

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum GameResult {
  Win,
  Loss,
  Draw,
}

impl Into<f32> for GameResult {
  fn into(self) -> f32 {
    match self {
      GameResult::Win => 1.0,
      GameResult::Draw => 0.5,
      GameResult::Loss => 0.0,
    }
  }
}

impl FromStr for GameResult {
  type Err = &'static str;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "[1.0]" => Ok(Self::Win),
      "[0.5]" => Ok(Self::Draw),
      "[0.0]" => Ok(Self::Loss),
      _ => Err("Failed to parse game result"),
    }
  }
}

////////////////////////////////////////////////////////////////////////////////
//
// Utilities
//
////////////////////////////////////////////////////////////////////////////////

/// Helper trait for pretty printing durations for our output
trait Pretty {
  fn pretty(&self) -> String;
}

impl Pretty for Duration {
  /// Pretty-print a duration timestamp
  fn pretty(&self) -> String {
    let mins = self.as_secs() / 60;
    let secs = self.as_secs() % 60;

    format!("[{mins:0>2}:{secs:0>2}]")
      .bright_black()
      .to_string()
  }
}

/// Write the current tuner state to the provided output file
fn write_output(path: &PathBuf, tuner: &Tuner<{ EvalWeights::LEN }>) {
  let mut file = File::create(&path).expect("Failed to open file");
  let new_weights = EvalWeights::from(*tuner.weights());

  write!(
    file,
    "\
use crate::evaluate::S;
use crate::s;
use super::tuner::EvalWeights;

pub const PARAMS: EvalWeights = {new_weights:#?};"
  )
  .unwrap();
}

/// Turn a `Board` and `GameResult` into a `DataEntry` that we can pass to
/// the `Tuner`.
fn create_data_entry(board: Board, result: GameResult) -> DataEntry {
  use bytemuck::cast;
  let trace = EvalTrace::new(&board);
  let trace = cast::<EvalTrace, [i32; EvalWeights::LEN + 1]>(trace);

  let eg_scaling = trace[0];

  let activations = trace[1..]
    .into_iter()
    .enumerate()
    .filter(|&(_, &value)| value != 0)
    .map(|(idx, &value)| Activation::new(idx, value as f32))
    .collect::<Vec<_>>();

  DataEntry {
    eg_scaling: eg_scaling as f32 / 128.0,
    mg_phase: board.phase() as f32 / 24.0,
    eg_phase: (24.0 - board.phase() as f32) / 24.0,
    activations,
    result: result.into(),
  }
}
