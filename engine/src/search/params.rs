use macros::tunable;
use std::mem::transmute;

pub use tunable_params::*;

pub const DEFAULT_TT_SIZE: usize = 64;
pub const MAX_DEPTH: usize = 128;
pub const MAX_KILLERS: usize = 2;

const LMR_TABLE: [[usize; 64]; 64] =
  unsafe { transmute(*include_bytes!("../../../bins/lmr.bin")) };

pub fn lmr_reduction(depth: usize, move_count: usize) -> usize {
  LMR_TABLE[depth.min(63)][move_count.min(63)]
}

/// This module holds all of the tunable search parameters
///
/// The #[tunable] macro converts every constant defined here into a getter
/// function (lower cased!). If the "spsa" compile feature is enabled,
/// the variables get replaced by corresponding Atomics, and UCI options are
/// generated.
///
/// Example:
/// `const FP_BASE: i32 = 64` is expanded into
///
/// ```rust
/// #[cfg(not(feature = "spsa"))]
/// const FP_BASE: i32 = 64;
///
/// #[cfg(not(feature = "spsa"))]
/// pub fn fp_base() -> i32 {
///   FP_BASE
/// }
///
/// #[cfg(feature = "spsa")]
/// const FP_BASE: AtomicI32 = AtomicI32::new(64);
///
/// #[cfg(feature = "spsa")]
/// pub fn fp_base() -> i32 {
///   FP_BASE.load(Ordering::Relaxed)
/// }
///
/// #[cfg(feature = "spsa")]
/// const SPSA_UCI_OPTIONS: [UciOption; 1] = [...]
///
/// #[cfg(feature = "spsa")]
/// pub fn set_param(name: &str, value: i32) { ... }
/// ```
#[tunable]
pub mod tunable_params {
  ////////////////////////////////////////////////////////////////////////////
  //
  // Null-move pruning
  //
  ////////////////////////////////////////////////////////////////////////////

  #[uci(min = 0, max = 8, step = 1)]
  const NMP_BASE_REDUCTION: usize = 4;

  #[uci(min = 0, max = 8, step = 1)]
  const NMP_REDUCTION_FACTOR: usize = 5;

  #[uci(min = 0, max = 100, step = 5)]
  const NMP_BASE_MARGIN: i32 = -122;

  #[uci(min = -200 , max = 0, step = 10)]
  const NMP_MARGIN_FACTOR: i32 = 24;

  #[uci(min = 0, max = 150, step = 10)]
  const NMP_IMPROVING_MARGIN: i32 = 67;

  ////////////////////////////////////////////////////////////////////////////
  //
  // Aspiration windows
  //
  ////////////////////////////////////////////////////////////////////////////

  #[uci(min = 1, max = 10, step = 1)]
  const ASPIRATION_MIN_DEPTH: usize = 8;

  #[uci(min = 10, max = 50, step = 10)]
  const ASPIRATION_BASE_WINDOW: i32 = 29;

  #[uci(min = 500, max = 1300, step = 50)]
  const ASPIRATION_MAX_WINDOW: i32 = 753;

  ////////////////////////////////////////////////////////////////////////////
  //
  // Futility pruning
  //
  ////////////////////////////////////////////////////////////////////////////

  #[uci(min = 1, max = 12, step = 1)]
  const FP_THRESHOLD: usize = 4;

  #[uci(min = 0, max = 150, step = 10)]
  const FP_BASE: i32 = 59;

  #[uci(min = 0, max = 150, step = 10)]
  const FP_MARGIN: i32 = 67;

  ////////////////////////////////////////////////////////////////////////////
  //
  // Reverse futility pruning
  //
  ////////////////////////////////////////////////////////////////////////////

  #[uci(min = 1, max = 12, step = 1)]
  const RFP_THRESHOLD: usize = 9;

  #[uci(min = 0, max = 150, step = 10)]
  const RFP_MARGIN: i32 = 57;

  #[uci(min = 0, max = 150, step = 10)]
  const RFP_IMPROVING_MARGIN: i32 = 99;

  ////////////////////////////////////////////////////////////////////////////
  //
  // Late move pruning
  //
  ////////////////////////////////////////////////////////////////////////////

  #[uci(min = 1, max = 12, step = 1)]
  const LMP_THRESHOLD: usize = 4;

  #[uci(min = 0, max = 10, step = 1)]
  const LMP_BASE: usize = 4;

  #[uci(min = 1, max = 5, step = 1)]
  const LMP_FACTOR: usize = 2;

  ////////////////////////////////////////////////////////////////////////////
  //
  // Late move reductions
  //
  ////////////////////////////////////////////////////////////////////////////

  #[uci(min = 1, max = 5, step = 1)]
  const LMR_MIN_DEPTH: usize = 1;

  #[uci(min = 1, max = 5, step = 1)]
  const LMR_THRESHOLD: usize = 2;

  ////////////////////////////////////////////////////////////////////////////
  //
  // Delta pruning
  //
  ////////////////////////////////////////////////////////////////////////////

  #[uci(min = 100, max = 250, step = 20)]
  const DELTA_PRUNING_MARGIN: i32 = 125;

  ////////////////////////////////////////////////////////////////////////////
  //
  // SEE pruning
  //
  ////////////////////////////////////////////////////////////////////////////

  #[uci(min = 0, max = 200, step = 10)]
  const SEE_QUIET_MARGIN: i32 = 34;

  #[uci(min = 0, max = 200, step = 10)]
  const SEE_TACTICAL_MARGIN: i32 = 93;

  ////////////////////////////////////////////////////////////////////////////
  //
  // History pruning
  //
  ////////////////////////////////////////////////////////////////////////////

  #[uci(min = 0, max = 10, step = 1)]
  const HP_THRESHOLD: usize = 4;

  #[uci(min = -4096, max = 0, step = 200)]
  const QUIET_HP_MARGIN: i32 = -1508;

  #[uci(min = -4096, max = 4096, step = 400)]
  const QUIET_HP_OFFSET: i32 = -1421;

  #[uci(min = -4096, max = 0, step = 200)]
  const TACTICAL_HP_MARGIN: i32 = -2418;

  #[uci(min = -4096, max = 4096, step = 400)]
  const TACTICAL_HP_OFFSET: i32 = -856;

  ////////////////////////////////////////////////////////////////////////////
  //
  // Corrhist contributions
  //
  ////////////////////////////////////////////////////////////////////////////

  #[uci(min = 0, max = 1024, step = 32)]
  const PAWN_CORR_WEIGHT: i32 = 221;

  #[uci(min = 0, max = 1024, step = 32)]
  const NONPAWN_CORR_WEIGHT: i32 = 132;

  #[uci(min = 0, max = 1024, step = 32)]
  const MATERIAL_CORR_WEIGHT: i32 = 997;

  #[uci(min = 0, max = 1024, step = 32)]
  const MINOR_CORR_WEIGHT: i32 = 222;

  #[uci(min = 0, max = 1024, step = 32)]
  const CONT_CORR_WEIGHT: i32 = 151;
  ////////////////////////////////////////////////////////////////////////////
  //
  // Singular extensions
  //
  ////////////////////////////////////////////////////////////////////////////

  #[uci(min = 1, max = 14, step = 1)]
  const SE_THRESHOLD: usize = 8;

  #[uci(min = 1, max = 4, step = 1)]
  const SE_MARGIN: i32 = 2;

  #[uci(min = 1, max = 6, step = 1)]
  const SE_TT_DELTA: usize = 3;

  #[uci(min = 0, max = 30, step = 5)]
  const DOUBLE_EXT_MARGIN: i32 = 21;

  #[uci(min = 0, max = 20, step = 2)]
  const DOUBLE_EXT_MAX: u8 = 7;

  #[uci(min = 0, max = 150, step = 20)]
  const TRIPLE_EXT_MARGIN: i32 = 126;

  ////////////////////////////////////////////////////////////////////////////
  //
  // Piece values
  //
  ////////////////////////////////////////////////////////////////////////////

  #[uci(min = 0, max = 1000, step = 20)]
  const PAWN_VALUE: i32 = 105;

  #[uci(min = 0, max = 1000, step = 20)]
  const KNIGHT_VALUE: i32 = 300;

  #[uci(min = 0, max = 1000, step = 20)]
  const BISHOP_VALUE: i32 = 300;

  #[uci(min = 0, max = 1000, step = 20)]
  const ROOK_VALUE: i32 = 502;

  #[uci(min = 0, max = 1200, step = 20)]
  const QUEEN_VALUE: i32 = 876;

  ////////////////////////////////////////////////////////////////////////////
  //
  // Internal iterative reduction
  //
  ////////////////////////////////////////////////////////////////////////////

  #[uci(min = 0, max = 8, step = 1)]
  const IIR_THRESHOLD: usize = 5;

  #[uci(min = 0, max = 4, step = 1)]
  const IIR_REDUCTION: usize = 1;

  ////////////////////////////////////////////////////////////////////////////
  //
  // Quiet/capture history
  //
  ////////////////////////////////////////////////////////////////////////////

  #[uci(min = 0, max = 16, step = 1)]
  const HIST_BONUS_CONST_CUTOFF: usize = 13;

  #[uci(min = 0, max = 100, step = 10)]
  const HIST_BONUS_CONST: i16 = 39;

  #[uci(min = 0, max = 200, step = 20)]
  const HIST_BONUS_LINEAR: i16 = 149;

  #[uci(min = 0, max = 100, step = 10)]
  const HIST_BONUS_QUADRATIC: i16 = 18;

  #[uci(min = 1, max = 16382, step = 100)]
  const HIST_LMR_DIVISOR: i32 = 8242;

  ////////////////////////////////////////////////////////////////////////////
  //
  // Time management
  //
  ////////////////////////////////////////////////////////////////////////////

  #[uci(min = 1, max = 100, step = 5)]
  const INC_FRAC: u32 = 72;

  #[uci(min = 1, max = 100, step = 5)]
  const LIMIT_TIME_FRAC: u32 = 77;

  #[uci(min = 1, max = 1000, step = 5)]
  const BASE_TIME_FRAC: u32 = 58;

  #[uci(min = 1, max = 100, step = 5)]
  const SOFT_TIME_FRAC: u32 = 82;

  #[uci(min = 1, max = 100, step = 5)]
  const HARD_TIME_FRAC: u32 = 301;

  #[uci(min = 1, max = 200, step = 10)]
  const NODE_FRAC_BASE: u32 = 156;

  #[uci(min = 1, max = 200, step = 10)]
  const NODE_FRAC_MULT: u32 = 173;
}
