use bytemuck::Pod;
use bytemuck::Zeroable;
use chess::board::Board;
use tuner::Score;
use std::fmt;
use super::Eval;
use super::Score as EvalScore;
use crate::evaluate::S;

////////////////////////////////////////////////////////////////////////////////
//
// EvalWeights
//
////////////////////////////////////////////////////////////////////////////////

#[derive(Pod, Zeroable)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(C)]
pub struct EvalWeights {
    pub piece_values: [S; 6],
    pub pawn_psqt: [S; 64],
    pub knight_psqt: [S; 64],
    pub bishop_psqt: [S; 64],
    pub rook_psqt: [S; 64],
    pub queen_psqt: [S; 64],
    pub king_psqt: [S; 64],
    pub passed_pawn: [S; 64],
    pub knight_mobility: [S; 9],
    pub bishop_mobility: [S; 14],
    pub rook_mobility: [S; 15],
    pub queen_mobility: [S; 28],
    pub virtual_mobility: [S; 28],
    pub king_zone: [S; 16],
    pub isolated_pawn: [S; 8],
    pub doubled_pawn: [S; 8],
    pub protected_pawn: [S; 8],
    pub phalanx_pawn: [S; 8],
    pub bishop_pair: S,
    pub rook_open_file: S,
    pub rook_semiopen_file: S,
    pub connected_rooks: S,
    pub major_on_seventh: S,
    pub queen_open_file: S,
    pub queen_semiopen_file: S,
    pub pawn_shield: [S; 3],
    pub pawn_storm: [S; 3],
    pub passers_friendly_king: [S; 7],
    pub passers_enemy_king: [S; 7],
    pub pawn_attacks: [S; 6],
    pub knight_attacks: [S; 6],
    pub bishop_attacks: [S; 6],
    pub rook_attacks: [S; 6],
    pub queen_attacks: [S; 6],
    pub knight_outposts: S,
    pub bishop_outposts: S,
    pub knight_shelter: S,
    pub bishop_shelter: S,
    pub tempo: S,
    pub safe_checks: [S; 6],
    pub unsafe_checks: [S; 6],
    pub bad_bishops: [S; 9],
    pub square_rule: S,
    pub free_passer: [[S; 8]; 8],
    pub protected_passer: [S; 8],
    pub bishop_long_diagonal: S,
    pub push_threats: [S; 6],
}

impl EvalWeights {
    pub const LEN: usize = std::mem::size_of::<Self>() / std::mem::size_of::<i32>();
}

impl Default for EvalWeights {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

////////////////////////////////////////////////////////////////////////////////
//
// EvalTrace
//
// Used for tracing the activations of a given board position by some
// `impl Tracer<EvalTrace>`
//
//
// TODO: Would be sick if we could derive this from EvalWeights
//
// So, something like
// ```rust
// #[derive(Tracer)]
// struct EvalWeights {}
// ```
//
// would generate an `EvalWeightsTrace` struct and an implementation of 
// `Tracer<EvalWeightsTrace>` for it.
//
////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Copy, Clone, PartialEq, Eq, Pod, Zeroable)]
#[repr(C)]
pub struct EvalTrace {
    pub eg_scaling: i32,
    pub piece_values: [i32; 6],
    pub pawn_psqt: [i32; 64],
    pub knight_psqt: [i32; 64],
    pub bishop_psqt: [i32; 64],
    pub rook_psqt: [i32; 64],
    pub queen_psqt: [i32; 64],
    pub king_psqt: [i32; 64],
    pub passed_pawn: [i32; 64],
    pub knight_mobility: [i32; 9],
    pub bishop_mobility: [i32; 14],
    pub rook_mobility: [i32; 15],
    pub queen_mobility: [i32; 28],
    pub virtual_mobility: [i32; 28],
    pub king_zone: [i32; 16],
    pub isolated_pawn: [i32; 8],
    pub doubled_pawn: [i32; 8],
    pub protected_pawn: [i32; 8],
    pub phalanx_pawn: [i32; 8],
    pub bishop_pair: i32,
    pub rook_open_file: i32,
    pub rook_semiopen_file: i32,
    pub connected_rooks: i32,
    pub major_on_seventh: i32,
    pub queen_open_file: i32,
    pub queen_semiopen_file: i32,
    pub pawn_shield: [i32; 3],
    pub pawn_storm: [i32; 3],
    pub passers_friendly_king: [i32; 7],
    pub passers_enemy_king: [i32; 7],
    pub pawn_attacks: [i32; 6],
    pub knight_attacks: [i32; 6],
    pub bishop_attacks: [i32; 6],
    pub rook_attacks: [i32; 6],
    pub queen_attacks: [i32; 6],
    pub knight_outposts: i32,
    pub bishop_outposts: i32,
    pub knight_shelter: i32,
    pub bishop_shelter: i32,
    pub tempo: i32,
    pub safe_checks: [i32; 6],
    pub unsafe_checks: [i32; 6],
    pub bad_bishops: [i32; 9],
    pub square_rule: i32,
    pub free_passer: [[i32; 8]; 8],
    pub protected_passer: [i32; 8],
    pub bishop_long_diagonal: i32,
    pub push_threats: [i32; 6],
}

impl EvalTrace {
    pub fn new(board: &Board) -> Self {
        let mut trace = EvalTrace::default();
        let mut eval = Eval::new(board, &mut trace);
        eval.total(board, &mut trace);
        trace
    }
}

impl Default for EvalTrace {
    fn default() -> Self {
        Self::zeroed()
    }
}

////////////////////////////////////////////////////////////////////////////////
//
// Tracer<T> trait implementations
//
// The way this trait will be used is by having a "tracer" that can be used to
// keep track of any activations. But, it also allows us to be generic over
// the actual thing doing the tracing, so we can provide a `NullTracer`
// that does nothing and gets optimized out.
//
////////////////////////////////////////////////////////////////////////////////

pub trait Tracer<T: Sized> {
    fn add(&mut self, f: impl Fn(&mut T));
}

/// The default implementation that we will use for tuning
impl Tracer<EvalTrace> for EvalTrace {
    fn add(&mut self, f: impl Fn(&mut EvalTrace)) {
        f(self);
    }
}

/// A null tracer that will get optimized out everywhere
pub struct NullTracer;

impl Tracer<EvalTrace> for NullTracer {
    fn add(&mut self, f: impl Fn(&mut EvalTrace)) {}
}

////////////////////////////////////////////////////////////////////////////////
//
// Conversion helper traits
//
// Implement some helper traits that help us convert from simbelmyne's score
// types to what the tuner expects.
//
////////////////////////////////////////////////////////////////////////////////

impl From<Score> for S {
    fn from(score: Score) -> Self {
        Self::new(score.mg as EvalScore, score.eg as EvalScore)
    }
}

impl Into<Score> for S {
    fn into(self) -> Score {
        Score { mg: self.mg() as f32, eg: self.eg() as f32 }
    }
}

impl fmt::Debug for S {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "s!({},{})", self.mg(), self.eg())
    }
}

impl Into<[Score; EvalWeights::LEN]> for EvalWeights {
    fn into(self) -> [Score; EvalWeights::LEN ] {
        let weights_array = bytemuck::cast::<EvalWeights, [S; Self::LEN]>(self);
        let mut tuner_weights = [Score::default(); Self::LEN];

        for (i, weight) in weights_array.into_iter().enumerate() {
            tuner_weights[i] = weight.into()
        }

        tuner_weights
    }
}

impl From<[Score; EvalWeights::LEN]> for EvalWeights {
    fn from(tuner_weights: [Score; EvalWeights::LEN]) -> Self {
        let mut weights = [S::default(); EvalWeights::LEN];

        for (i, weight) in tuner_weights.into_iter().enumerate() {
            weights[i] = weight.into()
        }

        bytemuck::cast::<[S; EvalWeights::LEN], EvalWeights>(weights)
    }
}
