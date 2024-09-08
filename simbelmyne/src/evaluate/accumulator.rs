use arrayvec::ArrayVec;
use chess::board::Board;
use chess::piece::Piece;
use chess::square::Square;
use chess::piece::PieceType;
use super::{endgame_scaling, Eval, EvalContext};
use super::params::TEMPO_BONUS;
use super::tuner::{NullTrace, Trace};
pub use super::util::*;

// Helper consts to make generic parameters more readable.
const WHITE: bool = true;
const BLACK: bool = false;

////////////////////////////////////////////////////////////////////////////////
//
// Accumulator
//
////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
struct Update {
    added: ArrayVec<(Piece, Square), 2>,
    removed: ArrayVec<(Piece, Square), 2>,
}

#[derive(Default)]
struct PieceTypeSet(u8);

impl PieceTypeSet {
    pub fn new() -> Self {
        Self(0)
    }

    pub fn add(&mut self, ptype: PieceType) {
        self.0 |= 1 << ptype as usize;
    }

    pub fn has(&self, ptype: PieceType) -> bool {
        self.0 & 1 << ptype as usize != 0
    }
}

impl Iterator for PieceTypeSet {
    type Item = PieceType;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0 != 0 {
            None
        } else {
            #[allow(clippy::cast_possible_truncation)]
            let lsb: u8 = self.0.trailing_zeros() as u8;

            self.0 &= self.0 - 1;

            // SAFETY: 
            // We made sure the set is not empty, so `u64::trailing_zeros`
            // can only return a number between 0..=6, which are valid PieceType
            // indices.
            Some(unsafe { std::mem::transmute::<u8, PieceType>(lsb) })
        }
    }
}

#[derive(Default)]
pub struct Accumulator {
    correct: bool,
    pending: Update,
    psqt: S,
    material: S,
    dirty: PieceTypeSet,
    incremental: Eval,
}

impl Accumulator {
    pub fn new(board: &Board) -> Self {
        let mut acc = Self::default();

        for (sq_idx, piece) in board.piece_list.into_iter().enumerate() {
            if let Some(piece) = piece {
                let sq = Square::from(sq_idx);
                acc.material += acc.incremental.material(piece, &mut NullTrace);
                acc.psqt += acc.incremental.psqt(piece, sq, &mut NullTrace);
            }
        }

        acc.correct = true;
        acc
    }

    /// Return the total (tapered) score for the position as the sum of the
    /// incremental evaluation terms and the volatile terms.
    pub fn eval(&mut self, board: &Board, trace: &mut impl Trace) -> Score {
        // We pass around an EvalContext so expensive information gathered in 
        // some evaluation terms can be shared with other eval terms, instead
        // of recomputing them again.
        let mut ctx = EvalContext::new(board);

        // Add up all of the incremental terms stored on the Eval struct
        let mut total = self.material;
        total += self.psqt;
        total += self.incremental.pawn_structure.score();
        total += self.incremental.pawn_shield;
        total += self.incremental.pawn_storm;
        total += self.incremental.passers;
        total += self.incremental.knight_outposts;
        total += self.incremental.bishop_outposts;
        total += self.incremental.knight_shelter;
        total += self.incremental.bishop_shelter;
        total += self.incremental.bishop_pair;
        total += self.incremental.rook_open_file;
        total += self.incremental.rook_semiopen_file;
        total += self.incremental.queen_open_file;
        total += self.incremental.queen_semiopen_file;
        total += self.incremental.major_on_seventh;
        total += self.incremental.bad_bishops;

        // Compute and add up the "volatile" evaluation terms. These are the 
        // terms that need to get recomputed in every node, anyway.
        total += self.incremental.connected_rooks::<WHITE>(board, trace);
        total -= self.incremental.connected_rooks::<BLACK>(board, trace);
        total += self.incremental.mobility::<WHITE>(board, &mut ctx, trace);
        total -= self.incremental.mobility::<BLACK>(board, &mut ctx, trace);
        total += self.incremental.virtual_mobility::<WHITE>(board, trace);
        total -= self.incremental.virtual_mobility::<BLACK>(board, trace);
        total += self.incremental.king_zone::<WHITE>(&mut ctx, trace);
        total -= self.incremental.king_zone::<BLACK>(&mut ctx, trace);
        total += self.incremental.threats::<WHITE>(&ctx, trace);
        total -= self.incremental.threats::<BLACK>(&ctx, trace);
        total += self.incremental.safe_checks::<WHITE>(board, &ctx, trace);
        total -= self.incremental.safe_checks::<BLACK>(board, &ctx, trace);
        total += self.incremental.volatile_passers::<WHITE>(board, &ctx, trace);
        total -= self.incremental.volatile_passers::<BLACK>(board, &ctx, trace);

        // Add a side-relative tempo bonus
        // The position should be considered slightly more advantageous for the
        // current side-to-move.
        let perspective = if board.current.is_white() { 1 } else { -1 };
        total += TEMPO_BONUS * perspective;
        trace.add(|t| t.tempo += perspective);

        // Downscale the endgame score depending on how drawish the position is
        let eg_scaling = endgame_scaling(board, total.eg());
        let total = S::new(total.mg(), total.eg() * eg_scaling / 128);
        trace.add(|t| t.eg_scaling = eg_scaling);

        // Interpolate between midgame and endgame evals, taking into account
        // the endgame scaling.
        let score = total.lerp(board.phase());

        // Return the score relative to the current side-to-move
        perspective * score
    }
}
