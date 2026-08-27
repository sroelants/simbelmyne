// FIXME:
//   - Get rid of `pawn_squares`
//   - Scrap most of the compile-time generated stuff, we can express it much
//     neater now with const impls and const blocks.
use crate::bitboard::Bitboard;
use crate::piece::Color;
use crate::square::Square;

/// Look up the Bitboard of all squares between two squares, excluding the
/// endpoints.
pub const fn between(sq1: Square, sq2: Square) -> Bitboard {
  BETWEEN[sq1 as usize][sq2 as usize]
}

/// Return the ray from `origin` passing through `target` and onwards.
pub const fn rays(origin: Square, target: Square) -> Bitboard {
  RAYS[origin as usize][target as usize]
}

/// Get a bitboard for all the squares under attack by a pawn on this
/// square.
#[inline(always)]
pub const fn pawn_attacks(sq: Square, side: Color) -> Bitboard {
  PAWN_ATTACKS[side][sq]
}

#[inline(always)]
pub const fn pawn_pushes(
  sq: Square,
  side: Color,
  blockers: Bitboard,
) -> Bitboard {
  PAWN_PUSHES[side][sq] & !blockers
}

// FIXME: Get rid of this
/// Get a bitboard for all the squares visible to a pawn on this square
pub const fn pawn_squares(
  sq: Square,
  side: Color,
  blockers: Bitboard,
) -> Bitboard {
  let push_mask = PAWN_PUSHES[side][sq];
  let dbl_push_mask = PAWN_DBLPUSHES[side][sq];

  let on_original_rank = if side.is_white() {
    sq.rank() == 1
  } else {
    sq.rank() == 6
  };

  let can_push = (push_mask & blockers).is_empty();
  let can_dbl_push =
    on_original_rank && can_push && (dbl_push_mask & blockers).is_empty();

  if can_dbl_push {
    push_mask | dbl_push_mask
  } else if can_push {
    push_mask
  } else {
    Bitboard::EMPTY
  }
}

/// Get a bitboard for all the squares visible to a knight on this square.
#[inline(always)]
pub const fn knight_squares(sq: Square) -> Bitboard {
  KNIGHT_ATTACKS[sq]
}

/// Get a bitboard for all the squares visible to a bishop on this square.
#[inline(always)]
pub fn bishop_squares(sq: Square, blockers: Bitboard) -> Bitboard {
  #[cfg(target_feature = "bmi2")]
  return crate::movegen::pext::bishop_squares(sq, blockers);

  #[cfg(not(target_feature = "bmi2"))]
  return crate::movegen::magics::bishop_squares(sq, blockers);
}

/// Get a bitboard for all the squares visible to a rook on this square.
#[inline(always)]
pub fn rook_squares(sq: Square, blockers: Bitboard) -> Bitboard {
  #[cfg(target_feature = "bmi2")]
  return crate::movegen::pext::rook_squares(sq, blockers);

  #[cfg(not(target_feature = "bmi2"))]
  return crate::movegen::magics::rook_squares(sq, blockers);
}

/// Get a bitboard for all the squares visible to a queen on this square.
#[inline(always)]
pub fn queen_squares(sq: Square, blockers: Bitboard) -> Bitboard {
  bishop_squares(sq, blockers) | rook_squares(sq, blockers)
}

/// Get a bitboard for all the squares visible to a king on this square.
#[inline(always)]
pub const fn king_squares(sq: Square) -> Bitboard {
  KING_ATTACKS[sq]
}

// ---- Lookup generation ----

// For internal use as more readable const parameters
const WHITE: bool = true;
const BLACK: bool = false;

type BBTable = [Bitboard; 64];
type BBBTable = [[Bitboard; 64]; 64];

// Pawn attacks

pub const PAWN_PUSHES: [BBTable; Color::COUNT] =
  [gen_pawn_pushes::<WHITE>(), gen_pawn_pushes::<BLACK>()];

pub const PAWN_DBLPUSHES: [BBTable; Color::COUNT] = [
  gen_pawn_double_pushes::<WHITE>(),
  gen_pawn_double_pushes::<BLACK>(),
];

pub const PAWN_ATTACKS: [BBTable; Color::COUNT] =
  [gen_pawn_attacks::<WHITE>(), gen_pawn_attacks::<BLACK>()];

/// Generate pawn push squares from a given square
const fn gen_pawn_pushes<const WHITE: bool>() -> BBTable {
  let mut bbs: BBTable = [Bitboard(0); 64];
  let mut square: usize = 0;

  while square < 64 {
    let rank = square / 8;
    let mut bitboard: u64 = 0;

    if WHITE {
      if rank < 7 {
        let up = square + 8;
        bitboard |= 1 << up
      }
    } else {
      if rank > 0 {
        let down = square - 8;
        bitboard |= 1 << down
      }
    }

    bbs[square] = Bitboard(bitboard);
    square += 1
  }

  bbs
}

/// Generate pawn double push squares from a given square
const fn gen_pawn_double_pushes<const WHITE: bool>() -> BBTable {
  let mut bbs: BBTable = [Bitboard(0); 64];
  let mut square: usize = 0;

  while square < 64 {
    let rank = square / 8;
    let mut bitboard: u64 = 0;

    if WHITE {
      if rank == 1 {
        let upup = square + 16;
        bitboard |= 1 << upup
      }
    } else {
      if rank == 6 {
        let downdown = square - 16;
        bitboard |= 1 << downdown
      }
    }

    bbs[square] = Bitboard(bitboard);
    square += 1
  }

  bbs
}

/// Generate pawn attack squares from a given square
const fn gen_pawn_attacks<const WHITE: bool>() -> BBTable {
  let mut bbs: BBTable = [Bitboard(0); 64];
  let mut square: usize = 0;

  while square < 64 {
    let file = square % 8;
    let rank = square / 8;
    let mut bitboard: u64 = 0;

    if WHITE {
      if file > 0 && rank < 7 {
        let up_left = square + 7;
        bitboard |= 1 << up_left
      }
      if file < 7 && rank < 7 {
        let up_right = square + 9;
        bitboard |= 1 << up_right;
      }
    } else {
      // BLACK
      if file > 0 && rank > 0 {
        let down_left = square - 9;
        bitboard |= 1 << down_left;
      }
      if file < 7 && rank > 0 {
        let down_right = square - 7;
        bitboard |= 1 << down_right
      }
    }
    bbs[square] = Bitboard(bitboard);
    square += 1
  }

  bbs
}

// Generate Knight attacks

pub const KNIGHT_ATTACKS: BBTable = gen_knight_attacks();

/// Generate knight attack squares from a given square
const fn gen_knight_attacks() -> BBTable {
  let mut bbs: BBTable = [Bitboard(0); 64];
  let mut square: usize = 0;

  while square < 64 {
    let file = square % 8;
    let rank = square / 8;
    let mut bitboard: u64 = 0;

    if file > 1 && rank < 7 {
      let leftleftup = square + 6;
      bitboard |= 1 << leftleftup;
    }

    if file > 0 && rank < 6 {
      let upupleft = square + 15;
      bitboard |= 1 << upupleft;
    }

    if file > 1 && rank > 0 {
      let leftleftdown = square - 10;
      bitboard |= 1 << leftleftdown;
    }

    if file > 0 && rank > 1 {
      let downdownleft = square - 17;
      bitboard |= 1 << downdownleft;
    }

    if file < 6 && rank < 7 {
      let rightrightup = square + 10;
      bitboard |= 1 << rightrightup;
    }

    if file < 7 && rank < 6 {
      let upupright = square + 17;
      bitboard |= 1 << upupright;
    }

    if file < 6 && rank > 0 {
      let rightrightdown = square - 6;
      bitboard |= 1 << rightrightdown;
    }

    if file < 7 && rank > 1 {
      let downdownright = square - 15;
      bitboard |= 1 << downdownright;
    }

    bbs[square] = Bitboard(bitboard);
    square += 1
  }

  bbs
}

// Generate King attacks

pub const KING_ATTACKS: BBTable = gen_king_attacks();

/// Generate king attack squares from a given square
const fn gen_king_attacks() -> BBTable {
  let mut bbs: BBTable = [Bitboard(0); 64];
  let mut square: usize = 0;

  while square < 64 {
    let file = square % 8;
    let rank = square / 8;
    let mut bitboard: u64 = 0;

    if file > 0 {
      let left = square - 1;
      bitboard |= 1 << left;

      if rank > 0 {
        let downleft = square - 9;
        bitboard |= 1 << downleft;
      }

      if rank < 7 {
        let upleft = square + 7;
        bitboard |= 1 << upleft;
      }
    }

    if file < 7 {
      let right = square + 1;
      bitboard |= 1 << right;

      if rank < 7 {
        let upright = square + 9;
        bitboard |= 1 << upright;
      }

      if rank > 0 {
        let downright = square - 7;
        bitboard |= 1 << downright;
      }
    }

    if rank > 0 {
      let down = square - 8;
      bitboard |= 1 << down;
    }

    if rank < 7 {
      let up = square + 8;
      bitboard |= 1 << up;
    }

    bbs[square] = Bitboard(bitboard);
    square += 1
  }

  bbs
}

// Generate Between table

const BETWEEN: BBBTable = const {
  let mut between = [[Bitboard::EMPTY; 64]; 64];
  let mut sq1: usize = 0;

  while sq1 < 64 {
    let mut sq2 = 0;

    while sq2 < 64 {
      between[sq1][sq2] = bb_between(sq1, sq2);
      sq2 += 1;
    }

    sq1 += 1;
  }

  between
};

const fn bb_between(sq1: usize, sq2: usize) -> Bitboard {
  let mut bb: u64 = 0;
  let mut x1 = sq1 % 8;
  let mut y1 = sq1 / 8;
  let mut x2 = sq2 % 8;
  let mut y2 = sq2 / 8;

  // Horizontal
  if x1 == x2 && y1 + 1 < y2 {
    while y1 + 1 < y2 {
      y1 += 1;
      bb |= 1 << (x1 + 8 * y1)
    }
  } else if x1 == x2 && y2 + 1 < y1 {
    while y2 + 1 < y1 {
      y2 += 1;
      bb |= 1 << (x2 + 8 * y2)
    }
  } else if x1 + 1 < x2 && y1 == y2 {
    while x1 + 1 < x2 {
      x1 += 1;
      bb |= 1 << (x1 + 8 * y1)
    }
  } else if x2 + 1 < x1 && y1 == y2 {
    while x2 + 1 < x1 {
      x2 += 1;
      bb |= 1 << (x2 + 8 * y2)
    }
  }
  // Diagonal
  else if x1 + 1 < x2 && y1 + 1 < y2 && x2 - x1 == y2 - y1 {
    while x1 + 1 < x2 && y1 + 1 < y2 {
      x1 += 1;
      y1 += 1;
      bb |= 1 << (x1 + 8 * y1);
    }
  } else if x2 + 1 < x1 && y2 + 1 < y1 && x1 - x2 == y1 - y2 {
    while x2 < x1 - 1 && y2 < y1 - 1 {
      x2 += 1;
      y2 += 1;
      bb |= 1 << (x2 + 8 * y2);
    }
  } else if x1 + 1 < x2 && y2 + 1 < y1 && x2 - x1 == y1 - y2 {
    while x1 + 1 < x2 && y2 + 1 < y1 {
      x1 += 1;
      y1 -= 1;
      bb |= 1 << (x1 + 8 * y1);
    }
  } else if x2 + 1 < x1 && y1 + 1 < y2 && x1 - x2 == y2 - y1 {
    while x2 + 1 < x1 && y1 + 1 < y2 {
      x1 -= 1;
      y1 += 1;
      bb |= 1 << (x1 + 8 * y1);
    }
  }

  Bitboard(bb)
}

// Generate Rays table

const RAYS: BBBTable = const {
  let mut rays = [[Bitboard::EMPTY; 64]; 64];
  let mut sq1: usize = 0;

  while sq1 < 64 {
    let mut sq2 = 0;

    while sq2 < 64 {
      rays[sq1][sq2] = ray_bb(sq1, sq2);
      sq2 += 1;
    }

    sq1 += 1;
  }

  rays
};

const fn ray_bb(sq1: usize, sq2: usize) -> Bitboard {
  let mut bb: u64 = 0;
  let mut x1 = sq1 % 8;
  let mut y1 = sq1 / 8;
  let x2 = sq2 % 8;
  let y2 = sq2 / 8;

  // Horizontal
  if x1 == x2 && y1 < y2 {
    while y1 < 7 {
      y1 += 1;
      bb |= 1 << (x1 + 8 * y1)
    }
  } else if x1 == x2 && y1 > y2 {
    while y1 > 0 {
      y1 -= 1;
      bb |= 1 << (x1 + 8 * y1)
    }
  } else if x1 < x2 && y1 == y2 {
    while x1 < 7 {
      x1 += 1;
      bb |= 1 << (x1 + 8 * y1)
    }
  } else if x2 < x1 && y1 == y2 {
    while x1 > 0 {
      x1 -= 1;
      bb |= 1 << (x1 + 8 * y1)
    }
  }
  // Diagonal
  else if x1 < x2 && y1 < y2 && x2 - x1 == y2 - y1 {
    while x1 < 7 && y1 < 7 {
      x1 += 1;
      y1 += 1;
      bb |= 1 << (x1 + 8 * y1);
    }
  } else if x2 < x1 && y2 < y1 && x1 - x2 == y1 - y2 {
    while x1 > 0 && y1 > 0 {
      x1 -= 1;
      y1 -= 1;
      bb |= 1 << (x1 + 8 * y1);
    }
  } else if x1 < x2 && y1 > y2 && x2 - x1 == y1 - y2 {
    while x1 < 7 && y1 > 0 {
      x1 += 1;
      y1 -= 1;
      bb |= 1 << (x1 + 8 * y1);
    }
  } else if x1 > x2 && y1 < y2 && x1 - x2 == y2 - y1 {
    while x1 > 0 && y1 < 7 {
      x1 -= 1;
      y1 += 1;
      bb |= 1 << (x1 + 8 * y1);
    }
  }

  Bitboard(bb)
}

// ---- Slider movegen ----
//
// Compile time methods to generate slider moves for a given square and
// set of blockers. To be used to build a table indexed either with magics, or
// directly with PEXT indices.

/// Get the movement mask for a bishop at a given square
pub const fn bishop_mask(square: Square) -> Bitboard {
  let mut bb: u64 = 0;

  // Up left
  let mut tgt = square as usize;
  while tgt % 8 > 1 && tgt / 8 < 6 {
    tgt += 7;
    bb |= 1 << tgt;
  }

  // Up right
  let mut tgt = square as usize;
  while tgt % 8 < 6 && tgt / 8 < 6 {
    tgt += 9;
    bb |= 1 << tgt;
  }

  // Down left
  let mut tgt = square as usize;
  while tgt % 8 > 1 && tgt / 8 >= 2 {
    tgt -= 9;
    bb |= 1 << tgt;
  }

  // Down right
  let mut tgt = square as usize;
  while tgt % 8 < 6 && tgt / 8 >= 2 {
    tgt -= 7;
    bb |= 1 << tgt;
  }

  Bitboard(bb as u64)
}

// Get the attacked squares for a bishop on a given square, with a given
// set of blockers
pub const fn gen_bishop_attacks(
  square: Square,
  blockers: Bitboard,
) -> Bitboard {
  let mut bb: u64 = 0;

  // Up left
  let mut tgt = square as usize;
  while tgt % 8 > 0 && tgt / 8 < 7 {
    tgt += 7;
    bb |= 1 << tgt;

    // If we've hit a piece, break
    if blockers.0 & (1 << tgt) > 0 {
      break;
    }
  }

  // Up right
  let mut tgt = square as usize;
  while tgt % 8 < 7 && tgt / 8 < 7 {
    tgt += 9;
    bb |= 1 << tgt;

    // If we've hit a piece, break
    if blockers.0 & (1 << tgt) > 0 {
      break;
    }
  }

  // Down left
  let mut tgt = square as usize;
  while tgt % 8 > 0 && tgt / 8 >= 1 {
    tgt -= 9;
    bb |= 1 << tgt;

    // If we've hit a piece, break
    if blockers.0 & (1 << tgt) > 0 {
      break;
    }
  }

  // Down right
  let mut tgt = square as usize;
  while tgt % 8 < 7 && tgt / 8 >= 1 {
    tgt -= 7;
    bb |= 1 << tgt;

    // If we've hit a piece, break
    if blockers.0 & (1 << tgt) > 0 {
      break;
    }
  }

  Bitboard(bb)
}

/// Get the movement mask for a rook at a given square
pub const fn rook_mask(square: Square) -> Bitboard {
  let file_bb = 0x001010101010100 << square.file();
  let rank_bb = 0x00000000000007e << square.rank() * 8;
  let square = 1 << square as u64;

  Bitboard((file_bb | rank_bb) & !square)
}

// Get the attacked squares for a rook on a given square, with a given
// set of blockers
pub const fn gen_rook_attacks(square: Square, blockers: Bitboard) -> Bitboard {
  let mut bb: u64 = 0;

  // Up
  let mut tgt = square as usize;
  while tgt / 8 < 7 {
    tgt += 8;
    bb |= 1 << tgt;

    // If we've hit a piece, break
    if blockers.0 & (1 << tgt) > 0 {
      break;
    }
  }

  // Right
  let mut tgt = square as usize;
  while tgt % 8 < 7 {
    tgt += 1;
    bb |= 1 << tgt;

    // If we've hit a piece, break
    if blockers.0 & (1 << tgt) > 0 {
      break;
    }
  }

  // Down
  let mut tgt = square as usize;
  while tgt / 8 >= 1 {
    tgt -= 8;
    bb |= 1 << tgt;

    // If we've hit a piece, break
    if blockers.0 & (1 << tgt) > 0 {
      break;
    }
  }

  // Left
  let mut tgt = square as usize;
  while tgt % 8 > 0 {
    tgt -= 1;
    bb |= 1 << tgt;

    // If we've hit a piece, break
    if blockers.0 & (1 << tgt) > 0 {
      break;
    }
  }

  Bitboard(bb)
}
