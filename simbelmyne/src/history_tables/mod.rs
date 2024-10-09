use arrayvec::ArrayVec;
use capthist::TacticalHistoryTable;
use chess::{board::Board, movegen::moves::Move, piece::PieceType, square::Square};
use conthist::ContHist;
use corrhist::CorrHistTable;
use countermoves::CountermoveTable;
use history::{HistoryIndex, HistoryScore};
use killers::Killers;
use threats::{ThreatIndex, ThreatsHistoryTable};

use crate::{search::params::MAX_DEPTH, zobrist::ZHash};

pub mod history;
pub mod threats;
pub mod conthist;
pub mod killers;
pub mod countermoves;
pub mod pv;
pub mod capthist;
pub mod corrhist;

#[derive(Debug)]
pub struct History {
    pub main_hist: Box<ThreatsHistoryTable>,
    pub cont_hist: Box<ContHist>,
    pub tact_hist: Box<TacticalHistoryTable>,
    pub corr_hist: Box<CorrHistTable>,
    countermoves: Box<CountermoveTable>,
    pub killers: [Killers; MAX_DEPTH],
    pub indices: ArrayVec<HistoryIndex, MAX_DEPTH>,
    rep_hist: ArrayVec<(u8, ZHash), MAX_DEPTH>,
    node_counts: [[u32; Square::COUNT]; Square::COUNT],
}

impl History {
    pub fn new() -> Self {
        Self {
            main_hist: ThreatsHistoryTable::boxed(),
            cont_hist: ContHist::boxed(),
            tact_hist: TacticalHistoryTable::boxed(),
            countermoves: CountermoveTable::boxed(),
            corr_hist: CorrHistTable::boxed(),
            killers: [Killers::new(); MAX_DEPTH],
            indices: ArrayVec::new(),
            rep_hist: ArrayVec::new(),
            node_counts: [[0; Square::COUNT]; Square::COUNT],
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
    pub fn add_hist_bonus(
        &mut self, 
        mv: Move, 
        board: &Board, 
        bonus: HistoryScore
    ) {
        let idx = HistoryIndex::new(board, mv);
        let threat_idx = ThreatIndex::new(board.threats, mv);

        if mv.is_tactical() {
            let victim = if let Some(piece) = board.get_at(mv.tgt()) {
                piece.piece_type()
            } else {
                PieceType::Pawn
            };

            self.tact_hist[victim][threat_idx][idx] += bonus;
        } 

        else {
            self.main_hist[threat_idx][idx] += bonus;

            if let Some(oneply) = self.indices.len()
                .checked_sub(1)
                .map(|ply| self.indices[ply]) {
                self.cont_hist[oneply][idx] += bonus;
            }

            if let Some(twoply) = self.indices.len()
                .checked_sub(2)
                .map(|ply| self.indices[ply]) {
                self.cont_hist[twoply][idx] += bonus;
            }

            if let Some(fourply) = self.indices.len()
                .checked_sub(4)
                .map(|ply| self.indices[ply]) {
                self.cont_hist[fourply][idx] += bonus;
            }
        }
    }

    pub fn get_hist_score(&self, mv: Move, board: &Board) -> i32 {
        let idx = HistoryIndex::new(board, mv);
        let threat_idx = ThreatIndex::new(board.threats, mv);

        if mv.is_tactical() {
            let victim = if let Some(piece) = board.get_at(mv.tgt()) {
                piece.piece_type()
            } else {
                PieceType::Pawn
            };

            i32::from(self.tact_hist[victim][threat_idx][idx])
        } else {
            let threat_idx = ThreatIndex::new(board.threats, mv);
            let mut total = i32::from(self.main_hist[threat_idx][idx]);

            if let Some(oneply) = self.indices.len()
                .checked_sub(1)
                .map(|ply| self.indices[ply]) {
                total += i32::from(self.cont_hist[oneply][idx]);
            }

            if let Some(twoply) = self.indices.len()
                .checked_sub(2)
                .map(|ply| self.indices[ply]) {
                total += i32::from(self.cont_hist[twoply][idx]);
            }

            if let Some(fourply) = self.indices.len()
                .checked_sub(4)
                .map(|ply| self.indices[ply]) {
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
        self.countermoves = CountermoveTable::boxed();
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
}
