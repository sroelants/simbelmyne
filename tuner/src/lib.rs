use std::iter::Sum;
use std::ops::Add;
use std::ops::AddAssign;
use std::ops::Div;
use std::ops::Mul;
use std::ops::Sub;
use std::ops::SubAssign;
use std::str::FromStr;
use rayon::iter::IntoParallelRefIterator;
use rayon::prelude::IntoParallelIterator;
use rayon::prelude::IntoParallelRefMutIterator;
use rayon::prelude::ParallelIterator;

pub struct DataEntry {
    pub eg_scaling: f32,
    pub mg_phase: f32,
    pub eg_phase: f32,
    pub result: f32,
    pub activations: Vec<Activation>
}

////////////////////////////////////////////////////////////////////////////////
//
// Tuner struct
//
// A Tuner holds all the state we need to tune the weights provided by an 
// `impl Tune`. More importantly, it allows us to stop/continue tuning 
// so we can report and write intermediate results, etc...
//
////////////////////////////////////////////////////////////////////////////////
pub struct Tuner<const N: usize> {
    k: f32,
    weights: [Score; N],
    training_data: Vec<Entry>,
    momenta: [Score; N],
    velocities: [Score; N],
}

impl<const N: usize> Tuner<N> {
    pub fn new<T: Into<[Score; N]> + From<[Score; N]>> (weights: T, training_data: Vec<DataEntry>) -> Self {
        let weights = weights.into();
        let momenta: [Score; N] = [Score::default(); N];
        let velocities: [Score; N] = [Score::default(); N];
        let k = 0.01;

        let training_data = training_data
            .into_par_iter()
            .map(|data| {
                let mut entry = Entry {
                    mg_phase: data.mg_phase,
                    eg_phase: data.eg_phase,
                    eg_scaling: data.eg_scaling,
                    activations: data.activations,
                    result: data.result,
                    eval: 0.0,
                };

                entry.eval = entry.evaluate(&weights);
                entry

            })
            .collect::<Vec<_>>();

        Self {
            k, weights, momenta, velocities, training_data
        }
    }

    pub fn weights(&self) -> &[Score; N] {
        &self.weights
    }

    pub fn mse(&self) -> f32 {
        mse(&self.training_data, self.k)
    }

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
        self.training_data.par_iter_mut().for_each(|entry| {
            entry.eval = entry.evaluate(&self.weights);
        });
    }

    fn gradient(entries: &[Entry], k: f32) -> [Score; N] {
        let update_gradient = |mut gradient: [Score; N], entry: &Entry| {
            let sigm = sigmoid(entry.eval, k);
            let result: f32 = entry.result.into();
            let factor = -2.0 * k * (result - sigm) * sigm * (1.0 - sigm) / entries.len() as f32;

            for &Activation { idx, value } in &entry.activations {
                gradient[idx] += Score { 
                    mg: entry.mg_phase * value, 
                    eg: entry.eg_phase * value * entry.eg_scaling
                } * factor;
            }

            gradient
        };

        let combine_gradients = |mut gradient: [Score; N], partial: [Score; N]| {
            for (idx, score)  in partial.iter().enumerate() {
                gradient[idx] += *score;
            }

            gradient
        };

        entries
            .par_iter()
            .fold(  || [Score::default(); N], update_gradient)
            .reduce(|| [Score::default(); N], combine_gradients)
    }

    pub fn training_data(&self) -> &[Entry] {
        &self.training_data
    }
}

/// Calculate the mean square error for a given set of result entries, 
/// for a given sigmoid scaling function
fn mse(entries: &[Entry], k: f32) -> f32 {
    entries.into_par_iter().map(|entry| {
        let result: f32 = entry.result.into();
        let delta = result - sigmoid(entry.eval, k);
        delta * delta
    }).sum::<f32>() / entries.len() as f32
}

////////////////////////////////////////////////////////////////////////////////
//
// Game result entries
//
////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct Entry {
    /// The board position
    activations: Vec<Activation>,

    /// The static eval for this entry
    eval: f32,

    /// The result, encoded as 0, 0.5 or 1
    /// TODO: Maybe encode this an an enum instead?
    result: f32,

    /// The game phase
    mg_phase: f32,
    eg_phase: f32,
    eg_scaling: f32,
}

impl Entry {
    pub fn evaluate(&self, weights: &[Score]) -> f32 {
    let score = self.activations
        .iter()
        .map(|&Activation { value, idx }| weights[idx] * value)
        .sum::<Score>();

        self.mg_phase * score.mg + self.eg_phase * score.eg * self.eg_scaling
    }
}

////////////////////////////////////////////////////////////////////////////////
//
// Game result parsing
//
////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum GameResult { Win, Loss, Draw }

impl Into<f32> for GameResult {
    fn into(self) -> f32 {
        match self {
            GameResult::Win  => 1.0,
            GameResult::Draw => 0.5,
            GameResult::Loss => 0.0
        }
    }
}

impl FromStr for GameResult {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "[1.0]"     => Ok(Self::Win),
            "[0.5]"     => Ok(Self::Draw),
            "[0.0]"     => Ok(Self::Loss),
            _ => Err("Failed to parse game result")
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
//
// Activation
//
////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Copy, Clone)]
pub struct Activation {
    idx: usize,
    value: f32,
}

impl Activation {
    pub fn new(idx: usize, value: f32) -> Self { 
        Self { idx, value } 
    }
}

////////////////////////////////////////////////////////////////////////////////
//
// Score
//
////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Default, Copy, Clone)]
pub struct Score {
    pub mg: f32,
    pub eg: f32
}

impl Add for Score {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self { mg: self.mg + rhs.mg, eg: self.eg + rhs.eg }
    }
}

impl Add<f32> for Score {
    type Output = Self;

    fn add(self, rhs: f32) -> Self::Output {
        Self { mg: self.mg + rhs as f32, eg: self.eg + rhs as f32 }
    }
}

impl AddAssign<f32> for Score {
    fn add_assign(&mut self, rhs: f32) {
        self.mg += rhs as f32;
        self.eg += rhs as f32;
    }
}

impl AddAssign for Score {
    fn add_assign(&mut self, rhs: Self) {
        self.mg += rhs.mg;
        self.eg += rhs.eg;
    }
}

impl Sub for Score {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self { mg: self.mg - rhs.mg, eg: self.eg - rhs.eg }
    }
}

impl Sub<f32> for Score {
    type Output = Self;

    fn sub(self, rhs: f32) -> Self::Output {
        Self { mg: self.mg - rhs as f32, eg: self.eg - rhs as f32 }
    }
}

impl SubAssign<f32> for Score {
    fn sub_assign(&mut self, rhs: f32) {
        self.mg -= rhs as f32;
        self.eg -= rhs as f32;
    }
}

impl SubAssign for Score {
    fn sub_assign(&mut self, rhs: Self) {
        self.mg -= rhs.mg;
        self.eg -= rhs.eg;
    }
}


impl Mul<f32> for Score {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self { mg: rhs * self.mg, eg: rhs * self.eg }
    }
}

impl Mul for Score {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self { mg: self.mg * rhs.mg, eg: self.eg * rhs.eg }
    }
}

impl Div<f32> for Score {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Self { mg: self.mg / rhs, eg: self.eg / rhs }
    }
}

impl Sum for Score {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Score::default(), Score::add)
    }
}

fn sigmoid(x: f32, k: f32) -> f32 {
    1.0 / ( 1.0 + f32::exp(- k * x))
}
