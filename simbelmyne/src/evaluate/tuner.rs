use bytemuck::Pod;
use bytemuck::Zeroable;
use chess::board::Board;
use chess::square::Square;
use tuner::Component;
use tuner::Score; use tuner::Tune;
use std::fmt::Display;
use super::bishop_outposts;
use super::bishop_pair;
use super::connected_rooks;
use super::king_zone;
use super::knight_outposts;
use super::major_on_seventh;
use super::material;
use super::mobility;
use super::passers_enemy_king;
use super::passers_friendly_king;
use super::pawn_shield;
use super::pawn_storm;
use super::pawn_structure::PawnStructure;
use super::psqt;
use super::queen_open_file;
use super::queen_semiopen_file;
use super::rook_open_file;
use super::rook_semiopen_file;
use super::safe_checks;
use super::threats;
use super::virtual_mobility;
use super::params::*;
use super::EvalContext;
use super::Score as EvalScore;
use crate::evaluate::S;

const WHITE: bool = true;
const BLACK: bool = false;

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
    pawn_attacks_on_minors: S,
    pawn_attacks_on_rooks: S,
    pawn_attacks_on_queens: S,
    minor_attacks_on_rooks: S,
    minor_attacks_on_queens: S,
    rook_attacks_on_queens: S,
    knight_outposts: S,
    bishop_outposts: S,
    tempo: S,
    safe_checks: [S; 6],
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

    fn components(board: &Board) -> Vec<Component> {
        let trace = EvalTrace::new(board);

        bytemuck::cast::<EvalTrace, [i32; EvalWeights::LEN]>(trace)
            .into_iter()
            .enumerate()
            .filter(|&(_, value)| value != 0)
            .map(|(idx, value)| Component::new(idx, value as f32))
            .collect::<Vec<_>>()
    }
}

impl Display for EvalWeights {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut weights = self.weights().into_iter().map(S::from);

        let piece_values          = weights.by_ref().take(6).collect::<Vec<_>>();
        let pawn_psqt             = weights.by_ref().take(64).collect::<Vec<_>>();
        let knight_psqt           = weights.by_ref().take(64).collect::<Vec<_>>();
        let bishop_psqt           = weights.by_ref().take(64).collect::<Vec<_>>();
        let rook_psqt             = weights.by_ref().take(64).collect::<Vec<_>>();
        let queen_psqt            = weights.by_ref().take(64).collect::<Vec<_>>();
        let king_psqt             = weights.by_ref().take(64).collect::<Vec<_>>();
        let passed_pawn           = weights.by_ref().take(64).collect::<Vec<_>>();
        let knight_mobility       = weights.by_ref().take(9).collect::<Vec<_>>();
        let bishop_mobility       = weights.by_ref().take(14).collect::<Vec<_>>();
        let rook_mobility         = weights.by_ref().take(15).collect::<Vec<_>>();
        let queen_mobility        = weights.by_ref().take(28).collect::<Vec<_>>();
        let virtual_mobility      = weights.by_ref().take(28).collect::<Vec<_>>();
        let king_zone             = weights.by_ref().take(16).collect::<Vec<_>>();
        let isolated_pawn         = weights.by_ref().next().unwrap();
        let doubled_pawn          = weights.by_ref().next().unwrap();
        let protected_pawn          = weights.by_ref().next().unwrap();
        let phalanx_pawn          = weights.by_ref().next().unwrap();
        let bishop_pair           = weights.by_ref().next().unwrap();
        let rook_open_file        = weights.by_ref().next().unwrap();
        let rook_semiopen_file    = weights.by_ref().next().unwrap();
        let connected_rooks       = weights.by_ref().next().unwrap();
        let major_on_seventh      = weights.by_ref().next().unwrap();
        let queen_open_file       = weights.by_ref().next().unwrap();
        let queen_semiopen_file   = weights.by_ref().next().unwrap();
        let pawn_shield           = weights.by_ref().take(3).collect::<Vec<_>>();
        let pawn_storm            = weights.by_ref().take(3).collect::<Vec<_>>();
        let passers_friendly_king = weights.by_ref().take(7).collect::<Vec<_>>();
        let passers_enemy_king    = weights.by_ref().take(7).collect::<Vec<_>>();
        let pawn_attacks_on_minors= weights.by_ref().next().unwrap();
        let pawn_attacks_on_rooks = weights.by_ref().next().unwrap();
        let pawn_attacks_on_queens= weights.by_ref().next().unwrap();
        let minor_attacks_on_rooks= weights.by_ref().next().unwrap();
        let minor_attacks_on_queens= weights.by_ref().next().unwrap();
        let rook_attacks_on_queens= weights.by_ref().next().unwrap();
        let knight_outposts       = weights.by_ref().next().unwrap();
        let bishop_outposts       = weights.by_ref().next().unwrap();
        let tempo                 = weights.by_ref().next().unwrap();
        let safe_checks           = weights.by_ref().take(6).collect::<Vec<_>>();

        writeln!(f, "use crate::evaluate::S;\n")?;

        writeln!(f, "pub const PIECE_VALUES: [S; 6] = {};\n",                print_vec(&piece_values))?;
        writeln!(f, "pub const PAWN_PSQT: [S; 64] = {};\n",                  print_table(&pawn_psqt))?;
        writeln!(f, "pub const KNIGHT_PSQT: [S; 64] = {};\n",                print_table(&knight_psqt))?;
        writeln!(f, "pub const BISHOP_PSQT: [S; 64] = {};\n",                print_table(&bishop_psqt))?;
        writeln!(f, "pub const ROOK_PSQT: [S; 64] = {};\n",                  print_table(&rook_psqt))?;
        writeln!(f, "pub const QUEEN_PSQT: [S; 64] = {};\n",                 print_table(&queen_psqt))?;
        writeln!(f, "pub const KING_PSQT: [S; 64] = {};\n",                  print_table(&king_psqt))?;
        writeln!(f, "pub const PASSED_PAWN_TABLE: [S; 64] = {};\n",          print_table(&passed_pawn))?;
        writeln!(f, "pub const KNIGHT_MOBILITY_BONUS: [S; 9] = {};\n",       print_vec(&knight_mobility))?;
        writeln!(f, "pub const BISHOP_MOBILITY_BONUS: [S; 14] = {};\n",      print_vec(&bishop_mobility))?;
        writeln!(f, "pub const ROOK_MOBILITY_BONUS: [S; 15] = {};\n",        print_vec(&rook_mobility))?;
        writeln!(f, "pub const QUEEN_MOBILITY_BONUS: [S; 28] = {};\n",       print_vec(&queen_mobility))?;
        writeln!(f, "pub const VIRTUAL_MOBILITY_PENALTY: [S; 28] = {};\n",   print_vec(&virtual_mobility))?;
        writeln!(f, "pub const KING_ZONE_ATTACKS: [S; 16] = {};\n",          print_vec(&king_zone))?;
        writeln!(f, "pub const ISOLATED_PAWN_PENALTY: S = {};\n",            isolated_pawn)?;
        writeln!(f, "pub const DOUBLED_PAWN_PENALTY: S = {};\n",             doubled_pawn)?;
        writeln!(f, "pub const PROTECTED_PAWN_BONUS: S = {};\n",           protected_pawn)?;
        writeln!(f, "pub const PHALANX_PAWN_BONUS: S = {};\n",               phalanx_pawn)?;
        writeln!(f, "pub const BISHOP_PAIR_BONUS: S = {};\n",                bishop_pair)?;
        writeln!(f, "pub const ROOK_OPEN_FILE_BONUS: S = {};\n",             rook_open_file)?;
        writeln!(f, "pub const ROOK_SEMIOPEN_FILE_BONUS: S = {};\n",         rook_semiopen_file)?;
        writeln!(f, "pub const CONNECTED_ROOKS_BONUS: S = {};\n",            connected_rooks)?;
        writeln!(f, "pub const MAJOR_ON_SEVENTH_BONUS: S = {};\n",           major_on_seventh)?;
        writeln!(f, "pub const QUEEN_OPEN_FILE_BONUS: S = {};\n",            queen_open_file)?;
        writeln!(f, "pub const QUEEN_SEMIOPEN_FILE_BONUS: S = {};\n",        queen_semiopen_file)?;
        writeln!(f, "pub const PAWN_SHIELD_BONUS: [S; 3] = {};\n",           print_vec(&pawn_shield))?;
        writeln!(f, "pub const PAWN_STORM_BONUS: [S; 3] = {};\n",            print_vec(&pawn_storm))?;
        writeln!(f, "pub const PASSERS_FRIENDLY_KING_BONUS: [S; 7] = {};\n", print_vec(&passers_friendly_king))?;
        writeln!(f, "pub const PASSERS_ENEMY_KING_PENALTY: [S; 7] = {};\n",  print_vec(&passers_enemy_king))?;
        writeln!(f, "pub const PAWN_ATTACKS_ON_MINORS: S = {};\n",           pawn_attacks_on_minors)?;
        writeln!(f, "pub const PAWN_ATTACKS_ON_ROOKS: S = {};\n",            pawn_attacks_on_rooks)?;
        writeln!(f, "pub const PAWN_ATTACKS_ON_QUEENS: S = {};\n",           pawn_attacks_on_queens)?;
        writeln!(f, "pub const MINOR_ATTACKS_ON_ROOKS: S = {};\n",           minor_attacks_on_rooks)?;
        writeln!(f, "pub const MINOR_ATTACKS_ON_QUEENS: S = {};\n",          minor_attacks_on_queens)?;
        writeln!(f, "pub const ROOK_ATTACKS_ON_QUEENS: S = {};\n",           rook_attacks_on_queens)?;
        writeln!(f, "pub const KNIGHT_OUTPOSTS: S = {};\n",                  knight_outposts)?;
        writeln!(f, "pub const BISHOP_OUTPOSTS: S = {};\n",                  bishop_outposts)?;
        writeln!(f, "pub const TEMPO_BONUS: S = {};\n",                      tempo)?;
        writeln!(f, "pub const SAFE_CHECKS: [S; 6] = {};\n",                  print_vec(&safe_checks))?;

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
            pawn_attacks_on_minors:PAWN_ATTACKS_ON_MINORS,
            pawn_attacks_on_rooks: PAWN_ATTACKS_ON_ROOKS,
            pawn_attacks_on_queens:PAWN_ATTACKS_ON_QUEENS,
            minor_attacks_on_rooks:MINOR_ATTACKS_ON_ROOKS,
            minor_attacks_on_queens:MINOR_ATTACKS_ON_QUEENS,
            rook_attacks_on_queens:ROOK_ATTACKS_ON_QUEENS,
            knight_outposts:       KNIGHT_OUTPOSTS,
            bishop_outposts:       BISHOP_OUTPOSTS,
            tempo:                 TEMPO_BONUS,
            safe_checks:           SAFE_CHECKS,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Pod, Zeroable)]
#[repr(C)]
pub struct EvalTrace {
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
    pub pawn_attacks_on_minors: i32,
    pub pawn_attacks_on_rooks: i32,
    pub pawn_attacks_on_queens: i32,
    pub minor_attacks_on_rooks: i32,
    pub minor_attacks_on_queens: i32,
    pub rook_attacks_on_queens: i32,
    pub knight_outposts: i32,
    pub bishop_outposts: i32,
    pub tempo: i32,
    pub safe_checks: [i32; 6],
}

impl EvalTrace {
    pub fn new(board: &Board) -> Self {
        let mut trace = Self::default();
        let pawn_structure = PawnStructure::new(board);
        let mut ctx = EvalContext::new(board);

        // Material + psqt
        for (sq, &piece) in board.piece_list.iter().enumerate() {
            if let Some(piece) = piece {
                material(piece, Some(&mut trace));
                psqt(piece, Square::from(sq), Some(&mut trace));
            }
        }

        // Treat tempo separately
        if board.current.is_white() {
            trace.tempo += 1;
        } else {
            trace.tempo -= 1;
        }

        pawn_structure.compute_score::<WHITE>(board, Some(&mut trace));
        pawn_structure.compute_score::<BLACK>(board, Some(&mut trace));
        bishop_pair::<WHITE>(board, Some(&mut trace));
        bishop_pair::<BLACK>(board, Some(&mut trace));
        rook_open_file::<WHITE>(board, &pawn_structure, Some(&mut trace));
        rook_open_file::<BLACK>(board, &pawn_structure, Some(&mut trace));
        rook_semiopen_file::<WHITE>(board, &pawn_structure, Some(&mut trace));
        rook_semiopen_file::<BLACK>(board, &pawn_structure, Some(&mut trace));
        queen_open_file::<WHITE>(board, &pawn_structure, Some(&mut trace));
        queen_open_file::<BLACK>(board, &pawn_structure, Some(&mut trace));
        queen_semiopen_file::<WHITE>(board, &pawn_structure, Some(&mut trace));
        queen_semiopen_file::<BLACK>(board, &pawn_structure, Some(&mut trace));
        major_on_seventh::<WHITE>(board, Some(&mut trace));
        major_on_seventh::<BLACK>(board, Some(&mut trace));
        pawn_shield::<WHITE>(board, Some(&mut trace));
        pawn_shield::<BLACK>(board, Some(&mut trace));
        pawn_storm::<WHITE>(board, Some(&mut trace));
        pawn_storm::<BLACK>(board, Some(&mut trace));
        passers_friendly_king::<WHITE>(board, &pawn_structure, Some(&mut trace));
        passers_friendly_king::<BLACK>(board, &pawn_structure, Some(&mut trace));
        passers_enemy_king::<WHITE>(board, &pawn_structure, Some(&mut trace));
        passers_enemy_king::<BLACK>(board, &pawn_structure, Some(&mut trace));
        knight_outposts::<WHITE>(board, &pawn_structure, Some(&mut trace));
        knight_outposts::<BLACK>(board, &pawn_structure, Some(&mut trace));
        bishop_outposts::<WHITE>(board, &pawn_structure, Some(&mut trace));
        bishop_outposts::<BLACK>(board, &pawn_structure, Some(&mut trace));
        connected_rooks::<WHITE>(board, Some(&mut trace));
        connected_rooks::<BLACK>(board, Some(&mut trace));
        mobility::<WHITE>(board, &pawn_structure, &mut ctx, Some(&mut trace));
        mobility::<BLACK>(board, &pawn_structure, &mut ctx, Some(&mut trace));
        virtual_mobility::<WHITE>(board, Some(&mut trace));
        virtual_mobility::<BLACK>(board, Some(&mut trace));
        king_zone::<WHITE>(&ctx, Some(&mut trace));
        king_zone::<BLACK>(&ctx, Some(&mut trace));
        threats::<WHITE>(&ctx, Some(&mut trace));
        threats::<BLACK>(&ctx, Some(&mut trace));
        safe_checks::<WHITE>(board, &ctx, Some(&mut trace));
        safe_checks::<BLACK>(board, &ctx, Some(&mut trace));

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
