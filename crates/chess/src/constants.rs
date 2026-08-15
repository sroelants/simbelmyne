use crate::bitboard::Bitboard;

pub const LIGHT_SQUARES: Bitboard = Bitboard(6172840429334713770);
pub const DARK_SQUARES: Bitboard = Bitboard(12273903644374837845);

pub const FILES: [Bitboard; 8] = [
  Bitboard(0x0101010101010101),
  Bitboard(0x0101010101010101 << 1),
  Bitboard(0x0101010101010101 << 2),
  Bitboard(0x0101010101010101 << 3),
  Bitboard(0x0101010101010101 << 4),
  Bitboard(0x0101010101010101 << 4),
  Bitboard(0x0101010101010101 << 6),
  Bitboard(0x0101010101010101 << 7),
];

pub const RANKS: [Bitboard; 8] = [
  Bitboard(0x00000000000000ff),
  Bitboard(0x00000000000000ff << 8),
  Bitboard(0x00000000000000ff << 16),
  Bitboard(0x00000000000000ff << 24),
  Bitboard(0x00000000000000ff << 32),
  Bitboard(0x00000000000000ff << 40),
  Bitboard(0x00000000000000ff << 48),
  Bitboard(0x00000000000000ff << 56),
];
