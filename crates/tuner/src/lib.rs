use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;
pub use score::Score;

mod gradient_descent;
mod score;

/// A `Tuner` takes a set of initial weights and a set of training data, and
/// exposes a `Tuner::tune` method that runs a single iteration of an Adam
/// optimization.
pub struct Tuner<const N: usize> {
  k: f32,
  weights: [Score; N],
  training_data: Vec<(DataEntry, f32)>,
  momenta: [Score; N],
  velocities: [Score; N],
}

impl<const N: usize> Tuner<N> {
  pub fn new<T: Into<[Score; N]> + From<[Score; N]>>(
    weights: T,
    training_data: Vec<DataEntry>,
  ) -> Self {
    let weights = weights.into();
    let momenta: [Score; N] = [Score::default(); N];
    let velocities: [Score; N] = [Score::default(); N];
    let k = 0.01;

    let training_data = training_data
      .into_par_iter()
      .map(|data| {
        let eval = data.evaluate(&weights);
        (data, eval)
      })
      .collect::<Vec<_>>();

    Self {
      k,
      weights,
      momenta,
      velocities,
      training_data,
    }
  }

  pub fn weights(&self) -> &[Score; N] {
    &self.weights
  }

  pub fn training_data(&self) -> &[(DataEntry, f32)] {
    &self.training_data
  }
}

////////////////////////////////////////////////////////////////////////////////
//
// Helpers
//
////////////////////////////////////////////////////////////////////////////////

/// A bare entry holding only the reusable data, to be provided when
/// constructing a Tuner.
#[derive(Debug)]
pub struct DataEntry {
  pub eg_scaling: f32,
  pub mg_phase: f32,
  pub eg_phase: f32,
  pub result: f32,
  pub activations: Vec<Activation>,
}

impl DataEntry {
  pub fn evaluate(&self, weights: &[Score]) -> f32 {
    let score = self
      .activations
      .iter()
      .map(|&Activation { value, idx }| weights[idx] * value)
      .sum::<Score>();

    self.mg_phase * score.mg + self.eg_phase * score.eg * self.eg_scaling
  }
}

/// The activation of a given eval feature
#[derive(Debug, Copy, Clone)]
pub struct Activation {
  /// The index of the eval feature, which ties it to a weight in a set of
  /// tunable weights.
  idx: usize,

  /// The value of the activation
  value: f32,
}

impl Activation {
  pub fn new(idx: usize, value: f32) -> Self {
    Self { idx, value }
  }
}
