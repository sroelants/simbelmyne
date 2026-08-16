use crate::zobrist::{self as z, ZHash};
use chess::{
  bitboard::Bitboard,
  piece::{Piece, PieceType},
  square::Square,
};

static mut CUCKOO: [CuckooEntry; 8192] = [CuckooEntry::NULL; _];

pub struct CuckooEntry {
  hash: ZHash,
  from: Square,
  to: Square,
}

impl CuckooEntry {
  const NULL: Self = Self {
    hash: ZHash::NULL,
    from: Square::A1,
    to: Square::A1,
  };
}

pub fn init() {
  use PieceType::*;
  let mut count = 0;
  let mut table = [CuckooEntry::NULL; 8192];

  for piece in Piece::ALL {
    if piece.piece_type() == Pawn {
      continue;
    }

    for from in 0..64 {
      for to in (from + 1)..64 {
        let from = Square::from(from);
        let to = Square::from(to);
        let diff = z::piece(piece, from) ^ z::piece(piece, to) ^ z::side();

        // Check whether the piece in question can move to the `to` square.
        if !from.attacks_for(piece, Bitboard::EMPTY).contains(to) {
          continue;
        }

        let mut entry = CuckooEntry {
          hash: diff,
          from,
          to,
        };

        let mut idx = h1(diff);

        // Loop and keep evicting entries until we've hit an empty slot.
        loop {
          std::mem::swap(&mut table[idx], &mut entry);

          if hash(idx) == ZHash::NULL {
            break;
          }

          if idx == h1(diff) {
            idx = h2(diff)
          } else {
            idx = h1(diff)
          }
        }

        count += 1;
      }
    }
  }

  assert_eq!(count, 3668);

  unsafe { CUCKOO = table };
}

pub fn h1(hash: ZHash) -> usize {
  ((hash.0) & 0x1fff) as usize
}

pub fn h2(hash: ZHash) -> usize {
  ((hash.0 >> 16) & 0x1fff) as usize
}

pub fn hash(key: usize) -> ZHash {
  unsafe { CUCKOO[key].hash }
}

pub fn from(key: usize) -> Square {
  unsafe { CUCKOO[key].from }
}

pub fn to(key: usize) -> Square {
  unsafe { CUCKOO[key].to }
}

#[cfg(test)]
pub mod tests {
  use super::*;
  use crate::position::Position;
  use Piece::*;
  use Square::*;
  use chess::movegen::moves::BareMove;

  fn prepare_position() -> Position {
    use Square::*;
    let board = "3k4/8/5r2/8/8/1R6/8/3K4 w - - 0 1".parse().unwrap();
    let mut pos = Position::new(board);
    let moves = [
      BareMove::new(B3, B2, None),
      BareMove::new(F6, F7, None),
      BareMove::new(B2, B3, None),
    ];

    for mv in moves {
      pos = pos.play_bare_move(mv);
    }

    pos
  }

  #[test]
  fn move_diffs() {
    super::init();

    // Check that the xor of two positions is the same as the constructed
    // move diff
    let pos = prepare_position();
    let diff = pos.history[0] ^ pos.history[1];
    let mv = z::piece(WR, B3) ^ z::piece(WR, B2) ^ z::side();

    assert_eq!(diff, mv);
  }

  #[test]
  fn squares() {
    super::init();
    let mv = z::piece(WR, B3) ^ z::piece(WR, B2) ^ z::side();

    let mut key = h1(mv);
    if hash(key) != mv {
      key = h2(mv);
    }

    assert!(hash(key) == mv);
    assert!(from(key) == B2 || to(key) == B2);
    assert!(from(key) == B3 || to(key) == B3);
    assert!(from(key) != to(key));
  }
}
