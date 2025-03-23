use std::fmt::Display;
use std::iter::Sum;
use std::ops::Add;
use std::ops::AddAssign;
use std::ops::Div;
use std::ops::Mul;
use std::ops::Sub;
use std::ops::SubAssign;
use std::str::FromStr;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::PathBuf;
use chess::board::Board;
use rayon::iter::IntoParallelRefIterator;
use rayon::prelude::IntoParallelIterator;
use rayon::prelude::IntoParallelRefMutIterator;
use rayon::prelude::ParallelBridge;
use rayon::prelude::ParallelIterator;

pub struct ActivationParams {
    pub eg_scaling: i32,
    pub components: Vec<Component>
}

pub trait Tune<const N: usize>: Display + Default + Sync + From<[Score; N]> {
    const DEFAULT_K: f32 = 0.1;
    fn weights(&self) -> [Score; N];
    fn activations(board: &Board) -> ActivationParams;

    /// Load game positions and their game outcome from a file.
    ///
    /// The expected format is a single game per line, with each line 
    /// consisting of: 
    /// `<fen> [0.0 | 0.5 | 1.0]`
    /// TODO: Make this more robust... Something like: several supported 
    /// formats, and we just look for a matching pattern _anywhere_ in the 
    /// string.
    fn load_entries(&self, file: &PathBuf, max_positions: Option<usize>) -> Result<Vec<Entry>, &'static str> {
        let file = BufReader::new(
            File::open(file).map_err(|_| "Failed to open file")?
        );

        let weights = self.weights();

        let entries = file.lines()
            .filter_map(|line| line.ok())
            .take(max_positions.unwrap_or(usize::MAX))
            .par_bridge()
            .map(|line| parse_line(&line))
            .map(|(board, result)| self.create_entry(&board, result, &weights))
            .collect::<Vec<_>>();

        Ok(entries)
    }

    fn create_entry(&self, board: &Board, result: GameResult, weights: &[Score]) -> Entry {
        let mg_phase = board.phase() as f32 / 24.0;
        let eg_phase = 1.0 - mg_phase;
        let activations = Self::activations(board);

        let mut entry = Entry {
            mg_phase,
            eg_phase,
            components: activations.components,
            eg_scaling: activations.eg_scaling as f32 / 128.0,
            result,
            eval: 0.0
        };

        entry.eval = entry.evaluate(weights);
        entry
    }
}

fn parse_line(line: &str) -> (Board, GameResult) {
    let mut parts = line.split(' ');
    let fen = parts.by_ref().take(6).collect::<Vec<_>>().join(" ");
    let result = parts.by_ref().collect::<String>();

    let board: Board = fen.parse().expect("Invalid FEN");
    let result: GameResult = result.parse().expect("Invalid WLD");

    (board, result)
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
    pub fn new(tune: &impl Tune<N>, training_data: Vec<Entry>) -> Self {
        let weights = tune.weights();
        let momenta: [Score; N] = [Score::default(); N];
        let velocities: [Score; N] = [Score::default(); N];
        let k = 0.01;

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
            // println!("sigmoid(eval): {sigm}, prediction: {result}");
            // println!("k {k}");
            let factor = -2.0 * k * (result - sigm) * sigm * (1.0 - sigm) / entries.len() as f32;

            for &Component { idx, value } in &entry.components {
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
    components: Vec<Component>,

    /// The static eval for this entry
    eval: f32,

    /// The result, encoded as 0, 0.5 or 1
    /// TODO: Maybe encode this an an enum instead?
    result: GameResult,

    /// The game phase
    mg_phase: f32,
    eg_phase: f32,
    eg_scaling: f32,
}

impl Entry {
    pub fn evaluate(&self, weights: &[Score]) -> f32 {
    let score = self.components
        .iter()
        .map(|&Component { value, idx }| weights[idx] * value)
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
// Component
//
////////////////////////////////////////////////////////////////////////////////
#[derive(Debug, Copy, Clone)]
pub struct Component {
    idx: usize,
    value: f32,
}

impl Component {
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
