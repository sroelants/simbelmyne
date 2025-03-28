use std::time::Duration;
use std::{path::PathBuf, time::Instant};
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use crate::evaluate::tuner::{EvalTrace, EvalWeights};
use chess::board::Board;
use rayon::iter::{ParallelBridge, ParallelIterator};
use tuner::{Activation, DataEntry, GameResult, Tuner};
use colored::Colorize;

pub fn run_tune(file: PathBuf, positions: Option<usize>, epochs: usize, output: Option<PathBuf>, interval: usize) {
    rayon::ThreadPoolBuilder::new()
        .stack_size(8_000_000) // 8mb
        .build_global()
        .unwrap();

    let start = Instant::now();

    let weights = EvalWeights::default();

    eprintln!("Loading input from {}... ", file.to_str().unwrap().blue());
    let file = BufReader::new(File::open(file).expect("Failed to open file: {file}"));

    let training_data = file
        .lines()
        .take(positions.unwrap_or(usize::MAX))
        .filter_map(|line| line.ok())
        .par_bridge()
        .map(|line| parse_line(&line))
        .map(|(board, result)| create_data_entry(board, result))
        .collect();

    let mut tuner = Tuner::new(weights, training_data);

    eprintln!(
        "{} Loaded {} entries", 
        start.elapsed().pretty(), 
        tuner.training_data().len().to_string().blue()
    );


    for epoch in 1..=epochs {
        if epoch % interval == 0 {
            eprintln!(
                "{} Epoch {epoch} - Mean Squared Error: {}", 
                start.elapsed().pretty(), 
                tuner.mse()
            );

            if let Some(ref path) = output {
                let mut file = File::create(&path).expect("Failed to open file");
                let new_weights = EvalWeights::from(*tuner.weights());

                write!(file, "use crate::evaluate::S;
use crate::s;
use super::tuner::EvalWeights;

pub const PARAMS: EvalWeights = {new_weights:#?};"
                ).unwrap();
            }
        }

        tuner.tune();
    }
}

trait Pretty {
    fn pretty(&self) -> String;
}

impl Pretty for Duration {
    fn pretty(&self) -> String {
        let mins = self.as_secs() / 60;
        let secs = self.as_secs() % 60;

        format!("[{mins:0>2}:{secs:0>2}]").bright_black().to_string()
    }
}

fn parse_line(line: &str) -> (Board, GameResult) {
    let mut parts = line.split(' ');
    let fen = parts.by_ref().take(6).collect::<Vec<_>>().join(" ");
    let result = parts.by_ref().collect::<String>();

    let board: Board = fen.parse().expect("Invalid FEN");
    let result: GameResult = result.parse().expect("Invalid WLD");

    (board, result)
}

fn create_data_entry(board: Board, result: GameResult) -> DataEntry {
    use bytemuck::cast;
    let trace = EvalTrace::new(&board);
    let trace = cast::<EvalTrace, [i32; EvalWeights::LEN+1]>(trace);

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
