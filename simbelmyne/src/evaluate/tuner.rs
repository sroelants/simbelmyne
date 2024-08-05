use bytemuck::Pod;
use bytemuck::Zeroable;
use chess::board::Board;
use chess::constants::RANKS;
use chess::movegen::lookups::BETWEEN;
use tuner::Component;
use tuner::Score; use tuner::Tune;
use std::fmt::Display;
use chess::bitboard::Bitboard;
use chess::piece::Color;
use chess::piece::PieceType;
use super::params::BISHOP_OUTPOSTS;
use super::params::KNIGHT_OUTPOSTS;
use super::params::MINOR_ATTACKS_ON_QUEENS;
use super::params::MINOR_ATTACKS_ON_ROOKS;
use super::params::PAWN_ATTACKS_ON_MINORS;
use super::params::PAWN_ATTACKS_ON_QUEENS;
use super::params::PAWN_ATTACKS_ON_ROOKS;
use super::params::PROTECTED_PAWN_BONUS;
use super::params::CONNECTED_ROOKS_BONUS;
use super::params::PASSERS_ENEMY_KING_PENALTY;
use super::params::PASSERS_FRIENDLY_KING_BONUS;
use super::params::PAWN_STORM_BONUS;
use super::params::KING_ZONE_ATTACKS;
use super::params::PHALANX_PAWN_BONUS;
use super::params::MAJOR_ON_SEVENTH_BONUS;
use super::params::QUEEN_OPEN_FILE_BONUS;
use super::params::QUEEN_SEMIOPEN_FILE_BONUS;
use super::params::ROOK_ATTACKS_ON_QUEENS;
use super::params::ROOK_SEMIOPEN_FILE_BONUS;
use super::params::TEMPO_BONUS;
use super::Score as EvalScore;
use super::params::PAWN_SHIELD_BONUS;
use super::params::VIRTUAL_MOBILITY_PENALTY;
use crate::evaluate::S;
use super::params::BISHOP_MOBILITY_BONUS;
use super::params::BISHOP_PAIR_BONUS;
use super::params::BISHOP_PSQT;
use super::params::DOUBLED_PAWN_PENALTY;
use super::params::ISOLATED_PAWN_PENALTY;
use super::params::KING_PSQT;
use super::params::KNIGHT_MOBILITY_BONUS;
use super::params::KNIGHT_PSQT;
use super::params::PASSED_PAWN_TABLE;
use super::params::PAWN_PSQT;
use super::params::PIECE_VALUES;
use super::params::QUEEN_MOBILITY_BONUS;
use super::params::QUEEN_PSQT;
use super::params::ROOK_MOBILITY_BONUS;
use super::params::ROOK_OPEN_FILE_BONUS;
use super::params::ROOK_PSQT;
use super::lookups::DOUBLED_PAWN_MASKS;
use super::lookups::FILES;
use super::lookups::ISOLATED_PAWN_MASKS;
use super::lookups::PASSED_PAWN_MASKS;

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
        use PieceType::*;
        use std::iter::{once, empty};

        empty()
            .chain(Self::material_components(board))
            .chain(Self::psqt_components(board, Pawn))
            .chain(Self::psqt_components(board, Knight))
            .chain(Self::psqt_components(board, Bishop))
            .chain(Self::psqt_components(board, Rook))
            .chain(Self::psqt_components(board, Queen))
            .chain(Self::psqt_components(board, King))
            .chain(Self::passed_pawn_components(board))
            .chain(Self::knight_mobility_components(board))
            .chain(Self::bishop_mobility_components(board))
            .chain(Self::rook_mobility_components(board))
            .chain(Self::queen_mobility_components(board))
            .chain(Self::virtual_mobility_components(board))
            .chain(Self::king_zone_components(board))
            .chain(once(Self::isolated_pawn_component(board)))
            .chain(once(Self::doubled_pawn_component(board)))
            .chain(once(Self::protected_pawn_component(board)))
            .chain(once(Self::phalanx_pawn_component(board)))
            .chain(once(Self::bishop_pair_component(board)))
            .chain(once(Self::rook_open_file_component(board)))
            .chain(once(Self::rook_semiopen_file_component(board)))
            .chain(once(Self::connected_rooks_component(board)))
            .chain(once(Self::major_on_seventh_component(board)))
            .chain(once(Self::queen_open_file_component(board)))
            .chain(once(Self::queen_semiopen_file_component(board)))
            .chain(Self::pawn_shield_component(board))
            .chain(Self::pawn_storm_component(board))
            .chain(Self::passers_friendly_king_components(board))
            .chain(Self::passers_enemy_king_components(board))
            .chain(once(Self::pawn_attacks_on_minors_component(board)))
            .chain(once(Self::pawn_attacks_on_rooks_component(board)))
            .chain(once(Self::pawn_attacks_on_queens_component(board)))
            .chain(once(Self::minor_attacks_on_rooks_component(board)))
            .chain(once(Self::minor_attacks_on_queens_component(board)))
            .chain(once(Self::rook_attacks_on_queens_component(board)))
            .chain(once(Self::knight_outposts_component(board)))
            .chain(once(Self::bishop_outposts_component(board)))
            .chain(once(Self::tempo_component(board)))
            .enumerate()
            .filter(|&(_, value)| value != 0.0)
            .map(|(idx, value)| Component::new(idx, value))
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
}

impl EvalTrace {
}

impl Default for EvalTrace {
    fn default() -> Self {
        Self::zeroed()
    }
}

impl EvalWeights {
    fn material_components(board: &Board) -> [f32; 6] {
        use Color::*;
        [
            board.pawns(White).count()   as f32 - board.pawns(Black).count()   as f32,
            board.knights(White).count() as f32 - board.knights(Black).count() as f32,
            board.bishops(White).count() as f32 - board.bishops(Black).count() as f32,
            board.rooks(White).count()   as f32 - board.rooks(Black).count()   as f32,
            board.queens(White).count()  as f32 - board.queens(Black).count()  as f32,
            board.kings(White).count()   as f32 - board.kings(Black).count()   as f32,
        ]
    }

    fn psqt_components(board: &Board, piece: PieceType) -> [f32; 64] {
        use Color::*;
        let mut components = [0.0; 64];
        let all_pieces = board.piece_bbs[piece as usize];
        let white_pieces = all_pieces & board.occupied_by(White);
        let black_pieces = all_pieces & board.occupied_by(Black);

        for square in white_pieces {
            components[square.flip() as usize] += 1.0;
        }

        for square in black_pieces {
            components[square as usize] -= 1.0;
        }

        components
    }

    fn passed_pawn_components(board: &Board) -> [f32; 64] {
        use Color::*;
        let mut components = [0.0; 64];
        let white_pawns = board.pawns(White);
        let black_pawns = board.pawns(Black);

        for sq in white_pawns {
            let mask = PASSED_PAWN_MASKS[White as usize][sq as usize];

            if black_pawns & mask == Bitboard::EMPTY {
                // Passed pawn tables are from black's perspective, so as to be
                // more readable. Hence the `.flip()`
                components[sq.flip() as usize] += 1.0;
            }
        }

        for sq in black_pawns {
            let mask = PASSED_PAWN_MASKS[Black as usize][sq as usize];

            if white_pawns & mask == Bitboard::EMPTY {
                // Passed pawn tables are from black's perspective, so as to be
                // more readable. Hence no `.flip()`
                components[sq as usize] -= 1.0;
            }
        }

        components
    }

    fn knight_mobility_components(board: &Board) -> [f32; 9] {
        use Color::*;
        let mut components = [0.0; 9];

        let white_pawn_attacks: Bitboard = board.pawns(White)
            .map(|sq| sq.pawn_attacks(White))
            .collect();

        let black_pawn_attacks: Bitboard = board.pawns(Black)
            .map(|sq| sq.pawn_attacks(Black))
            .collect();

        let white_blocked_pawns = board.pawns(White) & (board.pawns(Black) >> 8);
        let black_blocked_pawns = board.pawns(Black) & (board.pawns(White) << 8);

        for sq in board.knights(White) {
            let mut available_squares = sq.knight_squares() 
                & !white_blocked_pawns
                & !black_pawn_attacks;

            if board.get_pinrays(White).contains(sq) {
                available_squares &= board.get_pinrays(White);
            }

            let sq_count = available_squares.count();
            components[sq_count as usize] += 1.0;
        }

        for sq in board.knights(Black) {
            let mut available_squares = sq.knight_squares() 
                & !black_blocked_pawns
                & !white_pawn_attacks;

            if board.get_pinrays(Black).contains(sq) {
                available_squares &= board.get_pinrays(Black);
            }
            let sq_count = available_squares.count();
            components[sq_count as usize] -= 1.0;
        }

        components
    }

    fn bishop_mobility_components(board: &Board) -> [f32; 14] {
        use Color::*;
        let blockers = board.all_occupied();
        let mut components = [0.0; 14];

        let white_pawn_attacks: Bitboard = board.pawns(White)
            .map(|sq| sq.pawn_attacks(White))
            .collect();

        let black_pawn_attacks: Bitboard = board.pawns(Black)
            .map(|sq| sq.pawn_attacks(Black))
            .collect();

        let white_blocked_pawns = board.pawns(White) & (board.pawns(Black) >> 8);
        let black_blocked_pawns = board.pawns(Black) & (board.pawns(White) << 8);

        for sq in board.bishops(White) {
            let mut available_squares = sq.bishop_squares(blockers) 
                & !white_blocked_pawns
                & !black_pawn_attacks;

            if board.get_pinrays(White).contains(sq) {
                available_squares &= board.get_pinrays(White);
            }

            let sq_count = available_squares.count();
            components[sq_count as usize] += 1.0;
        }

        for sq in board.bishops(Black) {
            let mut available_squares = sq.bishop_squares(blockers) 
                & !black_blocked_pawns
                & !white_pawn_attacks;

            if board.get_pinrays(Black).contains(sq) {
                available_squares &= board.get_pinrays(Black);
            }

            let sq_count = available_squares.count();
            components[sq_count as usize] -= 1.0;
        }

        components
    }

    fn rook_mobility_components(board: &Board) -> [f32; 15] {
        use Color::*;
        let blockers = board.all_occupied();
        let mut components = [0.0; 15];

        let white_pawn_attacks: Bitboard = board.pawns(White)
            .map(|sq| sq.pawn_attacks(White))
            .collect();

        let black_pawn_attacks: Bitboard = board.pawns(Black)
            .map(|sq| sq.pawn_attacks(Black))
            .collect();

        let white_blocked_pawns = board.pawns(White) & (board.pawns(Black) >> 8);
        let black_blocked_pawns = board.pawns(Black) & (board.pawns(White) << 8);

        for sq in board.rooks(White) {
            let mut available_squares = sq.rook_squares(blockers) 
                & !white_blocked_pawns
                & !black_pawn_attacks;

            if board.get_pinrays(White).contains(sq) {
                available_squares &= board.get_pinrays(White);
            }

            let sq_count = available_squares.count();
            components[sq_count as usize] += 1.0;
        }

        for sq in board.rooks(Black) {
            let mut available_squares = sq.rook_squares(blockers) 
                & !black_blocked_pawns
                & !white_pawn_attacks;

            if board.get_pinrays(Black).contains(sq) {
                available_squares &= board.get_pinrays(Black);
            }

            let sq_count = available_squares.count();
            components[sq_count as usize] -= 1.0;
        }

        components
    }

    fn queen_mobility_components(board: &Board) -> [f32; 28] {
        use Color::*;
        let blockers = board.all_occupied();
        let mut components = [0.0; 28];

        let white_blocked_pawns = board.pawns(White) & (board.pawns(Black) >> 8);
        let black_blocked_pawns = board.pawns(Black) & (board.pawns(White) << 8);

        let white_pawn_attacks: Bitboard = board.pawns(White)
            .map(|sq| sq.pawn_attacks(White))
            .collect();

        let black_pawn_attacks: Bitboard = board.pawns(Black)
            .map(|sq| sq.pawn_attacks(Black))
            .collect();


        for sq in board.queens(White) {
            let mut available_squares = sq.queen_squares(blockers) 
                & !white_blocked_pawns
                & !black_pawn_attacks;

            if board.get_pinrays(White).contains(sq) {
                available_squares &= board.get_pinrays(White);
            }


            let sq_count = available_squares.count();
            components[sq_count as usize] += 1.0;
        }

        for sq in board.queens(Black) {
            let mut available_squares = sq.queen_squares(blockers) 
                & !black_blocked_pawns
                & !white_pawn_attacks;

            if board.get_pinrays(Black).contains(sq) {
                available_squares &= board.get_pinrays(Black);
            }

            let sq_count = available_squares.count();
            components[sq_count as usize] -= 1.0;
        }

        components
    }

    fn virtual_mobility_components(board: &Board) -> [f32; 28] {
        use Color::*;
        let blockers = board.all_occupied();
        let white_pieces = board.occupied_by(White);
        let black_pieces = board.occupied_by(Black);
        let mut components = [0.0; 28];

        for king_sq in board.kings(White) {
            let available_squares = king_sq.queen_squares(blockers) 
                & !white_pieces;

            let sq_count = available_squares.count();
            components[sq_count as usize] += 1.0;
        }

        for king_sq in board.kings(Black) {
            let available_squares = king_sq.queen_squares(blockers) 
                & !black_pieces;

            let sq_count = available_squares.count();
            components[sq_count as usize] -= 1.0;
        }

        components

    }

    fn king_zone_components(board: &Board) -> [f32; 16] {
        use Color::*;
        let mut components = [0.0; 16];
        let blockers = board.all_occupied();

        for us in [White, Black] {
            let mut attacks = 0;

            let king_sq = board.kings(us).first();
            let king_zone = king_sq.king_squares();

            for knight in board.knights(!us) {
                attacks += (king_zone & knight.knight_squares()).count();
            }

            for bishop in board.bishops(!us) {
                attacks += (king_zone & bishop.bishop_squares(blockers)).count();
            }

            for rook in board.rooks(!us) {
                attacks += (king_zone & rook.rook_squares(blockers)).count();
            }

            for queen in board.queens(!us) {
                attacks += (king_zone & queen.queen_squares(blockers)).count();
            }

            let attacks = usize::min(attacks as usize, 15);

            components[attacks] += if us.is_white() { 1.0 } else { -1.0 };
        }

        components
    }

    fn isolated_pawn_component(board: &Board) -> f32 {
        use Color::*;
        let white_pawns = board.pawns(White);
        let black_pawns = board.pawns(Black);
        let mut component = 0.0;

        for sq in white_pawns {
            let mask = ISOLATED_PAWN_MASKS[sq as usize];

            if white_pawns & mask == Bitboard::EMPTY {
                component += 1.0;
            }
        }

        for sq in black_pawns {
            let mask = ISOLATED_PAWN_MASKS[sq as usize];

            if black_pawns & mask == Bitboard::EMPTY {
                component -= 1.0;
            }
        }

        component
    }

    fn doubled_pawn_component(board: &Board) -> f32 {
        use Color::*;
        let white_pawns = board.pawns(White);
        let black_pawns = board.pawns(Black);

        let mut component = 0.0;

        for mask in DOUBLED_PAWN_MASKS {
            let doubled_white = (white_pawns & mask).count().saturating_sub(1) as f32;
            let doubled_black = (black_pawns & mask).count().saturating_sub(1) as f32;
            component += doubled_white - doubled_black;
        }

        component
    }

    fn protected_pawn_component(board: &Board) -> f32 {
        use Color::*;
        let mut component = 0.0;

        for us in [White, Black] {
            let our_pawns = board.pawns(us);

            let pawn_attacks = if us.is_white() {
                our_pawns.forward_left::<true>() | our_pawns.forward_right::<true>()
            } else {
                our_pawns.forward_left::<false>() | our_pawns.forward_right::<false>()
            };

            let protected_pawns = our_pawns & pawn_attacks;

            if us.is_white() {
                component += protected_pawns.count() as f32;
            } else {
                component -= protected_pawns.count() as f32;
            }
        }

        component
    }

    fn phalanx_pawn_component(board: &Board) -> f32 {
        use Color::*;
        let mut component = 0.0;

        for us in [White, Black] {
            let our_pawns = board.pawns(us);
            let phalanx_pawns = our_pawns & (our_pawns.left() | our_pawns.right());
            if us.is_white() { 
                component += phalanx_pawns.count() as f32;
            } else {
                component -= phalanx_pawns.count() as f32;
            }
        }

        component
    }

    fn bishop_pair_component(board: &Board) -> f32 {
        use Color::*;
        let mut component = 0.0;

        if board.bishops(White).count() == 2 {
            component += 1.0;
        }

        if board.bishops(Black).count() == 2 {
            component -= 1.0;
        }

        component
    }

    fn rook_open_file_component(board: &Board) -> f32 {
        use Color::*;
        use PieceType::*;
        let pawns = board.piece_bbs[Pawn as usize];
        let mut component = 0.0;

        for sq in board.rooks(White) {
            if (FILES[sq as usize] & pawns).is_empty() {
                component += 1.0;
            }
        }

        for sq in board.rooks(Black) {
            if (FILES[sq as usize] & pawns).is_empty() {
                component -= 1.0;
            }
        }

        component
    }

    fn rook_semiopen_file_component(board: &Board) -> f32 {
        use Color::*;
        let white_pawns = board.pawns(White);
        let black_pawns = board.pawns(Black);
        let mut component = 0.0;

        for sq in board.rooks(White) {
            if (FILES[sq as usize] & white_pawns).is_empty() {
                component += 1.0;
            }
        }

        for sq in board.rooks(Black) {
            if (FILES[sq as usize] & black_pawns).is_empty() {
                component -= 1.0;
            }
        }

        component
    }

    fn connected_rooks_component(board: &Board) -> f32 {
        use Color::*;
        let mut component = 0.0;

        for us in [White, Black] {
            let mut rooks = board.rooks(us);
            let back_rank = if us.is_white() { 0 } else { 7 };

            if let Some(first) = rooks.next() {
                if let Some(second) = rooks.next() {
                    let on_back_rank = first.rank() == back_rank 
                        && second.rank() == back_rank;

                    let between = BETWEEN[first as usize][second as usize] 
                        & board.all_occupied();

                    let connected = between.is_empty();

                    if on_back_rank && connected {
                        component += if us.is_white() { 1.0 } else { -1.0 };
                    }
                }
            }
        }

        component
    }

    fn major_on_seventh_component(board: &Board) -> f32 {
        use Color::*;
        let mut component = 0.0;

        for us in [White, Black] {
            let seventh_rank = if us.is_white() { RANKS[6] } else { RANKS[1] };
            let eighth_rank = if us.is_white() { RANKS[7] } else { RANKS[0] };

            let pawns_on_seventh = !(board.pawns(!us) & seventh_rank).is_empty();
            let king_on_eighth = !(board.kings(!us) & eighth_rank).is_empty();
            let majors = board.rooks(us) | board.queens(us);

            if pawns_on_seventh || king_on_eighth {
                let bonus = (majors & seventh_rank).count() as f32;
                component += if us.is_white() { bonus } else { -bonus };
            }
        }

        component
    }

    fn queen_open_file_component(board: &Board) -> f32 {
        use Color::*;
        use PieceType::*;
        let pawns = board.piece_bbs[Pawn as usize];
        let mut component = 0.0;

        for us in [White, Black] {
            for sq in board.queens(us) {
                if (FILES[sq as usize] & pawns).is_empty() {
                    component += if us.is_white() { 1.0 } else { -1.0 };
                }
            }
        }

        component
    }

    fn queen_semiopen_file_component(board: &Board) -> f32 {
        use Color::*;
        let mut component = 0.0;

        for us in [White, Black] {
            let our_pawns = board.pawns(us);
            let their_pawns = board.pawns(!us);

            for sq in board.queens(us) {
                if (FILES[sq as usize] & our_pawns).is_empty() 
                    && !(FILES[sq as usize] & their_pawns).is_empty() {
                    component += if us.is_white() { 1.0 } else { -1.0 };
                }
            }
        }

        component
    }

    fn pawn_shield_component(board: &Board) -> [f32; 3] {
        use Color::*;
        let mut components = [0.0; 3];

        let white_king = board.kings(White).first();
        let black_king = board.kings(Black).first();

        let white_pawns = board.pawns(White);
        let black_pawns = board.pawns(Black);

        let white_shield_mask = PASSED_PAWN_MASKS[White as usize][white_king as usize];
        let black_shield_mask = PASSED_PAWN_MASKS[Black as usize][black_king as usize];

        let white_shield_pawns = white_pawns & white_shield_mask;
        let black_shield_pawns = black_pawns & black_shield_mask;

        for pawn in white_shield_pawns {
            let distance = pawn.vdistance(white_king).min(3) - 1;
            components[distance] += 1.0;
        }

        for pawn in black_shield_pawns {
            let distance = pawn.vdistance(black_king).min(3) - 1;
            components[distance] -= 1.0;
        }

        components
    }

    fn pawn_storm_component(board: &Board) -> [f32; 3] {
        use Color::*;
        let mut components = [0.0; 3];

        let white_king = board.kings(White).first();
        let black_king = board.kings(Black).first();

        let white_pawns = board.pawns(White);
        let black_pawns = board.pawns(Black);

        let white_storm_mask = PASSED_PAWN_MASKS[Black as usize][black_king as usize];
        let black_storm_mask = PASSED_PAWN_MASKS[White as usize][white_king as usize];

        let white_storm_pawns = white_pawns & white_storm_mask;
        let black_storm_pawns = black_pawns & black_storm_mask;

        for pawn in white_storm_pawns {
            let distance = pawn.vdistance(black_king).min(3) - 1; // 0, 1 or 2
            components[distance] += 1.0;
        }

        for pawn in black_storm_pawns {
            let distance = pawn.vdistance(white_king).min(3) - 1; // 0, 1 or 2
            components[distance] -= 1.0;
        }

        components
    }

    fn passers_friendly_king_components(board: &Board) -> [f32; 7] {
        use Color::*;
        let mut components = [0.0; 7];

        let white_king = board.kings(White).first();
        let black_king = board.kings(Black).first();

        let white_pawns = board.pawns(White);
        let black_pawns = board.pawns(Black);

        for pawn in white_pawns {
            if (PASSED_PAWN_MASKS[White as usize][pawn as usize] & black_pawns).is_empty() {
                let distance = pawn.max_dist(white_king);
                components[distance - 1] += 1.0;
            }
        }

        for pawn in black_pawns {
            if (PASSED_PAWN_MASKS[Black as usize][pawn as usize] & white_pawns).is_empty() {
                let distance = pawn.max_dist(black_king);
                components[distance - 1] -= 1.0;
            }
        }

        components
    }

    fn passers_enemy_king_components(board: &Board) -> [f32; 7] {
        use Color::*;
        let mut components = [0.0; 7];

        let white_king = board.kings(White).first();
        let black_king = board.kings(Black).first();

        let white_pawns = board.pawns(White);
        let black_pawns = board.pawns(Black);

        for pawn in white_pawns {
            if (PASSED_PAWN_MASKS[White as usize][pawn as usize] & black_pawns).is_empty() {
                let distance = pawn.max_dist(black_king);
                components[distance - 1] += 1.0;
            }
        }

        for pawn in black_pawns {
            if (PASSED_PAWN_MASKS[Black as usize][pawn as usize] & white_pawns).is_empty() {
                let distance = pawn.max_dist(white_king);
                components[distance - 1] -= 1.0;
            }
        }

        components
    }

    fn pawn_attacks_on_minors_component(board: &Board) -> f32 {
        use Color::*;
        let mut total = 0.0;

        for us in [White, Black] {
            let their_minors = board.knights(!us) | board.bishops(!us);
            let pawn_attacks = board.pawns(us)
                .map(|sq| sq.pawn_attacks(us))
                .collect::<Bitboard>();

            if us.is_white() {
                total += (pawn_attacks & their_minors).count() as f32;
            } else {
                total -= (pawn_attacks & their_minors).count() as f32;
            }
        }

        total
    }

    fn pawn_attacks_on_rooks_component(board: &Board) -> f32 {
        use Color::*;
        let mut total = 0.0;

        for us in [White, Black] {
            let their_rooks = board.rooks(!us);

            let pawn_attacks = board.pawns(us)
                .map(|sq| sq.pawn_attacks(us))
                .collect::<Bitboard>();

            if us.is_white() {
                total += (pawn_attacks & their_rooks).count() as f32;
            } else {
                total -= (pawn_attacks & their_rooks).count() as f32;
            }
        }

        total
    }

    fn pawn_attacks_on_queens_component(board: &Board) -> f32 {
        use Color::*;
        let mut total = 0.0;

        for us in [White, Black] {
            let their_queens = board.queens(!us);

            let pawn_attacks = board.pawns(us)
                .map(|sq| sq.pawn_attacks(us))
                .collect::<Bitboard>();

            if us.is_white() {
                total += (pawn_attacks & their_queens).count() as f32;
            } else {
                total -= (pawn_attacks & their_queens).count() as f32;
            }
        }

        total
    }

    fn minor_attacks_on_rooks_component(board: &Board) -> f32 {
        use Color::*;
        let mut total = 0.0;
        let blockers = board.all_occupied();

        for us in [White, Black] {
            let their_rooks = board.rooks(!us);

            for sq in board.knights(us) {
                if us.is_white() {
                    total += (sq.knight_squares() & their_rooks).count() as f32;
                } else {
                    total -= (sq.knight_squares() & their_rooks).count() as f32;
                }
            }

            for sq in board.bishops(us) {
                if us.is_white() {
                    total += (sq.bishop_squares(blockers) & their_rooks).count() as f32;
                } else {
                    total -= (sq.bishop_squares(blockers) & their_rooks).count() as f32;
                }
            }
        }

        total
    }

    fn minor_attacks_on_queens_component(board: &Board) -> f32 {
        use Color::*;
        let mut total = 0.0;
        let blockers = board.all_occupied();

        for us in [White, Black] {
            let their_queens = board.queens(!us);

            for sq in board.knights(us) {
                if us.is_white() {
                    total += (sq.knight_squares() & their_queens).count() as f32;
                } else {
                    total -= (sq.knight_squares() & their_queens).count() as f32;
                }
            }

            for sq in board.bishops(us) {
                if us.is_white() {
                    total += (sq.bishop_squares(blockers) & their_queens).count() as f32;
                } else {
                    total -= (sq.bishop_squares(blockers) & their_queens).count() as f32;
                }
            }
        }

        total
    }

    fn rook_attacks_on_queens_component(board: &Board) -> f32 {
        use Color::*;
        let mut total = 0.0;
        let blockers = board.all_occupied();

        for us in [White, Black] {
            let their_queens = board.queens(!us);

            for sq in board.rooks(us) {
                if us.is_white() {
                    total += (sq.rook_squares(blockers) & their_queens).count() as f32;
                } else {
                    total -= (sq.rook_squares(blockers) & their_queens).count() as f32;
                }
            }
        }

        total
    }

    fn knight_outposts_component(board: &Board) -> f32 {
        use Color::*;
        let mut total = 0.0;

        for us in [White, Black] {
            let outposts = if us.is_white() {
                outposts::<true>(board)
            } else {
                outposts::<false>(board)
            };

            if us.is_white() { 
                total += (board.knights(us) & outposts).count() as f32
            } else {
                total -= (board.knights(us) & outposts).count() as f32
            };
        }

        total
    }

    fn bishop_outposts_component(board: &Board) -> f32 {
        use Color::*;
        let mut total = 0.0;

        for us in [White, Black] {
            let outposts = if us.is_white() {
                outposts::<true>(board)
            } else {
                outposts::<false>(board)
            };

            if us.is_white() { 
                total += (board.bishops(us) & outposts).count() as f32
            } else {
                total -= (board.bishops(us) & outposts).count() as f32
            };
        }

        total
    }

    fn tempo_component(board: &Board) -> f32 {
        if board.current.is_white() { 1.0 } else { -1.0 }
    }
}

fn outposts<const WHITE: bool>(board: &Board) -> Bitboard {
    use Color::*;
    let us = if WHITE { White } else { Black };

    let our_pawn_attacks = board.pawns(us)
        .map(|sq| sq.pawn_attacks(us))
        .collect::<Bitboard>();

    let their_pawn_attacks = board.pawns(!us)
        .map(|sq| sq.pawn_attacks(!us))
        .collect::<Bitboard>();

    let their_attack_spans = their_pawn_attacks 
            | their_pawn_attacks.backward_by::<WHITE>(1)
            | their_pawn_attacks.backward_by::<WHITE>(2)
            | their_pawn_attacks.backward_by::<WHITE>(3);

    our_pawn_attacks & !their_attack_spans
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

////////////////////////////////////////////////////////////////////////////////
//
// Tests
//
////////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{position::Position, tests::TEST_POSITIONS};
    use tuner::evaluate_components;

    #[test]
    #[ignore = "Broken test, try and make less flaky in the future"]
    fn default_evalweights_evaluation_returns_same_value() {
        for fen in TEST_POSITIONS {
            let board: Board = fen.parse().unwrap();
            let weights = EvalWeights::default().weights();
            let components = EvalWeights::components(&board);
            let weight_eval = evaluate_components(&weights, &components, board.phase());

            let position = Position::new(board);
            let classical_eval = position.score.total(&board);

            println!("{fen}\nClassical eval: {classical_eval}, EvalWeights eval: {weight_eval}\n\n");
            // Allow for slight discrepancies because of rounding differences
            assert!(f32::abs(weight_eval - classical_eval as f32) <= 5.0)
        }
    }
}
