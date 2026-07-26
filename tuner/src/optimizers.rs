use crate::Score;

const EPS: f32 = 0.0000001;

// TODO: This will implement deserialize, and we will only really read it from
// the config.
pub struct AdamConfig {
  /// The base learning rate
  lrate: f32,
  /// The weight of the momenta in the updates
  b1: f32,
  /// The weight of the velocities in the updates
  b2: f32,
  /// The eval scaling used in the sigmoid function
  k: f32,
}

impl AdamConfig {
  pub fn new(lrate: f32, b1: f32, b2: f32, k: f32) -> Self {
    Self { lrate, b1, b2, k }
  }
}

impl Default for AdamConfig {
  fn default() -> Self {
    Self {
      lrate: 1.0,
      b1: 0.9,
      b2: 0.999,
      k: 0.01,
    }
  }
}

pub struct Adam<const N: usize> {
  momenta: [Score; N],
  velocities: [Score; N],
}
