use macros::tunable;

pub use tunable_params::*;

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
  const NMP_BASE_REDUCTION: i32 = 4;

  #[uci(min = 0, max = 8, step = 1)]
  const NMP_REDUCTION_FACTOR: i32 = 4;

  #[uci(min = -200, max = 200, step = 20)]
  const NMP_BASE_MARGIN: i32 = -120;

  #[uci(min = -200 , max = 200, step = 20)]
  const NMP_MARGIN_FACTOR: i32 = 20;

  #[uci(min = 0, max = 150, step = 10)]
  const NMP_IMPROVING_MARGIN: i32 = 70;

  ////////////////////////////////////////////////////////////////////////////
  //
  // Aspiration windows
  //
  ////////////////////////////////////////////////////////////////////////////

  #[uci(min = 1, max = 10, step = 1)]
  const ASPIRATION_MIN_DEPTH: i32 = 7;

  #[uci(min = 10, max = 50, step = 10)]
  const ASPIRATION_BASE_WINDOW: i32 = 19;

  #[uci(min = 500, max = 1300, step = 50)]
  const ASPIRATION_MAX_WINDOW: i32 = 724;

  ////////////////////////////////////////////////////////////////////////////
  //
  // Futility pruning
  //
  ////////////////////////////////////////////////////////////////////////////

  #[uci(min = 1, max = 12, step = 1)]
  const FP_THRESHOLD: i32 = 4;

  #[uci(min = 0, max = 150, step = 10)]
  const FP_BASE: i32 = 64;

  #[uci(min = 0, max = 150, step = 10)]
  const FP_MARGIN: i32 = 71;

  ////////////////////////////////////////////////////////////////////////////
  //
  // Reverse futility pruning
  //
  ////////////////////////////////////////////////////////////////////////////

  #[uci(min = 1, max = 12, step = 1)]
  const RFP_THRESHOLD: i32 = 9;

  #[uci(min = 0, max = 150, step = 10)]
  const RFP_MARGIN: i32 = 47;

  #[uci(min = 0, max = 150, step = 10)]
  const RFP_IMPROVING_MARGIN: i32 = 100;

  ////////////////////////////////////////////////////////////////////////////
  //
  // Late move pruning
  //
  ////////////////////////////////////////////////////////////////////////////

  #[uci(min = 1, max = 12, step = 1)]
  const LMP_THRESHOLD: i32 = 5;

  #[uci(min = 0, max = 10, step = 1)]
  const LMP_BASE: i32 = 4;

  #[uci(min = 1, max = 5, step = 1)]
  const LMP_FACTOR: i32 = 1;

  ////////////////////////////////////////////////////////////////////////////
  //
  // Late move reductions
  //
  ////////////////////////////////////////////////////////////////////////////

  #[uci(min = 1, max = 5, step = 1)]
  const LMR_MIN_DEPTH: i32 = 1;

  #[uci(min = 1, max = 5, step = 1)]
  const LMR_THRESHOLD: i32 = 3;

  #[uci(min = 512, max = 1024, step = 20)]
  const LMR_BASE: u32 = 764;

  #[uci(min = 128, max = 512, step = 5)]
  const LMR_FACTOR: u32 = 219;

  #[uci(min = 0, max = 2048, step = 128)]
  const QUIET_LMR: i32 = 1045;

  #[uci(min = 0, max = 2048, step = 128)]
  const BAD_TACT_LMR: i32 = 989;

  #[uci(min = 0, max = 2048, step = 128)]
  const TT_TACT_LMR: i32 = 1045;

  #[uci(min = 0, max = 4096, step = 256)]
  const CUTNODE_LMR: i32 = 2149;

  #[uci(min = 0, max = 2048, step = 128)]
  const TTPV_LMR: i32 = 1061;

  #[uci(min = 0, max = 2048, step = 128)]
  const IN_CHECK_LMR: i32 = 1016;

  #[uci(min = 0, max = 2048, step = 128)]
  const GIVES_CHECK_LMR: i32 = 1056;

  #[uci(min = 0, max = 2048, step = 128)]
  const FAILHIGH_COUNT_LMR: i32 = 1084;

  #[uci(min = 0, max = 2048, step = 128)]
  const TT_FAILLOW_LMR: i32 = 990;

  #[uci(min = 0, max = 2048, step = 128)]
  const HISTORY_LMR: i32 = 1037;

  #[uci(min = 0, max = 60, step = 3)]
  const DEEPER_BASE: i32 = 20;

  #[uci(min = 0, max = 8, step = 1)]
  const DEEPER_FACTOR: i32 = 2;

  #[uci(min = -20, max = 20, step = 2)]
  const SHALLOWER_BASE: i32 = 0;

  #[uci(min = 0, max = 8, step = 1)]
  const SHALLOWER_FACTOR: i32 = 1;

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
  const SEE_QUIET_MARGIN: i32 = 40;

  #[uci(min = 0, max = 200, step = 10)]
  const SEE_TACTICAL_MARGIN: i32 = 100;

  ////////////////////////////////////////////////////////////////////////////
  //
  // History pruning
  //
  ////////////////////////////////////////////////////////////////////////////

  #[uci(min = 0, max = 10, step = 1)]
  const HP_THRESHOLD: i32 = 5;

  #[uci(min = -4096, max = 0, step = 200)]
  const QUIET_HP_MARGIN: i32 = -1500;

  #[uci(min = -4096, max = 4096, step = 400)]
  const QUIET_HP_OFFSET: i32 = -1000;

  #[uci(min = -4096, max = 0, step = 200)]
  const TACTICAL_HP_MARGIN: i32 = -2500;

  #[uci(min = -4096, max = 4096, step = 400)]
  const TACTICAL_HP_OFFSET: i32 = -1000;

  ////////////////////////////////////////////////////////////////////////////
  //
  // Corrhist contributions
  //
  ////////////////////////////////////////////////////////////////////////////

  #[uci(min = 0, max = 1024, step = 32)]
  const PAWN_CORR_WEIGHT: i32 = 256;

  #[uci(min = 0, max = 1024, step = 32)]
  const NONPAWN_CORR_WEIGHT: i32 = 128;

  #[uci(min = 0, max = 1024, step = 32)]
  const MATERIAL_CORR_WEIGHT: i32 = 1024;

  #[uci(min = 0, max = 1024, step = 32)]
  const MINOR_CORR_WEIGHT: i32 = 256;

  #[uci(min = 0, max = 1024, step = 32)]
  const CONT_CORR_WEIGHT: i32 = 128;

  ////////////////////////////////////////////////////////////////////////////
  //
  // Singular extensions
  //
  ////////////////////////////////////////////////////////////////////////////

  #[uci(min = 1, max = 14, step = 1)]
  const SE_THRESHOLD: i32 = 8;

  #[uci(min = 1, max = 4, step = 1)]
  const SE_MARGIN: i32 = 2;

  #[uci(min = 1, max = 6, step = 1)]
  const SE_TT_DELTA: i32 = 3;

  #[uci(min = 0, max = 30, step = 5)]
  const DOUBLE_EXT_MARGIN: i32 = 17;

  #[uci(min = 0, max = 20, step = 2)]
  const DOUBLE_EXT_MAX: u8 = 8;

  #[uci(min = 0, max = 150, step = 20)]
  const TRIPLE_EXT_MARGIN: i32 = 100;

  ////////////////////////////////////////////////////////////////////////////
  //
  // Piece values
  //
  ////////////////////////////////////////////////////////////////////////////

  #[uci(min = 0, max = 1000, step = 20)]
  const PAWN_VALUE: i32 = 100;

  #[uci(min = 0, max = 1000, step = 20)]
  const KNIGHT_VALUE: i32 = 300;

  #[uci(min = 0, max = 1000, step = 20)]
  const BISHOP_VALUE: i32 = 300;

  #[uci(min = 0, max = 1000, step = 20)]
  const ROOK_VALUE: i32 = 500;

  #[uci(min = 0, max = 1200, step = 20)]
  const QUEEN_VALUE: i32 = 900;

  ////////////////////////////////////////////////////////////////////////////
  //
  // Internal iterative reduction
  //
  ////////////////////////////////////////////////////////////////////////////

  #[uci(min = 0, max = 8, step = 1)]
  const IIR_THRESHOLD: i32 = 4;

  #[uci(min = 0, max = 4, step = 1)]
  const IIR_REDUCTION: i32 = 1;

  ////////////////////////////////////////////////////////////////////////////
  //
  // Quiet/capture history
  //
  ////////////////////////////////////////////////////////////////////////////

  #[uci(min = 0, max = 16, step = 1)]
  const HIST_BONUS_CONST_CUTOFF: i32 = 13;

  #[uci(min = 0, max = 100, step = 10)]
  const HIST_BONUS_CONST: i32 = 32;

  #[uci(min = 0, max = 200, step = 20)]
  const HIST_BONUS_LINEAR: i32 = 128;

  #[uci(min = 0, max = 100, step = 10)]
  const HIST_BONUS_QUADRATIC: i32 = 16;

  #[uci(min = 1, max = 16382, step = 100)]
  const HIST_LMR_DIVISOR: i32 = 8191;

  ////////////////////////////////////////////////////////////////////////////
  //
  // Time management
  //
  ////////////////////////////////////////////////////////////////////////////

  #[uci(min = 1, max = 128, step = 6)]
  const INC_FRAC: u32 = 75;

  #[uci(min = 1, max = 128, step = 6)]
  const LIMIT_TIME_FRAC: u32 = 76;

  #[uci(min = 1, max = 1024, step = 50)]
  const BASE_TIME_FRAC: u32 = 54;

  #[uci(min = 1, max = 128, step = 6)]
  const SOFT_TIME_FRAC: u32 = 76;

  #[uci(min = 1, max = 512, step = 25)]
  const HARD_TIME_FRAC: u32 = 304;

  #[uci(min = 1, max = 256, step = 12)]
  const NODE_FRAC_BASE: u32 = 152;

  #[uci(min = 1, max = 256, step = 12)]
  const NODE_FRAC_MULT: u32 = 174;
}

pub const DEFAULT_TT_SIZE: usize = 64;
pub const MAX_DEPTH: usize = 128;
pub const MAX_KILLERS: usize = 2;

#[inline(always)]
pub fn lmr_reduction(depth: i32, move_count: i32) -> i32 {
  if move_count == 0 {
    return 0;
  }

  (lmr_base() + lmr_factor() * depth.ilog2() * move_count.ilog2()) as i32
}
