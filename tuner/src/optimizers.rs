use crate::data_entry::DataEntry;
use crate::{batches::Batch, data_entry::Activation, score::Score};
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::IntoParallelRefMutIterator;
use rayon::prelude::IntoParallelIterator;
use rayon::prelude::ParallelIterator;

const EPS: f32 = 0.0000001;

// TODO: This will implement deserialize, and we will only really read it from
// the config.
#[derive(Copy, Clone)]
pub struct AdamConfig {
  /// The base learning rate
  pub lrate: f32,
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
  pub fn new(lrate: f32, b1: f32, b2: f32, k: f32) -> Self {
    Self {
      lrate,
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
  pub fn new(config: AdamConfig) -> Self {
    Self {
      config,
      m: [Score::default(); N],
      v: [Score::default(); N],
      w: [Score::default(); N],
    }
  }
  pub fn run(mut self, batch: &Batch) -> [Score; N] {
    let loss = &self.config.loss;
    let AdamConfig {
      lrate, b1, b2, k, ..
    } = self.config;

    // Compute the gradient
    let mut grad = [Score::default(); N];

    // TODO: This should become a rayon parallelized sum
    for entry in batch.iter() {
      let eval = entry.evaluate(&self.w);
      let sig = sigmoid(eval, k);
      let dsig = k * sig * (1.0 - sig);
      let dloss = loss.grad(sig, entry.result);
      let factor = dloss * dsig / batch.size() as f32;

      for &Activation { idx, value } in &entry.activations {
        grad[idx] += Score {
          mg: entry.mg_phase * value,
          eg: entry.eg_phase * value * entry.eg_scaling,
        } * factor;
      }
    }

    // Update the weights
    for i in 0..N {
      self.m[i] = self.m[i] * b1 + grad[i] * (1.0 - b1);
      self.v[i] = self.v[i] * b2 + grad[i] * grad[i] * (1.0 - b2);

      let update = Score {
        mg: self.m[i].mg / (f32::sqrt(self.v[i].mg) + EPS),
        eg: self.m[i].eg / (f32::sqrt(self.v[i].eg) + EPS),
      };

      self.w[i] -= update * lrate;
    }

    self.w
  }
}

//-----------------------------------------------------------------------------
//
// Loss functions
//
//-----------------------------------------------------------------------------

#[derive(Copy, Clone)]
pub enum LossFn {
  MeanSquareError,
}

impl LossFn {
  pub fn batch_loss<'a, const N: usize>(
    &self,
    entries: &[DataEntry],
    weights: &[Score; N],
  ) -> f32 {
    entries
      .par_iter()
      .map(|entry| entry.evaluate(weights))
      .sum::<f32>()
      / entries.len() as f32
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
