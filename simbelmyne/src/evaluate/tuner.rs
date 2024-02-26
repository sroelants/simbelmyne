use chess::board::Board;
use crate::tuner::Component;
use crate::tuner::Score;
use crate::tuner::Tune;
use std::fmt::Display;
use chess::bitboard::Bitboard;
use chess::piece::Color;
use chess::piece::PieceType;
use super::EG_BISHOP_PAIR_BONUS;
use super::EG_DOUBLED_PAWN_PENALTY;
use super::EG_ISOLATED_PAWN_PENALTY;
use super::EG_ROOK_OPEN_FILE_BONUS;
use super::EG_ROOK_SEMI_OPEN_FILE_BONUS;
use super::EG_VALUES;
use super::MG_BISHOP_PAIR_BONUS;
use super::MG_DOUBLED_PAWN_PENALTY;
use super::MG_ISOLATED_PAWN_PENALTY;
use super::MG_ROOK_OPEN_FILE_BONUS;
use super::MG_ROOK_SEMI_OPEN_FILE_BONUS;
use super::MG_VALUES;
use super::lookups::DOUBLED_PAWN_MASKS;
use super::lookups::EG_PASSED_PAWN_TABLE;
use super::lookups::FILES;
use super::lookups::ISOLATED_PAWN_MASKS;
use super::lookups::MG_PASSED_PAWN_TABLE;
use super::lookups::PASSED_PAWN_MASKS;
use super::piece_square_tables::EG_BISHOP_TABLE;
use super::piece_square_tables::EG_KING_TABLE;
use super::piece_square_tables::EG_KNIGHT_TABLE;
use super::piece_square_tables::EG_PAWN_TABLE;
use super::piece_square_tables::EG_QUEEN_TABLE;
use super::piece_square_tables::EG_ROOK_TABLE;
use super::piece_square_tables::MG_BISHOP_TABLE;
use super::piece_square_tables::MG_KING_TABLE;
use super::piece_square_tables::MG_KNIGHT_TABLE;
use super::piece_square_tables::MG_PAWN_TABLE;
use super::piece_square_tables::MG_QUEEN_TABLE;
use super::piece_square_tables::MG_ROOK_TABLE;


////////////////////////////////////////////////////////////////////////////////
//
// Tune implementation for EvalWeights struct
//
////////////////////////////////////////////////////////////////////////////////

const NUM_WEIGHTS: usize = 459;

#[derive(Debug, Copy, Clone)]
pub struct EvalWeights {
    mg_piece_values: [i32; 6],
    eg_piece_values: [i32; 6],

    mg_pawn_psqt: [i32; 64],
    eg_pawn_psqt: [i32; 64],

    mg_knight_psqt: [i32; 64],
    eg_knight_psqt: [i32; 64],

    mg_bishop_psqt: [i32; 64],
    eg_bishop_psqt: [i32; 64],

    mg_rook_psqt: [i32; 64],
    eg_rook_psqt: [i32; 64],

    mg_queen_psqt: [i32; 64],
    eg_queen_psqt: [i32; 64],

    mg_king_psqt: [i32; 64],
    eg_king_psqt: [i32; 64],

    mg_passed_pawn: [i32; 64],
    eg_passed_pawn: [i32; 64],

    mg_isolated_pawn: i32,
    eg_isolated_pawn: i32,

    mg_doubled_pawn: i32,
    eg_doubled_pawn: i32,

    mg_bishop_pair: i32,
    eg_bishop_pair: i32,

    mg_rook_open_file: i32,
    eg_rook_open_file: i32,

    mg_rook_semiopen_file: i32,
    eg_rook_semiopen_file: i32,
}

impl Tune<NUM_WEIGHTS> for EvalWeights {
    fn weights(&self) -> [Score; NUM_WEIGHTS] {
        use std::iter::{once, empty};
        let mut weights = [Score::default(); NUM_WEIGHTS];

        let mg_weights = empty()
            .chain(self.mg_piece_values)
            .chain(self.mg_pawn_psqt)
            .chain(self.mg_knight_psqt)
            .chain(self.mg_bishop_psqt)
            .chain(self.mg_rook_psqt)
            .chain(self.mg_queen_psqt)
            .chain(self.mg_king_psqt)
            .chain(self.mg_passed_pawn)
            .chain(once(self.mg_isolated_pawn))
            .chain(once(self.mg_doubled_pawn))
            .chain(once(self.mg_bishop_pair))
            .chain(once(self.mg_rook_open_file))
            .chain(once(self.mg_rook_semiopen_file));

        let eg_weights = empty()
            .chain(self.eg_piece_values)
            .chain(self.eg_pawn_psqt)
            .chain(self.eg_knight_psqt)
            .chain(self.eg_bishop_psqt)
            .chain(self.eg_rook_psqt)
            .chain(self.eg_queen_psqt)
            .chain(self.eg_king_psqt)
            .chain(self.eg_passed_pawn)
            .chain(once(self.eg_isolated_pawn))
            .chain(once(self.eg_doubled_pawn))
            .chain(once(self.eg_bishop_pair))
            .chain(once(self.eg_rook_open_file))
            .chain(once(self.eg_rook_semiopen_file));

        for (i, (mg, eg)) in mg_weights.zip(eg_weights).enumerate() {
            weights[i] = Score { mg: mg as f32, eg: eg as f32 }
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
            .chain(once(Self::isolated_pawn_component(board)))
            .chain(once(Self::doubled_pawn_component(board)))
            .chain(once(Self::bishop_pair_component(board)))
            .chain(once(Self::rook_open_file_component(board)))
            .chain(once(Self::rook_semiopen_file_component(board)))
            .enumerate()
            .filter(|&(_, value)| value != 0.0)
            .map(|(idx, value)| Component::new(idx, value))
            .collect::<Vec<_>>()
    }

    fn load_weights(&mut self, weights: [Score; NUM_WEIGHTS]) {
        let mut mg_weights = weights.iter().map(|score| score.mg as i32);

        self.mg_piece_values = mg_weights.by_ref().take(6).collect::<Vec<_>>().try_into().unwrap();
        self.mg_pawn_psqt = mg_weights.by_ref().take(64).collect::<Vec<_>>().try_into().unwrap();
        self.mg_knight_psqt = mg_weights.by_ref().take(64).collect::<Vec<_>>().try_into().unwrap();
        self.mg_bishop_psqt = mg_weights.by_ref().take(64).collect::<Vec<_>>().try_into().unwrap();
        self.mg_rook_psqt = mg_weights.by_ref().take(64).collect::<Vec<_>>().try_into().unwrap();
        self.mg_queen_psqt = mg_weights.by_ref().take(64).collect::<Vec<_>>().try_into().unwrap();
        self.mg_king_psqt = mg_weights.by_ref().take(64).collect::<Vec<_>>().try_into().unwrap();
        self.mg_passed_pawn = mg_weights.by_ref().take(64).collect::<Vec<_>>().try_into().unwrap();
        self.mg_isolated_pawn = mg_weights.by_ref().next().unwrap();
        self.mg_doubled_pawn = mg_weights.by_ref().next().unwrap();
        self.mg_bishop_pair = mg_weights.by_ref().next().unwrap();
        self.mg_rook_open_file = mg_weights.by_ref().next().unwrap();
        self.mg_rook_semiopen_file = mg_weights.by_ref().next().unwrap();

        let mut eg_weights = weights.iter().map(|score| score.eg as i32);
        self.eg_piece_values = eg_weights.by_ref().take(6).collect::<Vec<_>>().try_into().unwrap();
        self.eg_pawn_psqt = eg_weights.by_ref().take(64).collect::<Vec<_>>().try_into().unwrap();
        self.eg_knight_psqt = eg_weights.by_ref().take(64).collect::<Vec<_>>().try_into().unwrap();
        self.eg_bishop_psqt = eg_weights.by_ref().take(64).collect::<Vec<_>>().try_into().unwrap();
        self.eg_rook_psqt = eg_weights.by_ref().take(64).collect::<Vec<_>>().try_into().unwrap();
        self.eg_queen_psqt = eg_weights.by_ref().take(64).collect::<Vec<_>>().try_into().unwrap();
        self.eg_king_psqt = eg_weights.by_ref().take(64).collect::<Vec<_>>().try_into().unwrap();
        self.eg_passed_pawn = eg_weights.by_ref().take(64).collect::<Vec<_>>().try_into().unwrap();
        self.eg_isolated_pawn = eg_weights.by_ref().next().unwrap();
        self.eg_doubled_pawn = eg_weights.by_ref().next().unwrap();
        self.eg_bishop_pair = eg_weights.by_ref().next().unwrap();
        self.eg_rook_open_file = eg_weights.by_ref().next().unwrap();
        self.eg_rook_semiopen_file = eg_weights.by_ref().next().unwrap();
    }
}

impl Display for EvalWeights {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Default for EvalWeights {
    fn default() -> Self {
        Self {
            mg_piece_values: MG_VALUES,
            eg_piece_values: EG_VALUES,

            mg_pawn_psqt: MG_PAWN_TABLE,
            eg_pawn_psqt: EG_PAWN_TABLE,

            mg_knight_psqt: MG_KNIGHT_TABLE,
            eg_knight_psqt: EG_KNIGHT_TABLE,

            mg_bishop_psqt: MG_BISHOP_TABLE,
            eg_bishop_psqt: EG_BISHOP_TABLE,

            mg_rook_psqt: MG_ROOK_TABLE,
            eg_rook_psqt: EG_ROOK_TABLE,

            mg_queen_psqt: MG_QUEEN_TABLE,
            eg_queen_psqt: EG_QUEEN_TABLE,

            mg_king_psqt: MG_KING_TABLE,
            eg_king_psqt: EG_KING_TABLE,

            mg_passed_pawn:MG_PASSED_PAWN_TABLE, 
            eg_passed_pawn: EG_PASSED_PAWN_TABLE,

            mg_isolated_pawn: MG_ISOLATED_PAWN_PENALTY,
            eg_isolated_pawn: EG_ISOLATED_PAWN_PENALTY,

            mg_doubled_pawn: MG_DOUBLED_PAWN_PENALTY,
            eg_doubled_pawn: EG_DOUBLED_PAWN_PENALTY,

            mg_bishop_pair: MG_BISHOP_PAIR_BONUS,
            eg_bishop_pair: EG_BISHOP_PAIR_BONUS,

            mg_rook_open_file: MG_ROOK_OPEN_FILE_BONUS,
            eg_rook_open_file: EG_ROOK_OPEN_FILE_BONUS,

            mg_rook_semiopen_file: MG_ROOK_SEMI_OPEN_FILE_BONUS,
            eg_rook_semiopen_file: EG_ROOK_SEMI_OPEN_FILE_BONUS,
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
