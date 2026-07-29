use chess::board::Board;
use engine::evaluate::params::PARAMS;
use engine::evaluate::tuner::EvalTrace;
use engine::evaluate::tuner::EvalWeights;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use tuner::data_entry::Activation;
use tuner::score::Score;
use tuner::Tuner;

pub fn run_tune(
  file: PathBuf,
  positions: Option<usize>,
  epochs: usize,
  output: Option<PathBuf>,
  zero: bool,
) {
  // Set a custom stack size for each thread in rayon's thread pool
  rayon::ThreadPoolBuilder::new()
    .stack_size(8_000_000) // 8mb
    .build_global()
    .unwrap();

  let weights = if zero {
    [Score::default(); EvalWeights::LEN]
  } else {
    PARAMS.into()
  };

  let mut tuner = Tuner::new(epochs, 1 << 16);
  tuner.load(file, positions, get_activations);

  let tuned_weights = tuner.run(weights);

  if let Some(ref path) = output {
    write_output(path, &tuned_weights);
  }
}

/// Write the current tuner state to the provided output file
fn write_output(path: &PathBuf, weights: &[Score; EvalWeights::LEN]) {
  let mut file = File::create(&path).expect("Failed to open file");
  let new_weights = EvalWeights::from(*weights);

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
fn get_activations(board: Board) -> (Vec<Activation>, i32) {
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

  (activations, eg_scaling)
}
