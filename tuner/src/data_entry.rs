use crate::score::Score;

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
  pub idx: usize,

  /// The value of the activation
  pub value: f32,
}

impl Activation {
  pub fn new(idx: usize, value: f32) -> Self {
    Self { idx, value }
  }
}
