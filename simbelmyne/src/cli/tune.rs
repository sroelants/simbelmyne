use std::time::Duration;
use std::{path::PathBuf, time::Instant};
use std::fs::File;
use std::io::Write;
// use crate::evaluate::params::PARAMS;
use crate::evaluate::tuner::EvalWeights;
use tuner::{Tune, Tuner};
use colored::Colorize;

pub fn run_tune(file: PathBuf, positions: Option<usize>, epochs: usize, output: Option<PathBuf>, interval: usize) {
    rayon::ThreadPoolBuilder::new()
        .stack_size(8_000_000) // 8mb
        .build_global()
        .unwrap();

    let start = Instant::now();

    // Should we tune from 0, always?
    let weights = EvalWeights::default();
    // let weights = PARAMS;

    eprintln!("Loading input from {}... ", file.to_str().unwrap().blue());
    let training_data = weights.load_entries(&file, positions).unwrap();

    eprintln!(
        "{} Loaded {} entries", 
        start.elapsed().pretty(), 
        training_data.len().to_string().blue()
    );

    let mut tuner = Tuner::new(&weights, training_data);

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
