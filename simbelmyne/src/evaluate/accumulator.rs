use arrayvec::ArrayVec;
use chess::board::Board;
use chess::movegen::castling::CastleType;
use chess::movegen::moves::Move;
use chess::piece::Piece;
use chess::square::Square;
use chess::piece::PieceType;
use crate::position::Position;
use crate::search::params::MAX_DEPTH;
use crate::zobrist::ZHash;

use super::pawn_cache::PawnCache;
use super::tuner::NullTrace;
use super::{Eval, Score};

// Helper consts to make generic parameters more readable.
const WHITE: bool = true;
const BLACK: bool = false;

pub struct EvalState {
    accumulators: [Accumulator; MAX_DEPTH],
    pub current_acc: usize,
}

impl EvalState {
    pub fn reset(&mut self) {
        self.current_acc = 0;
    }

    pub fn init(&mut self, pos: &Position) {
        self.accumulators[0] = Accumulator::new(&pos.board, pos.pawn_hash);
        self.current_acc = 0;
    }

    pub fn push_update(&mut self, update: Update) {
        self.get_current().pending = update;
        self.current_acc += 1;
    }

    pub fn get_current(&mut self) -> &mut Accumulator {
        &mut self.accumulators[self.current_acc]
    }

    /// Scan back through the stack of accumulators until we find a correct one.
    /// Then, iteratively evaluate each dirty accumulator until we are back up
    /// to date.
    pub fn force(&mut self, pawn_cache: &mut PawnCache) {
        // 1) Find the last correct eval
        let mut last_correct = self.current_acc;

        while self.accumulators[last_correct].dirty {
            last_correct -= 1;
        }

        // 2) Update each eval from the last correct one to the current one
        for idx in last_correct..self.current_acc {
            self.accumulators[idx+1].eval = self
                .accumulators[idx+1]
                .apply_update(&self.accumulators[idx].eval, pawn_cache);

            self.accumulators[idx+1].dirty = false;
        }
    }

    pub fn eval(&self, board: &Board) -> Score {
        self.accumulators[self.current_acc].eval.total(board, &mut NullTrace)
    }
}

impl Default for EvalState {
    fn default() -> Self {
        Self {
            accumulators: std::array::from_fn(|_| Accumulator::default()),
            current_acc: 0,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
//
// Accumulator
//
////////////////////////////////////////////////////////////////////////////////

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Accumulator {
    /// The eval struct for this accumulator
    /// May be an out-of-date eval. See the `dirty` flag to figure out if 
    /// the eval is valid.
    eval: Eval,

    /// The board state that should be used for evaluating this accumulator
    pub board: Board,

    /// The pawn hash for the associated board position
    pub pawn_hash: ZHash,

    /// The update that is to be applied to the _previous_ accumulator eval to
    /// get the up-to-date eval for this accumulator.
    pub pending: Update,

    /// Whether or not the accumulator eval is up-to-date.
    dirty: bool,
}

impl Accumulator {
    pub fn new(board: &Board, pawn_hash: ZHash) -> Self {
        Self {
            eval: Eval::new(board, &mut NullTrace),
            board: board.clone(),
            pawn_hash,
            pending: Update::default(),
            dirty: false,
        }
    }

    pub fn apply_update(&self, old_eval: &Eval, pawn_cache: &mut PawnCache) -> Eval {
        let mut new_eval = old_eval.clone();
        let mut dirty_pieces = PieceTypeSet::new();

        for (piece, sq) in self.pending.added.iter() {
            new_eval.game_phase += Eval::phase_value(*piece);
            new_eval.material += new_eval.material(*piece, &mut NullTrace);
            new_eval.psqt += new_eval.psqt(*piece, *sq, &mut NullTrace);
            dirty_pieces.add(piece.piece_type());
        }

        for (piece, sq) in self.pending.removed.iter() {
            new_eval.game_phase -= Eval::phase_value(*piece);
            new_eval.material -= new_eval.material(*piece, &mut NullTrace);
            new_eval.psqt -= new_eval.psqt(*piece, *sq, &mut NullTrace);
            dirty_pieces.add(piece.piece_type());
        }

        for ptype in dirty_pieces {
            new_eval.update_incremental_terms(
                ptype,
                &self.board, 
                self.pawn_hash, 
                pawn_cache
            );
        }

        new_eval
    }
}

////////////////////////////////////////////////////////////////////////////////
//
// Update
//
////////////////////////////////////////////////////////////////////////////////

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Update {
    pub added: ArrayVec<(Piece, Square), 2>,
    pub removed: ArrayVec<(Piece, Square), 2>,
}

pub trait AsUpdate {
    fn as_update(&self, mv: Move) -> Update;
}

impl AsUpdate for Board {
    fn as_update(&self, mv: Move) -> Update {
        let mut update = Update::default();
        let piece = self.get_at(mv.src()).unwrap();
        let new_piece = if mv.is_promotion() {
            Piece::new(mv.get_promo_type().unwrap(), self.current)
        } else {
            piece
        };

        update.removed.push((piece, mv.src()));
        update.added.push((new_piece, mv.tgt()));

        if mv.is_capture() {
            let capture_sq = mv.get_capture_sq();
            let captured = self.get_at(capture_sq).unwrap();
            update.removed.push((captured, capture_sq));
        } else if mv.is_castle() {
            let castle_type = CastleType::from_move(mv).unwrap();
            let rook_move = castle_type.rook_move();
            let rook = self.get_at(rook_move.src()).unwrap();
            update.removed.push((rook, rook_move.src()));
            update.added.push((rook, rook_move.tgt()));
        }

        update
    }
}

////////////////////////////////////////////////////////////////////////////////
//
// PieceTypeSet
//
////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
pub struct PieceTypeSet(u8);

impl PieceTypeSet {
    pub fn new() -> Self {
        Self(0)
    }

    pub fn add(&mut self, ptype: PieceType) {
        self.0 |= 1 << ptype as usize;
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
