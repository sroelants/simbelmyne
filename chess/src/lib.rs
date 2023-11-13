pub mod bitboard;
pub mod board;
pub mod movegen;
pub mod util;
pub mod square;
pub mod piece;

#[cfg(feature = "jemalloc")]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;
