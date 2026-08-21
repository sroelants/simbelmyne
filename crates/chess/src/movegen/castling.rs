use crate::bitboard::Bitboard;
use crate::board::Board;
use crate::movegen::castling;
use crate::movegen::lookups;
use crate::movegen::lookups::between_incl;
use crate::movegen::moves::Move;
use crate::movegen::moves::MoveType;
use crate::piece::Color;
use crate::square::Square;
use Square::*;
use anyhow::anyhow;
use std::fmt::Display;
use std::str::FromStr;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct CastlingRights(Bitboard);

impl CastlingRights {
  pub fn get_for(&self, side: Color) -> Bitboard {
    if side.is_white() {
      self.0 & lookups::rank(0)
    } else {
      self.0 & lookups::rank(7)
    }
  }

  pub fn remove_for(&mut self, side: Color) {
    if side.is_white() {
      self.0 &= !lookups::rank(0)
    } else {
      self.0 &= !lookups::rank(7)
    }
  }

  pub fn remove(&mut self, sq: Square) {
    self.0 &= !sq.bb();
  }

  pub fn has(self, sq: Square) -> bool {
    self.0.contains(sq)
  }

  pub fn bb(self) -> Bitboard {
    self.0
  }
}

impl Display for CastlingRights {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    if self.0.is_empty() {
      write!(f, "-")?;
      return Ok(());
    }

    if self.0.contains(H1) {
      write!(f, "K")?;
    }

    if self.0.contains(A1) {
      write!(f, "Q")?;
    }

    if self.0.contains(H8) {
      write!(f, "k")?;
    }

    if self.0.contains(A8) {
      write!(f, "q")?;
    }

    Ok(())
  }
}

impl FromStr for CastlingRights {
  type Err = anyhow::Error;

  /// Parse the castling rights from a FEN string
  /// rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1
  ///                                               ^^^^
  fn from_str(castling_str: &str) -> Result<Self, Self::Err> {
    let mut rights = CastlingRights::default();

    for ch in castling_str.chars() {
      match ch {
        'Q' => rights.0 |= A1.into(),
        'K' => rights.0 |= H1.into(),
        'q' => rights.0 |= A8.into(),
        'k' => rights.0 |= H8.into(),
        '-' => {}
        _ => Err(anyhow!("Invalid FEN string"))?,
      }
    }

    Ok(rights)
  }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CastleType {
  Long,
  Short,
}

impl CastleType {
  pub fn mtype(self) -> MoveType {
    match self {
      Self::Long => MoveType::QueenCastle,
      Self::Short => MoveType::KingCastle,
    }
  }

  pub fn from_move(mv: Move) -> Self {
    debug_assert!(mv.is_castle(), "Tried to get castle type from non-castle");

    if mv.tgt() < mv.src() {
      Self::Long
    } else {
      Self::Short
    }
  }
}

const KING_TARGETS: [[Square; 2]; 2] = [[C1, G1], [C8, G8]];
const ROOK_TARGETS: [[Square; 2]; 2] = [[D1, F1], [D8, F8]];

#[inline(always)]
pub fn rook_target(king_sq: Square, rook_sq: Square) -> Square {
  ROOK_TARGETS[(king_sq > H1) as usize][(rook_sq > king_sq) as usize]
}

#[inline(always)]
pub fn king_target(king_sq: Square, rook_sq: Square) -> Square {
  KING_TARGETS[(king_sq > H1) as usize][(rook_sq > king_sq) as usize]
}

// Can we make this cheaper for regular chess? By hardcoding the travel bitboards?
// problem is: how do we get the bitboards?
// well, sp still does the if checks, as well... So I guess we can, too
impl Board {
  pub fn is_legal_castle(&self, king_src: Square, rook_src: Square) -> bool {
    let threats = self.threats;
    let blockers = self.all_occupied() & !king_src.bb() & !rook_src.bb();

    let king_tgt = castling::king_target(king_src, rook_src);
    let rook_tgt = castling::rook_target(king_src, rook_src);
    let king_travel = between_incl(king_src, king_tgt);
    let rook_travel = between_incl(rook_src, rook_tgt);
    let travel = king_travel | rook_travel;

    (king_travel & threats).is_empty() && (travel & blockers).is_empty()
  }
}
