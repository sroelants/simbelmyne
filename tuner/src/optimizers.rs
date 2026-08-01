use crate::data_entry::DataEntry;
use crate::{batches::Batch, data_entry::Activation, score::Score};
// use rayon::iter::IntoParallelRefIterator;
use rayon::prelude::*;

const EPS: f32 = 1.0e-8;

// TODO: This will implement deserialize, and we will only really read it from
// the config.
#[derive(Copy, Clone)]
pub struct AdamConfig {
  /// The base learning rate
  pub lrate: f32,
  /// The wdl mix to optimize for
  pub wdl: f32,
  /// The weight of the momenta in the updates
  pub b1: f32,
  /// The weight of the velocities in the updates
  pub b2: f32,
  /// The eval scaling used in the sigmoid function
  pub k: f32,
  /// The loss function to optimize against
  pub loss: LossFn,
}

impl AdamConfig {
  pub fn new(lrate: f32, wdl: f32, b1: f32, b2: f32, k: f32) -> Self {
    Self {
      lrate,
      wdl,
      b1,
      b2,
      k,
      loss: LossFn::MeanSquareError,
    }
  }
}

impl Default for AdamConfig {
  fn default() -> Self {
    Self {
      lrate: 1.0,
      wdl: 1.0,
      b1: 0.9,
      b2: 0.999,
      k: 0.01,
      loss: LossFn::MeanSquareError,
    }
  }
}

pub struct Adam<const N: usize> {
  config: AdamConfig,
  w: [Score; N],
  m: [Score; N],
  v: [Score; N],
}

impl<const N: usize> Adam<N> {
  pub fn new(w: [Score; N], config: AdamConfig) -> Self {
    Self {
      config,
      m: [Score::default(); N],
      v: [Score::default(); N],
      w,
    }
  }
  pub fn run(mut self, batch: &Batch) -> [Score; N] {
    let AdamConfig { lrate, b1, b2, .. } = self.config;

    // This is the expensive bit...
    let k = self.config.k;
    let loss = self.config.loss;
    let wdl = self.config.wdl;
    let grad = compute_gradient::<N>(batch, &self.w, k, loss, wdl);

    // Update the weights
    for i in 0..N {
      self.m[i] = self.m[i] * b1 + grad[i] * (1.0 - b1);
      self.v[i] = self.v[i] * b2 + grad[i] * grad[i] * (1.0 - b2);

      let update = Score {
        mg: self.m[i].mg / (f32::sqrt(self.v[i].mg) + EPS),
        eg: self.m[i].eg / (f32::sqrt(self.v[i].eg) + EPS),
      };

      self.w[i] -= (update + self.w[i] * 0.01) * lrate;
    }

    self.w
  }
}

// TODO: Create a (k, loss, wdl) wrapper?
fn compute_gradient<const N: usize>(
  batch: &Batch,
  w: &[Score; N],
  k: f32,
  loss: LossFn,
  wdl: f32,
) -> [Score; N] {
  // Helper that updates the gradient with a single DataEntry
  let update_partial_gradient =
    |mut grad: [Score; N], entries: &[DataEntry]| {
      for entry in entries {
        let eval = entry.evaluate(w);
        let sig = sigmoid(eval, k);
        let target = sig * (1.0 - wdl) + entry.result * wdl;

        let dsig = k * sig * (1.0 - sig);
        let dloss = loss.grad(sig, target);
        let factor = dloss * dsig / batch.size() as f32;

        for &Activation { idx, value } in &entry.activations {
          grad[idx] += Score {
            mg: entry.mg_phase * value,
            eg: entry.eg_phase * value * entry.eg_scaling,
          } * factor;
        }
      }

      grad
    };

  // Helper that combines multiple partial gradient contributions together
  let combine_gradients = |mut grad: [Score; N], partial: [Score; N]| {
    for (idx, score) in partial.iter().enumerate() {
      grad[idx] += *score;
    }

    grad
  };

  batch
    .entries
    .par_chunks(1024)
    .fold(|| [Score::default(); N], update_partial_gradient)
    .reduce(|| [Score::default(); N], combine_gradients)
}

//-----------------------------------------------------------------------------
// Loss functions
//-----------------------------------------------------------------------------

#[derive(Copy, Clone)]
pub enum LossFn {
  MeanSquareError,
}

impl LossFn {
  pub fn batch_loss<'a, const N: usize>(
    &self,
    batch: &[DataEntry],
    weights: &[Score; N],
    k: f32,
  ) -> f32 {
    batch
      .par_iter()
      .map(|entry| {
        let sig = sigmoid(entry.evaluate(weights), k);
        self.at(sig, entry.result)
      })
      .sum::<f32>()
      / batch.len() as f32
  }

  pub fn at(&self, x: f32, res: f32) -> f32 {
    match self {
      Self::MeanSquareError => {
        let err = x - res;
        err * err
      }
    }
  }

  pub fn grad(&self, x: f32, res: f32) -> f32 {
    match self {
      Self::MeanSquareError => {
        let err = x - res;
        2.0 * err
      }
    }
  }
}

//-----------------------------------------------------------------------------
//
// Helpers
//
//-----------------------------------------------------------------------------

pub fn sigmoid(x: f32, k: f32) -> f32 {
  1.0 / (1.0 + f32::exp(-k * x))
}
