pub mod bitboard;
pub mod board;
pub mod movegen;
pub mod square;
pub mod piece;
pub mod constants;
pub mod fen;

#[cfg(feature = "jemalloc")]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;
