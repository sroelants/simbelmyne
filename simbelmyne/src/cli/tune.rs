use std::path::PathBuf;
use crate::evaluate::tuner::EvalWeights;
use crate::tuner::Tune;

const DEBUG: bool = true;

pub fn run_tune(file: PathBuf, positions: usize, epochs: usize) {
    let mut weights = EvalWeights::default();
    let mut entries = weights.load_entries(&file, positions).unwrap();
    eprintln!("Loaded {} entries", entries.len());
    weights.tune::<DEBUG>(&mut entries, epochs);
    println!("{weights}");
}
