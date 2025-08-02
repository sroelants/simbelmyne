use arrayvec::ArrayVec;
use chess::board::Board;
use chess::movegen::moves::Move;
use chess::piece::Color;
use chess::piece::PieceType;
use chess::square::Square;
use corrhist::CorrHistEntry;
use corrhist::Hash;
use corrhist::CORRHIST_SIZE;
use history::Butterfly;
use history::HistoryIndex;
use history::HistoryScore;
use killers::Killers;
use threats::ThreatIndex;
use threats::Threats;

use crate::search::params::MAX_DEPTH;
use crate::zobrist::ZHash;

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
  pub pawn_corr: [Hash<CorrHistEntry, CORRHIST_SIZE>; Color::COUNT],
  pub w_nonpawn_corr: [Hash<CorrHistEntry, CORRHIST_SIZE>; Color::COUNT],
  pub b_nonpawn_corr: [Hash<CorrHistEntry, CORRHIST_SIZE>; Color::COUNT],
  pub minor_corr: [Hash<CorrHistEntry, CORRHIST_SIZE>; Color::COUNT],
  pub mat_corr: [Hash<CorrHistEntry, CORRHIST_SIZE>; Color::COUNT],
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
  pub fn add_hist_bonus(
    &mut self,
    mv: Move,
    board: &Board,
    bonus: HistoryScore,
  ) {
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

  pub fn clear_countermoves(&mut self) {
    self.countermoves = Butterfly::default();
  }

  // Node counter
  pub fn add_nodes(&mut self, mv: Move, nodes: u32) {
    self.node_counts[mv.src()][mv.tgt()] += nodes;
  }

  pub fn get_nodes(&self, mv: Move) -> u32 {
    self.node_counts[mv.src()][mv.tgt()]
  }

  pub fn clear_nodes(&mut self) {
    self.node_counts = [[0; Square::COUNT]; Square::COUNT];
  }
}
