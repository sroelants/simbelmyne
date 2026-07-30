use std::{
  fs::File,
  io::{BufRead, BufReader},
  path::PathBuf,
  str::FromStr,
};

use chess::board::Board;
use rayon::iter::{ParallelBridge, ParallelIterator};

use crate::{
  data_entry::{Activation, DataEntry},
  score::Score,
};

pub fn load_entries(
  file: PathBuf,
  positions: Option<usize>,
  activations: impl Fn(Board) -> (Vec<Activation>, i32) + Sync,
) -> Vec<DataEntry> {
  // Load the training data from the input file, and parse them into
  // `tuner::DataEntry`s that we can pass into `tuner::Tuner`.
  let entries: Vec<DataEntry> =
    BufReader::new(File::open(file).expect("Failed to open file: {file}"))
      .lines()
      .take(positions.unwrap_or(usize::MAX))
      .filter_map(|line| line.ok())
      .par_bridge()
      .map(|line| parse_line(&line))
      .map(|(board, result)| {
        let (activations, eg_scaling) = activations(board);
        DataEntry {
          eg_scaling: eg_scaling as f32 / 128.0,
          mg_phase: board.phase() as f32 / 24.0,
          eg_phase: (24.0 - board.phase() as f32) / 24.0,
          activations,
          eval: 0,
          result: result.into(),
        }
      })
      .collect();

  entries
}

pub fn load_chef_entries(
  file: PathBuf,
  positions: Option<usize>,
  activations: impl Fn(Board) -> (Vec<Activation>, i32) + Sync,
) -> Vec<DataEntry> {
  // Load the training data from the input file, and parse them into
  // `tuner::DataEntry`s that we can pass into `tuner::Tuner`.
  let entries: Vec<DataEntry> =
    BufReader::new(File::open(file).expect("Failed to open file: {file}"))
      .lines()
      .take(positions.unwrap_or(usize::MAX))
      .filter_map(|line| line.ok())
      .par_bridge()
      .map(|line| parse_chef_line(&line))
      .map(|(board, eval, result)| {
        let (activations, eg_scaling) = activations(board);
        DataEntry {
          eg_scaling: eg_scaling as f32 / 128.0,
          mg_phase: board.phase() as f32 / 24.0,
          eg_phase: (24.0 - board.phase() as f32) / 24.0,
          activations,
          eval,
          result: result.into(),
        }
      })
      .collect();

  entries
}

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

/// Parse a chef (TM) input line into a (Board, Eval, GameResult) triple
///
/// TODO: Make this more robust towards other input formats?
fn parse_chef_line(line: &str) -> (Board, i32, GameResult) {
  let mut parts = line.split('|');
  let fen: &str = parts.next().unwrap().trim();
  let score: i32 = parts.next().unwrap().trim().parse().unwrap();
  let wdl: GameResult =
    parts.next().unwrap().trim().parse::<f32>().unwrap().into();
  let board: Board = fen.parse().expect("Invalid FEN");

  (board, score, wdl)
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

impl From<f32> for GameResult {
  fn from(value: f32) -> Self {
    match value {
      1.0 => Self::Win,
      0.5 => Self::Draw,
      0.0 => Self::Loss,
      _ => panic!(),
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
