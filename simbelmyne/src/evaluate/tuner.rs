use chess::board::Board;
use tuner::Component;
use tuner::Score;
use tuner::Tune;
use std::fmt::Display;
use chess::bitboard::Bitboard;
use chess::piece::Color;
use chess::piece::PieceType;
use super::Score as EvalScore;
use super::params::PAWN_SHIELD_BONUS;
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

const NUM_WEIGHTS: usize = 525;

#[derive(Debug, Copy, Clone)]
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
    isolated_pawn: S,
    doubled_pawn: S,
    bishop_pair: S,
    rook_open_file: S,
    pawn_shield: S,
}

impl Tune<NUM_WEIGHTS> for EvalWeights {
    fn weights(&self) -> [Score; NUM_WEIGHTS] {
        use std::iter::{once, empty};
        let mut weights = [Score::default(); NUM_WEIGHTS];

        let weights_iter = empty()
            .chain(self.piece_values)
            .chain(self.pawn_psqt)
            .chain(self.knight_psqt)
            .chain(self.bishop_psqt)
            .chain(self.rook_psqt)
            .chain(self.queen_psqt)
            .chain(self.king_psqt)
            .chain(self.passed_pawn)
            .chain(self.knight_mobility)
            .chain(self.bishop_mobility)
            .chain(self.rook_mobility)
            .chain(self.queen_mobility)
            .chain(once(self.isolated_pawn))
            .chain(once(self.doubled_pawn))
            .chain(once(self.bishop_pair))
            .chain(once(self.rook_open_file))
            .chain(once(self.pawn_shield));

        for (i, weight) in weights_iter.enumerate() {
            weights[i] = weight.into()
        }

        weights
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
            .chain(once(Self::isolated_pawn_component(board)))
            .chain(once(Self::doubled_pawn_component(board)))
            .chain(once(Self::bishop_pair_component(board)))
            .chain(once(Self::rook_open_file_component(board)))
            .chain(once(Self::pawn_shield_component(board)))
            .enumerate()
            .filter(|&(_, value)| value != 0.0)
            .map(|(idx, value)| Component::new(idx, value))
            .collect::<Vec<_>>()
    }
}

impl Display for EvalWeights {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut weights = self.weights().into_iter().map(S::from);

        let piece_values       = weights.by_ref().take(6).collect::<Vec<_>>();
        let pawn_psqt          = weights.by_ref().take(64).collect::<Vec<_>>();
        let knight_psqt        = weights.by_ref().take(64).collect::<Vec<_>>();
        let bishop_psqt        = weights.by_ref().take(64).collect::<Vec<_>>();
        let rook_psqt          = weights.by_ref().take(64).collect::<Vec<_>>();
        let queen_psqt         = weights.by_ref().take(64).collect::<Vec<_>>();
        let king_psqt          = weights.by_ref().take(64).collect::<Vec<_>>();
        let passed_pawn        = weights.by_ref().take(64).collect::<Vec<_>>();
        let knight_mobility    = weights.by_ref().take(9).collect::<Vec<_>>();
        let bishop_mobility    = weights.by_ref().take(14).collect::<Vec<_>>();
        let rook_mobility      = weights.by_ref().take(15).collect::<Vec<_>>();
        let queen_mobility     = weights.by_ref().take(28).collect::<Vec<_>>();
        let isolated_pawn      = weights.by_ref().next().unwrap();
        let doubled_pawn       = weights.by_ref().next().unwrap();
        let bishop_pair        = weights.by_ref().next().unwrap();
        let rook_open_file     = weights.by_ref().next().unwrap();
        let pawn_shield        = weights.by_ref().next().unwrap();

        writeln!(f, "use crate::evaluate::S;\n")?;

        writeln!(f, "pub const PIECE_VALUES: [S; 6] = {};\n",             print_vec(&piece_values))?;
        writeln!(f, "pub const PAWN_PSQT: [S; 64] = {};\n",               print_table(&pawn_psqt))?;
        writeln!(f, "pub const KNIGHT_PSQT: [S; 64] = {};\n",             print_table(&knight_psqt))?;
        writeln!(f, "pub const BISHOP_PSQT: [S; 64] = {};\n",             print_table(&bishop_psqt))?;
        writeln!(f, "pub const ROOK_PSQT: [S; 64] = {};\n",               print_table(&rook_psqt))?;
        writeln!(f, "pub const QUEEN_PSQT: [S; 64] = {};\n",              print_table(&queen_psqt))?;
        writeln!(f, "pub const KING_PSQT: [S; 64] = {};\n",               print_table(&king_psqt))?;
        writeln!(f, "pub const PASSED_PAWN_TABLE: [S; 64] = {};\n",       print_table(&passed_pawn))?;
        writeln!(f, "pub const KNIGHT_MOBILITY_BONUS: [S; 9] = {};\n",    print_vec(&knight_mobility))?;
        writeln!(f, "pub const BISHOP_MOBILITY_BONUS: [S; 14] = {};\n",   print_vec(&bishop_mobility))?;
        writeln!(f, "pub const ROOK_MOBILITY_BONUS: [S; 15] = {};\n",     print_vec(&rook_mobility))?;
        writeln!(f, "pub const QUEEN_MOBILITY_BONUS: [S; 28] = {};\n",    print_vec(&queen_mobility))?;
        writeln!(f, "pub const ISOLATED_PAWN_PENALTY: S = {};\n",         isolated_pawn)?;
        writeln!(f, "pub const DOUBLED_PAWN_PENALTY: S = {};\n",          doubled_pawn)?;
        writeln!(f, "pub const BISHOP_PAIR_BONUS: S = {};\n",             bishop_pair)?;
        writeln!(f, "pub const ROOK_OPEN_FILE_BONUS: S = {};\n",          rook_open_file)?;
        writeln!(f, "pub const PAWN_SHIELD_BONUS: S = {};\n",             pawn_shield)?;

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
            piece_values:    PIECE_VALUES,
            pawn_psqt:       PAWN_PSQT,
            knight_psqt:     KNIGHT_PSQT,
            bishop_psqt:     BISHOP_PSQT,
            rook_psqt:       ROOK_PSQT,
            queen_psqt:      QUEEN_PSQT,
            king_psqt:       KING_PSQT,
            passed_pawn:     PASSED_PAWN_TABLE, 
            knight_mobility: KNIGHT_MOBILITY_BONUS,
            bishop_mobility: BISHOP_MOBILITY_BONUS,
            rook_mobility:   ROOK_MOBILITY_BONUS,
            queen_mobility:  QUEEN_MOBILITY_BONUS,
            isolated_pawn:   ISOLATED_PAWN_PENALTY,
            doubled_pawn:    DOUBLED_PAWN_PENALTY,
            bishop_pair:     BISHOP_PAIR_BONUS,
            rook_open_file:  ROOK_OPEN_FILE_BONUS,
            pawn_shield:     PAWN_SHIELD_BONUS,
        }
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
        let white_pieces = board.occupied_by(White);
        let black_pieces = board.occupied_by(Black);
        let mut components = [0.0; 9];

        for sq in board.knights(White) {
            let available_squares = sq.knight_squares() 
                & !white_pieces;

            let sq_count = available_squares.count();
            components[sq_count as usize] += 1.0;
        }

        for sq in board.knights(Black) {
            let available_squares = sq.knight_squares() 
                & !black_pieces;

            let sq_count = available_squares.count();
            components[sq_count as usize] -= 1.0;
        }

        components
    }

    fn bishop_mobility_components(board: &Board) -> [f32; 14] {
        use Color::*;
        let blockers = board.all_occupied();
        let white_pieces = board.occupied_by(White);
        let black_pieces = board.occupied_by(Black);
        let mut components = [0.0; 14];

        for sq in board.bishops(White) {
            let available_squares = sq.bishop_squares(blockers) 
                & !white_pieces;

            let sq_count = available_squares.count();
            components[sq_count as usize] += 1.0;
        }

        for sq in board.bishops(Black) {
            let available_squares = sq.bishop_squares(blockers) 
                & !black_pieces;

            let sq_count = available_squares.count();
            components[sq_count as usize] -= 1.0;
        }

        components
    }

    fn rook_mobility_components(board: &Board) -> [f32; 15] {
        use Color::*;
        let blockers = board.all_occupied();
        let white_pieces = board.occupied_by(White);
        let black_pieces = board.occupied_by(Black);
        let mut components = [0.0; 15];

        for sq in board.rooks(White) {
            let available_squares = sq.rook_squares(blockers) 
                & !white_pieces;

            let sq_count = available_squares.count();
            components[sq_count as usize] += 1.0;
        }

        for sq in board.rooks(Black) {
            let available_squares = sq.rook_squares(blockers) 
                & !black_pieces;

            let sq_count = available_squares.count();
            components[sq_count as usize] -= 1.0;
        }

        components
    }

    fn queen_mobility_components(board: &Board) -> [f32; 28] {
        use Color::*;
        let blockers = board.all_occupied();
        let white_pieces = board.occupied_by(White);
        let black_pieces = board.occupied_by(Black);
        let mut components = [0.0; 28];

        for sq in board.queens(White) {
            let available_squares = sq.queen_squares(blockers) 
                & !white_pieces;

            let sq_count = available_squares.count();
            components[sq_count as usize] += 1.0;
        }

        for sq in board.queens(Black) {
            let available_squares = sq.queen_squares(blockers) 
                & !black_pieces;

            let sq_count = available_squares.count();
            components[sq_count as usize] -= 1.0;
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

    fn pawn_shield_component(board: &Board) -> f32 {
        use Color::*;

        let white_king_sq = board.kings(White).first();
        let white_shield_squares = white_king_sq.forward(White).into_iter()
            .chain(white_king_sq.forward(White).and_then(|sq| sq.left()))
            .chain(white_king_sq.forward(White).and_then(|sq| sq.right()))
            .collect::<Bitboard>();


        let black_king_sq = board.kings(Black).first();
        let black_shield_squares = black_king_sq.forward(Black).into_iter()
            .chain(black_king_sq.forward(Black).and_then(|sq| sq.left()))
            .chain(black_king_sq.forward(Black).and_then(|sq| sq.right()))
            .collect::<Bitboard>();

        let white_pawn_shield = board.pawns(White) & white_shield_squares;
        let black_pawn_shield = board.pawns(Black) & black_shield_squares;

        white_pawn_shield.count() as f32 - black_pawn_shield.count() as f32
    }
}

impl From<Score> for S {
    fn from(score: Score) -> Self {
        Self(score.mg as EvalScore, score.eg as EvalScore)
    }
}

impl Into<Score> for S {
    fn into(self) -> Score {
        Score { mg: self.0 as f32, eg: self.1 as f32 }
    }
}

impl Display for S {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "S({},{})", self.0, self.1)
    }
}

impl<const N: usize> From<[Score; N]> for EvalWeights {
    fn from(weights: [Score; N]) -> Self {
        let mut weights = weights.into_iter().map(|score| S::from(score));

        Self {
            piece_values    : weights.by_ref().take(6).collect::<Vec<_>>().try_into().unwrap(),
            pawn_psqt       : weights.by_ref().take(64).collect::<Vec<_>>().try_into().unwrap(),
            knight_psqt     : weights.by_ref().take(64).collect::<Vec<_>>().try_into().unwrap(),
            bishop_psqt     : weights.by_ref().take(64).collect::<Vec<_>>().try_into().unwrap(),
            rook_psqt       : weights.by_ref().take(64).collect::<Vec<_>>().try_into().unwrap(),
            queen_psqt      : weights.by_ref().take(64).collect::<Vec<_>>().try_into().unwrap(),
            king_psqt       : weights.by_ref().take(64).collect::<Vec<_>>().try_into().unwrap(),
            passed_pawn     : weights.by_ref().take(64).collect::<Vec<_>>().try_into().unwrap(),
            knight_mobility : weights.by_ref().take(9).collect::<Vec<_>>().try_into().unwrap(),
            bishop_mobility : weights.by_ref().take(14).collect::<Vec<_>>().try_into().unwrap(),
            rook_mobility   : weights.by_ref().take(15).collect::<Vec<_>>().try_into().unwrap(),
            queen_mobility  : weights.by_ref().take(28).collect::<Vec<_>>().try_into().unwrap(),
            isolated_pawn   : weights.by_ref().next().unwrap(),
            doubled_pawn    : weights.by_ref().next().unwrap(),
            bishop_pair     : weights.by_ref().next().unwrap(),
            rook_open_file  : weights.by_ref().next().unwrap(),
            pawn_shield     : weights.by_ref().next().unwrap(),
        }
        
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
    use Color::*;

    #[test]
    fn default_evalweights_evaluation_returns_same_value() {
        for fen in TEST_POSITIONS {
            let board: Board = fen.parse().unwrap();
            let weights = EvalWeights::default().weights();
            let components = EvalWeights::components(&board);
            let weight_eval = EvalWeights::evaluate_components(&weights, &components, board.phase());

            let position = Position::new(board);
            let classical_eval = position.score.total(White);

            println!("{fen}\nClassical eval: {classical_eval}, EvalWeights eval: {weight_eval}\n\n");
            // Allow for slight discrepancies because of rounding differences
            assert!(f32::abs(weight_eval - classical_eval as f32) <= 5.0)
        }
    }
}
