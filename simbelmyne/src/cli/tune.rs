use std::path::PathBuf;
use std::fs::File;
use std::io::Write;
use crate::evaluate::tuner::EvalWeights;
use tuner::{Tune, Tuner};

pub fn run_tune(file: PathBuf, positions: Option<usize>, epochs: usize, output: Option<PathBuf>, interval: usize) {
    let weights = EvalWeights::default();
    let training_data = weights.load_entries(&file, positions).unwrap();
    eprintln!("Loaded {} entries", training_data.len());

    let mut tuner = Tuner::new(&weights, training_data);

    for epoch in 1..=epochs {
        if epoch % interval == 0 {
            eprintln!("Epoch {epoch} - Mean Squared Error: {}", tuner.mse());

            if let Some(ref path) = output {
                let mut file = File::create(&path).expect("Failed to open file");
                let new_weights = EvalWeights::from(*tuner.weights());
                write!(file, "{new_weights}").expect("Failed to write weights");
            }
        }

        tuner.tune();
    }
}
