use arrayvec::ArrayVec;
use chess::{
    board::Board,
    movegen::moves::Move,
    piece::{Color, PieceType},
    square::Square,
};
use corrhist::{CorrHistEntry, Hash, CORRHIST_SIZE};
use history::{Butterfly, HistoryIndex, HistoryScore};
use killers::Killers;
use threats::{ThreatIndex, Threats};

use crate::{evaluate::Score, position::Position, search::params::{cont_corr_weight, material_corr_weight, minor_corr_weight, nonpawn_corr_weight, pawn_corr_weight, MAX_DEPTH}, zobrist::ZHash};

pub mod corrhist;
pub mod history;
pub mod killers;
pub mod pv;
pub mod threats;

#[derive(Debug)]
pub struct History {
    pub main_hist: Threats<Butterfly<HistoryScore>>,
    pub cont_hist: Butterfly<Butterfly<HistoryScore>>,
    pub tact_hist: [Butterfly<HistoryScore>; PieceType::COUNT],
    pub corr_hist: [Hash<CorrHistEntry, CORRHIST_SIZE>; Color::COUNT],
    pub contcorr_hist: Butterfly<CorrHistEntry>,
    pub countermoves: Butterfly<Option<Move>>,
    pub killers: [Killers; MAX_DEPTH],
    pub indices: ArrayVec<HistoryIndex, MAX_DEPTH>,
    rep_hist: ArrayVec<(u8, ZHash), MAX_DEPTH>,
    node_counts: [[u32; Square::COUNT]; Square::COUNT],
}

impl History {
    pub fn boxed() -> Box<Self> {
        #![allow(clippy::cast_ptr_alignment)]
        // SAFETY: we're allocating a zeroed block of memory, and then casting
        // it to a Box<Self>. This is fine!
        // [[HistoryTable; Square::COUNT]; Piece::COUNT] is just a bunch of i16s
        // in disguise, which are fine to zero-out.
        unsafe {
            let layout = std::alloc::Layout::new::<Self>();
            let ptr = std::alloc::alloc_zeroed(layout);
            if ptr.is_null() {
                std::alloc::handle_alloc_error(layout);
            }
            Box::from_raw(ptr.cast())
        }
    }

    // History indices
    pub fn push_mv(&mut self, mv: Move, board: &Board) {
        self.indices.push(HistoryIndex::new(board, mv));
    }

    pub fn push_null_mv(&mut self) {
        self.indices.push(HistoryIndex::default());
    }

    pub fn pop_mv(&mut self) {
        self.indices.pop();
    }

    // Repitition history
    pub fn push_rep_entry(&mut self, halfmoves: u8, hash: ZHash) {
        self.rep_hist.push((halfmoves, hash));
    }
    pub fn pop_rep_entry(&mut self) {
        self.rep_hist.pop();
    }

    // Update History tables
    pub fn add_hist_bonus(&mut self, mv: Move, board: &Board, bonus: HistoryScore) {
        let idx = HistoryIndex::new(board, mv);

        if mv.is_tactical() {
            let victim = if let Some(piece) = board.get_at(mv.tgt()) {
                piece.piece_type()
            } else {
                PieceType::Pawn
            };

            self.tact_hist[victim][idx] += bonus;
        } else {
            let threat_idx = ThreatIndex::new(board.threats, mv);
            self.main_hist[threat_idx][idx] += bonus;

            if let Some(oneply) = self
                .indices
                .len()
                .checked_sub(1)
                .map(|ply| self.indices[ply])
            {
                self.cont_hist[oneply][idx] += bonus;
            }

            if let Some(twoply) = self
                .indices
                .len()
                .checked_sub(2)
                .map(|ply| self.indices[ply])
            {
                self.cont_hist[twoply][idx] += bonus;
            }

            if let Some(fourply) = self
                .indices
                .len()
                .checked_sub(4)
                .map(|ply| self.indices[ply])
            {
                self.cont_hist[fourply][idx] += bonus;
            }
        }
    }

    pub fn get_hist_score(&self, mv: Move, board: &Board) -> i32 {
        let idx = HistoryIndex::new(board, mv);

        if mv.is_tactical() {
            let victim = if let Some(piece) = board.get_at(mv.tgt()) {
                piece.piece_type()
            } else {
                PieceType::Pawn
            };

            i32::from(self.tact_hist[victim][idx])
        } else {
            let threat_idx = ThreatIndex::new(board.threats, mv);
            let mut total = i32::from(self.main_hist[threat_idx][idx]);

            if let Some(oneply) = self
                .indices
                .len()
                .checked_sub(1)
                .map(|ply| self.indices[ply])
            {
                total += i32::from(self.cont_hist[oneply][idx]);
            }

            if let Some(twoply) = self
                .indices
                .len()
                .checked_sub(2)
                .map(|ply| self.indices[ply])
            {
                total += i32::from(self.cont_hist[twoply][idx]);
            }

            if let Some(fourply) = self
                .indices
                .len()
                .checked_sub(4)
                .map(|ply| self.indices[ply])
            {
                total += i32::from(self.cont_hist[fourply][idx]);
            }

            total
        }
    }

    // Countermove table
    pub fn add_countermove(&mut self, mv: Move) {
        if let Some(&oneply) = self.indices.last() {
            self.countermoves[oneply] = Some(mv);
        }
    }

    pub fn get_countermove(&self) -> Option<Move> {
        self.indices.last().and_then(|&idx| self.countermoves[idx])
    }

    // Killers
    pub fn add_killer(&mut self, ply: usize, mv: Move) {
        self.killers[ply].add(mv);
    }

    pub fn clear_killers(&mut self, ply: usize) {
        self.killers[ply].clear();
    }

    pub fn clear_all_killers(&mut self) {
        self.killers = [Killers::new(); MAX_DEPTH];
    }

    pub fn clear_countermoves(&mut self) {
        self.countermoves = Butterfly::default();
    }

    pub fn clear_nodes(&mut self) {
        self.node_counts = [[0; Square::COUNT]; Square::COUNT];
    }

    pub fn add_nodes(&mut self, mv: Move, nodes: u32) {
        self.node_counts[mv.src()][mv.tgt()] += nodes;
    }

    pub fn get_nodes(&self, mv: Move) -> u32 {
        self.node_counts[mv.src()][mv.tgt()]
    }

    // Corrhist
    pub fn correct_eval(&self, pos: &Position, ply: usize, eval: Score) -> Score {
        use Color::*;
        let us = pos.board.current;

        let pawn_correction = self.corr_hist[us][pos.pawn_hash].corr();
        let w_nonpawn_correction = self.corr_hist[us][pos.nonpawn_hashes[White]].corr();
        let b_nonpawn_correction = self.corr_hist[us][pos.nonpawn_hashes[Black]].corr();
        let material_correction = self.corr_hist[us][pos.material_hash].corr();
        let minor_correction = self.corr_hist[us][pos.minor_hash].corr();
        let cont_correction = self
            .indices
            .get(ply - 2)
            .map(|idx| self.contcorr_hist[*idx].corr())
            .unwrap_or_default();

        let correction =
              pawn_corr_weight()     * pawn_correction
            + nonpawn_corr_weight()  * w_nonpawn_correction
            + nonpawn_corr_weight()  * b_nonpawn_correction
            + material_corr_weight() * material_correction
            + minor_corr_weight()    * minor_correction
            + cont_corr_weight()     * cont_correction;

        eval + correction
    }
    pub fn update_corrhist(
        &mut self,
        pos: &Position,
        ply: usize,
        depth: usize,
        score: Score,
        eval: Score,
    ) {
        use Color::*;
        let us = pos.board.current;

        // Update the pawn corrhist
        self.corr_hist[us][pos.pawn_hash].update(score, eval, depth);

        // Update the non-pawn corrhist
        self.corr_hist[us][pos.nonpawn_hashes[White]].update(score, eval, depth);
        self.corr_hist[us][pos.nonpawn_hashes[Black]].update(score, eval, depth);

        // Update the material corrhist
        self.corr_hist[us][pos.material_hash].update(score, eval, depth);

        // Update the minor corrhist
        self.corr_hist[us][pos.minor_hash].update(score, eval, depth);

        // Update the cont corrhist
        if let Some(idx) = self.indices.get(ply - 2) {
            self.contcorr_hist[*idx].update(score, eval, depth);
        }
    }
}
