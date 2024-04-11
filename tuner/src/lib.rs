use std::fmt::Display;
use std::iter::Sum;
use std::ops::Add;
use std::ops::AddAssign;
use std::ops::Mul;
use std::ops::Sub;
use std::ops::SubAssign;
use std::str::FromStr;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::PathBuf;
use chess::board::Board;
use rayon::prelude::IntoParallelIterator;
use rayon::prelude::IntoParallelRefMutIterator;
use rayon::prelude::ParallelBridge;
use rayon::prelude::ParallelIterator;

pub trait Tune<const N: usize>: Display + Default + Sync + From<[Score; N]> {
    const DEFAULT_K: f32 = 0.1;
    fn weights(&self) -> [Score; N];
    fn components(board: &Board) -> Vec<Component>;

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

        let entries = file.lines()
            .take(max_positions.unwrap_or(usize::MAX))
            .par_bridge()
            .map(|line| {
                let line = line.unwrap();
                let mut parts = line.split(' ');
                let fen = parts.by_ref().take(6).collect::<Vec<_>>().join(" ");
                let result = parts.by_ref().collect::<String>();

                let board: Board = fen.parse().expect("Invalid FEN");
                let result: GameResult = result.parse().expect("Invalid WLD");

                let weights = self.weights();
                let phase = board.phase();
                let components = Self::components(&board);
                let eval = evaluate_components(&weights, &components, phase);

                Entry {
                    components,
                    eval,
                    result,
                    phase: board.phase(),
                }
            })
            .collect::<Vec<_>>();

        Ok(entries)
    }

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
    grad_squares: [Score; N],
}

impl<const N: usize> Tuner<N> {
    pub fn new(tune: &impl Tune<N>, training_data: Vec<Entry>) -> Self {
        let weights = tune.weights();
        let grad_squares: [Score; N] = [Score::default(); N];
        let k = optimal_k(&training_data);
        eprintln!("Optimal k: {k}");

        Self {
            k, weights, grad_squares, training_data
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
        const EPS: f32 = 0.00000001;

        // Compute gradient
        let grad = Self::gradient(&self.training_data, self.k);

        // Update grad squares and weights
        for (i, &grad_i) in grad.iter().enumerate() {
            self.grad_squares[i] += grad_i * grad_i;

            let lrate = Score { 
                mg: BASE_LRATE / f32::sqrt(self.grad_squares[i].mg + EPS),
                eg: BASE_LRATE / f32::sqrt(self.grad_squares[i].eg + EPS),
            };

            self.weights[i] -= grad_i * lrate;
        }

        // Update evals on entries
        self.training_data.par_iter_mut().for_each(|entry| {
            entry.eval = evaluate_components(&self.weights, &entry.components, entry.phase)
        });
    }

    // NOTE: I really want this to be parallelized, but I kept getting stack
    // overflows. Not sure if I can just restrict the number of threads that
    // get spawned?
    fn gradient(entries: &[Entry], k: f32) -> [Score; N] {
        entries.iter().fold([Score::default(); N], |mut gradient, entry| {
            let sigm = sigmoid(entry.eval, k);
            let result: f32 = entry.result.into();
            let factor = -2.0 * k * (result - sigm) * sigm * (1.0 - sigm) / entries.len() as f32;

            for &Component { idx, value } in &entry.components {
                gradient[idx] += Score { 
                    mg: entry.phase as f32 * value, 
                    eg: (24.0 - entry.phase as f32) * value 
                } * factor;
            }

            gradient
        })
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

fn optimal_k(entries: &[Entry]) -> f32 {
    const PRECISION: usize = 10;
    let mut start = 0.0;
    let mut end = 10.0;
    let mut stepsize = 1.0;
    let mut min_err = f32::MAX;
    let mut best_k: f32 = end;

    for _ in 0..PRECISION {
        let mut current = start;

        while current < end {
            let err = mse(entries, current);

            if err < min_err {
                min_err = err;
                best_k = current;
            }

            current += stepsize;
        }

        start = best_k - stepsize;
        end = best_k + stepsize;
        stepsize /= 10.0;
    }

    best_k
}

pub fn evaluate_components(weights: &[Score], components: &[Component], phase: u8) -> f32 {
    components
        .iter()
        .map(|&Component { value, idx }| weights[idx] * value)
        .sum::<Score>()
        .lerp(phase)
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
    phase: u8,
}

////////////////////////////////////////////////////////////////////////////////
//
// Game result parsing
//
////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
enum GameResult { Win, Loss, Draw }

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
            "[0.5]" => Ok(Self::Draw),
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

impl Score {
    /// Interpolate between the midgame and endgame score according to a
    /// given `phase` which is a value between 0 and 24.
    fn lerp(&self, phase: u8) -> f32 {
        (phase as f32 * self.mg + (24.0 - phase as f32) * self.eg) / 24.0
    }
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

impl Sum for Score {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Score::default(), Score::add)
    }
}

fn sigmoid(x: f32, k: f32) -> f32 {
    1.0 / ( 1.0 + f32::exp(- k * x))
}
