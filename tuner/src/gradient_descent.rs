//! This module holds all of the logic pertaining to the gradient descent
//! optimization of the weights stored in the `Tuner` struct.
//!
//! To optimize the evaluation weights, we perform a Gradient Descent
//! optimization of a certain cost function.
//!
//! The cost function we're trying to optimize for is the Mean Squared Error
//! (MSE) of the evaluation obtained from the current set of weights, versus
//! the reported outcome of a game (0: Loss, 0.5: Draw, 1.0 Win), across _all_
//! of the provided training data.
//!
//! To be able to compare our evaluation to these W/D/L values, we run our
//! (in principle unbounded) eval through a
//! [`sigmoid` function](https://en.wikipedia.org/wiki/Sigmoid_function) to map
//! them to the range [0.0, 1.0].
//!
//! For more information, see: https://github.com/AndyGrant/Ethereal/blob/master/Tuning.pdf

use crate::Activation;
use crate::DataEntry;
use crate::Score;
use crate::Tuner;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::IntoParallelRefMutIterator;
use rayon::prelude::IntoParallelIterator;
use rayon::prelude::ParallelIterator;

impl<const N: usize> Tuner<N> {
    /// Run a single step of Adam(W) optimization
    ///
    /// See: https://www.geeksforgeeks.org/adam-optimizer/
    pub fn tune(&mut self) {
        const BASE_LRATE: f32 = 1.0;
        const W: f32 = 0.00; //FIXME: Why won't this work with non-zero weight decay?
        const B1: f32 = 0.9;
        const B2: f32 = 0.999;
        const EPS: f32 = 0.00000001;

        // Compute gradient
        let grad = Self::gradient(&self.training_data, self.k);

        // Update grad squares and weights
        for (i, &grad_i) in grad.iter().enumerate() {
            // Add in weight-decay
            let grad_i = grad_i + self.weights[i] * W;

            // Compute momenta and velocities
            self.momenta[i] = self.momenta[i] * B1 + grad_i * (1.0 - B1);
            self.velocities[i] = self.velocities[i] * B2 + grad_i * grad_i * (1.0 - B2);

            // Compute adaptive learning rates
            let lrate = Score {
                mg: self.momenta[i].mg / (f32::sqrt(self.velocities[i].mg) + EPS),
                eg: self.momenta[i].eg / (f32::sqrt(self.velocities[i].eg) + EPS),
            } * BASE_LRATE;

            // Update weights
            self.weights[i] = self.weights[i] - lrate - self.weights[i] * W;
        }

        // Update evals on entries
        self.training_data.par_iter_mut().for_each(|(entry, eval)| {
            *eval = entry.evaluate(&self.weights);
        });
    }

    /// Compute the gradient of our cost function (the Mean Squared Error of
    /// the current weights' evaluation against the training data)
    ///
    /// For the details of how we get this expression for the gradient, refer
    /// to the Ethereal paper:
    /// https://github.com/AndyGrant/Ethereal/blob/master/Tuning.pdf
    fn gradient(data: &[(DataEntry, f32)], k: f32) -> [Score; N] {
        // Helper that updates the gradient with a single DataEntry
        let update_partial_gradient =
            |mut gradient: [Score; N], (entry, eval): &(DataEntry, f32)| {
                // Map the current eval to the interval [0.0, 1.0]
                let sigm = sigmoid(*eval, k);
                let factor =
                    -2.0 * k * (entry.result - sigm) * sigm * (1.0 - sigm) / data.len() as f32;

                for &Activation { idx, value } in &entry.activations {
                    gradient[idx] += Score {
                        mg: entry.mg_phase * value,
                        eg: entry.eg_phase * value * entry.eg_scaling,
                    } * factor;
                }

                gradient
            };

        // Helper that combines multiple partial gradient contributions together
        let combine_gradients = |mut gradient: [Score; N], partial: [Score; N]| {
            for (idx, score) in partial.iter().enumerate() {
                gradient[idx] += *score;
            }

            gradient
        };

        data.par_iter()
            .fold(|| [Score::default(); N], update_partial_gradient)
            .reduce(|| [Score::default(); N], combine_gradients)
    }

    /// Calculate the Mean Square Error for the training data using the current
    /// evaluations.
    pub fn mse(&self) -> f32 {
        self.training_data()
            .into_par_iter()
            .map(|(entry, eval)| {
                let delta = entry.result - sigmoid(*eval, self.k);
                delta * delta
            })
            .sum::<f32>()
            / self.training_data.len() as f32
    }
}

/// Fold a continuous variable between -Infinity, +Infinity to the range [0, 1]
/// according to a smooth stepwise function.
fn sigmoid(x: f32, k: f32) -> f32 {
    1.0 / (1.0 + f32::exp(-k * x))
}
