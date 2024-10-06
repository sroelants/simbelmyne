use bytemuck::Pod;
use bytemuck::Zeroable;
use chess::board::Board;
use tuner::ActivationParams;
use tuner::Component;
use tuner::Score; use tuner::Tune;
use std::fmt::Display;
use super::params::*;
use super::Eval;
use super::Score as EvalScore;
use crate::evaluate::S;

////////////////////////////////////////////////////////////////////////////////
//
// Tune implementation for EvalWeights struct
//
////////////////////////////////////////////////////////////////////////////////

#[derive(Pod, Zeroable)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(C)]
pub struct EvalWeights {
    piece_values: [S; 6],
    pawn_psqt: [S; 64],
    knight_psqt: [S; 64],
    bishop_psqt: [S; 64],
    rook_psqt: [S; 64],
    queen_psqt: [S; 64],
    king_psqt: [S; 64],
    passed_pawn: [S; 64],
    knight_mobility: [S; 9],
    bishop_mobility: [S; 14],
    rook_mobility: [S; 15],
    queen_mobility: [S; 28],
    virtual_mobility: [S; 28],
    king_zone: [S; 16],
    isolated_pawn: S,
    doubled_pawn: S,
    protected_pawn: S,
    phalanx_pawn: S,
    bishop_pair: S,
    rook_open_file: S,
    rook_semiopen_file: S,
    connected_rooks: S,
    major_on_seventh: S,
    queen_open_file: S,
    queen_semiopen_file: S,
    pawn_shield: [S; 3],
    pawn_storm: [S; 3],
    passers_friendly_king: [S; 7],
    passers_enemy_king: [S; 7],
    pawn_attacks: [S; 6],
    knight_attacks: [S; 6],
    bishop_attacks: [S; 6],
    rook_attacks: [S; 6],
    queen_attacks: [S; 6],
    king_attacks: [S; 6],
    knight_outposts: S,
    bishop_outposts: S,
    knight_shelter: S,
    bishop_shelter: S,
    tempo: S,
    safe_checks: [S; 6],
    unsafe_checks: [S; 6],
    bad_bishops: [S; 9],
    square_rule: S,
    free_passer: [S; 8],
    protected_passer: [S; 8],
    bishop_long_diagonal: S,
}

impl EvalWeights {
    pub const LEN: usize = std::mem::size_of::<Self>() / std::mem::size_of::<i32>();
}

impl Tune<{Self::LEN}> for EvalWeights {
    fn weights(&self) -> [Score; Self::LEN] {
        let weights_array = bytemuck::cast::<EvalWeights, [S; Self::LEN]>(*self);
        let mut tuner_weights = [Score::default(); Self::LEN];

        for (i, weight) in weights_array.into_iter().enumerate() {
            tuner_weights[i] = weight.into()
        }

        tuner_weights
    }

    fn activations(board: &Board) -> ActivationParams {
        use bytemuck::cast;
        let trace = EvalTrace::new(board);
        let trace = cast::<EvalTrace, [i32; EvalWeights::LEN+1]>(trace);

        let eg_scaling = trace[0];
        let activations = &trace[1..];

        let components = activations
            .into_iter()
            .enumerate()
            .filter(|&(_, &value)| value != 0)
            .map(|(idx, &value)| Component::new(idx, value as f32))
            .collect::<Vec<_>>();

        ActivationParams { eg_scaling: 128, components }
    }
}

impl Display for EvalWeights {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "use crate::evaluate::S;")?;
        writeln!(f, "use crate::s;")?;
        writeln!(f)?;
        writeln!(f, "pub const PIECE_VALUES: [S; 6] = {};\n",                print_vec(&self.piece_values))?;
        writeln!(f, "pub const PAWN_PSQT: [S; 64] = {};\n",                  print_table(&self.pawn_psqt))?;
        writeln!(f, "pub const KNIGHT_PSQT: [S; 64] = {};\n",                print_table(&self.knight_psqt))?;
        writeln!(f, "pub const BISHOP_PSQT: [S; 64] = {};\n",                print_table(&self.bishop_psqt))?;
        writeln!(f, "pub const ROOK_PSQT: [S; 64] = {};\n",                  print_table(&self.rook_psqt))?;
        writeln!(f, "pub const QUEEN_PSQT: [S; 64] = {};\n",                 print_table(&self.queen_psqt))?;
        writeln!(f, "pub const KING_PSQT: [S; 64] = {};\n",                  print_table(&self.king_psqt))?;
        writeln!(f, "pub const PASSED_PAWN_TABLE: [S; 64] = {};\n",          print_table(&self.passed_pawn))?;
        writeln!(f, "pub const KNIGHT_MOBILITY_BONUS: [S; 9] = {};\n",       print_vec(&self.knight_mobility))?;
        writeln!(f, "pub const BISHOP_MOBILITY_BONUS: [S; 14] = {};\n",      print_vec(&self.bishop_mobility))?;
        writeln!(f, "pub const ROOK_MOBILITY_BONUS: [S; 15] = {};\n",        print_vec(&self.rook_mobility))?;
        writeln!(f, "pub const QUEEN_MOBILITY_BONUS: [S; 28] = {};\n",       print_vec(&self.queen_mobility))?;
        writeln!(f, "pub const VIRTUAL_MOBILITY_PENALTY: [S; 28] = {};\n",   print_vec(&self.virtual_mobility))?;
        writeln!(f, "pub const KING_ZONE_ATTACKS: [S; 16] = {};\n",          print_vec(&self.king_zone))?;
        writeln!(f, "pub const ISOLATED_PAWN_PENALTY: S = {};\n",            self.isolated_pawn)?;
        writeln!(f, "pub const DOUBLED_PAWN_PENALTY: S = {};\n",             self.doubled_pawn)?;
        writeln!(f, "pub const PROTECTED_PAWN_BONUS: S = {};\n",             self.protected_pawn)?;
        writeln!(f, "pub const PHALANX_PAWN_BONUS: S = {};\n",               self.phalanx_pawn)?;
        writeln!(f, "pub const BISHOP_PAIR_BONUS: S = {};\n",                self.bishop_pair)?;
        writeln!(f, "pub const ROOK_OPEN_FILE_BONUS: S = {};\n",             self.rook_open_file)?;
        writeln!(f, "pub const ROOK_SEMIOPEN_FILE_BONUS: S = {};\n",         self.rook_semiopen_file)?;
        writeln!(f, "pub const CONNECTED_ROOKS_BONUS: S = {};\n",            self.connected_rooks)?;
        writeln!(f, "pub const MAJOR_ON_SEVENTH_BONUS: S = {};\n",           self.major_on_seventh)?;
        writeln!(f, "pub const QUEEN_OPEN_FILE_BONUS: S = {};\n",            self.queen_open_file)?;
        writeln!(f, "pub const QUEEN_SEMIOPEN_FILE_BONUS: S = {};\n",        self.queen_semiopen_file)?;
        writeln!(f, "pub const PAWN_SHIELD_BONUS: [S; 3] = {};\n",           print_vec(&self.pawn_shield))?;
        writeln!(f, "pub const PAWN_STORM_BONUS: [S; 3] = {};\n",            print_vec(&self.pawn_storm))?;
        writeln!(f, "pub const PASSERS_FRIENDLY_KING_BONUS: [S; 7] = {};\n", print_vec(&self.passers_friendly_king))?;
        writeln!(f, "pub const PASSERS_ENEMY_KING_PENALTY: [S; 7] = {};\n",  print_vec(&self.passers_enemy_king))?;
        writeln!(f, "pub const PAWN_ATTACKS: [S; 6] = {};\n",                print_vec(&self.pawn_attacks))?;
        writeln!(f, "pub const KNIGHT_ATTACKS: [S; 6] = {};\n",              print_vec(&self.knight_attacks))?;
        writeln!(f, "pub const BISHOP_ATTACKS: [S; 6] = {};\n",              print_vec(&self.bishop_attacks))?;
        writeln!(f, "pub const ROOK_ATTACKS: [S; 6] = {};\n",                print_vec(&self.rook_attacks))?;
        writeln!(f, "pub const QUEEN_ATTACKS: [S; 6] = {};\n",               print_vec(&self.queen_attacks))?;
        writeln!(f, "pub const KING_ATTACKS: [S; 6] = {};\n",                print_vec(&self.king_attacks))?;
        writeln!(f, "pub const KNIGHT_OUTPOSTS: S = {};\n",                  self.knight_outposts)?;
        writeln!(f, "pub const BISHOP_OUTPOSTS: S = {};\n",                  self.bishop_outposts)?;
        writeln!(f, "pub const KNIGHT_SHELTER: S = {};\n",                   self.knight_shelter)?;
        writeln!(f, "pub const BISHOP_SHELTER: S = {};\n",                   self.bishop_shelter)?;
        writeln!(f, "pub const TEMPO_BONUS: S = {};\n",                      self.tempo)?;
        writeln!(f, "pub const SAFE_CHECKS: [S; 6] = {};\n",                 print_vec(&self.safe_checks))?;
        writeln!(f, "pub const UNSAFE_CHECKS: [S; 6] = {};\n",               print_vec(&self.unsafe_checks))?;
        writeln!(f, "pub const BAD_BISHOPS: [S; 9] = {};\n",                 print_vec(&self.bad_bishops))?;
        writeln!(f, "pub const SQUARE_RULE: S = {};\n",                      self.square_rule)?;
        writeln!(f, "pub const FREE_PASSER: [S; 8] = {};\n",                 print_vec(&self.free_passer))?;
        writeln!(f, "pub const PROTECTED_PASSER: [S; 8] = {};\n",            print_vec(&self.protected_passer))?;
        writeln!(f, "pub const BISHOP_LONG_DIAGONAL: S = {};\n",             self.bishop_long_diagonal)?;

        Ok(())
    }
}

fn print_vec(weights: &[S]) -> String {
        let rows = weights.iter()
            .map(|weight| format!("{weight},\n"))
            .collect::<String>();

    format!("[\n{rows}]")
}

fn print_table(weights: &[S]) -> String {
    let rows = weights.chunks(8)
        .map(|row| 
            row.iter()
                .map(|weight| format!("{:12}", format!("{weight},")))
                .collect::<String>()
        )
        .collect::<Vec<_>>()
        .join("\n");

    format!("[\n{rows} ]")
}

impl Default for EvalWeights {
    fn default() -> Self {
        Self {
            piece_values:          PIECE_VALUES,
            pawn_psqt:             PAWN_PSQT,
            knight_psqt:           KNIGHT_PSQT,
            bishop_psqt:           BISHOP_PSQT,
            rook_psqt:             ROOK_PSQT,
            queen_psqt:            QUEEN_PSQT,
            king_psqt:             KING_PSQT,
            passed_pawn:           PASSED_PAWN_TABLE, 
            knight_mobility:       KNIGHT_MOBILITY_BONUS,
            bishop_mobility:       BISHOP_MOBILITY_BONUS,
            rook_mobility:         ROOK_MOBILITY_BONUS,
            queen_mobility:        QUEEN_MOBILITY_BONUS,
            virtual_mobility:      VIRTUAL_MOBILITY_PENALTY,
            king_zone:             KING_ZONE_ATTACKS,
            isolated_pawn:         ISOLATED_PAWN_PENALTY,
            doubled_pawn:          DOUBLED_PAWN_PENALTY,
            protected_pawn:        PROTECTED_PAWN_BONUS,
            phalanx_pawn:          PHALANX_PAWN_BONUS,
            bishop_pair:           BISHOP_PAIR_BONUS,
            rook_open_file:        ROOK_OPEN_FILE_BONUS,
            rook_semiopen_file:    ROOK_SEMIOPEN_FILE_BONUS,
            connected_rooks:       CONNECTED_ROOKS_BONUS,
            major_on_seventh:      MAJOR_ON_SEVENTH_BONUS,
            queen_open_file:       QUEEN_OPEN_FILE_BONUS,
            queen_semiopen_file:   QUEEN_SEMIOPEN_FILE_BONUS,
            pawn_shield:           PAWN_SHIELD_BONUS,
            pawn_storm:            PAWN_STORM_BONUS,
            passers_friendly_king: PASSERS_FRIENDLY_KING_BONUS,
            passers_enemy_king:    PASSERS_ENEMY_KING_PENALTY,
            pawn_attacks:          PAWN_ATTACKS,
            knight_attacks:        KNIGHT_ATTACKS,
            bishop_attacks:        BISHOP_ATTACKS,
            rook_attacks:          ROOK_ATTACKS,
            queen_attacks:         QUEEN_ATTACKS,
            king_attacks:          KING_ATTACKS,
            knight_outposts:       KNIGHT_OUTPOSTS,
            bishop_outposts:       BISHOP_OUTPOSTS,
            knight_shelter:        KNIGHT_SHELTER,
            bishop_shelter:        BISHOP_SHELTER,
            tempo:                 TEMPO_BONUS,
            safe_checks:           SAFE_CHECKS,
            unsafe_checks:         UNSAFE_CHECKS,
            bad_bishops:           BAD_BISHOPS,
            square_rule:           SQUARE_RULE,
            free_passer:           FREE_PASSER,
            protected_passer:      PROTECTED_PASSER,
            bishop_long_diagonal:  BISHOP_LONG_DIAGONAL,
        }
    }
}

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
    pub isolated_pawn: i32,
    pub doubled_pawn: i32,
    pub protected_pawn: i32,
    pub phalanx_pawn: i32,
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
    pub king_attacks: [i32; 6],
    pub knight_outposts: i32,
    pub bishop_outposts: i32,
    pub knight_shelter: i32,
    pub bishop_shelter: i32,
    pub tempo: i32,
    pub safe_checks: [i32; 6],
    pub unsafe_checks: [i32; 6],
    pub bad_bishops: [i32; 9],
    pub square_rule: i32,
    pub free_passer: [i32; 8],
    pub protected_passer: [i32; 8],
    pub bishop_long_diagonal: i32,
}

pub trait Trace: Sized {
    fn add(&mut self, f: impl Fn(&mut EvalTrace) -> ());
}

impl Trace for EvalTrace {
    fn add(&mut self, f: impl Fn(&mut EvalTrace) -> ()) {
        f(self);
    }
}

pub struct NullTrace;

impl Trace for NullTrace {
    fn add(&mut self, f: impl Fn(&mut EvalTrace) -> ()) {}
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

impl Display for S {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "s!({},{})", self.mg(), self.eg())
    }
}

impl<const N: usize> From<[Score; N]> for EvalWeights {
    fn from(tuner_weights: [Score; N]) -> Self {
        let mut weights = [S::default(); N];

        for (i, weight) in tuner_weights.into_iter().enumerate() {
            weights[i] = weight.into()
        }

        bytemuck::cast::<[S; N], EvalWeights>(weights)
    }
}
