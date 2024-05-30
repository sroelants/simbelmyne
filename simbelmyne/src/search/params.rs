use std::mem::transmute;

use crate::evaluate::Score;

////////////////////////////////////////////////////////////////////////////////
//
// SearchParams struct
//
// Enables the engine to dynamically set the values for the search parameters,
// for example through UCI options
// 
//
////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Copy)]
pub struct SearchParams {
    // Null move pruning
    pub nmp_base_reduction: usize,
    pub nmp_reduction_factor: usize,
    pub nmp_improving_margin: Score,

    // Aspiration windows
    pub aspiration_min_depth: usize,
    pub aspiration_base_window: Score,
    pub aspiration_max_window: Score,

    // Futility pruning
    pub fp_threshold: usize,
    pub fp_base: Score,
    pub fp_margin: Score,

    // Reverse futility pruning
    pub rfp_threshold: usize,
    pub rfp_margin: Score,

    // Late move pruning
    pub lmp_threshold: usize,
    pub lmp_base: usize,
    pub lmp_factor: usize,

    // Late move reductions
    pub lmr_min_depth: usize,
    pub lmr_threshold: usize,

    pub delta_pruning_margin: Score,

    // SEE pruning
    pub see_pruning_threshold: usize,
    pub see_capture_margin: Score,
    pub see_quiet_margin: Score,
}

impl Default for SearchParams {
    fn default() -> Self {
        Self {
            // Null move pruning
            nmp_base_reduction: NMP_BASE_REDUCTION,
            nmp_reduction_factor: NMP_REDUCTION_FACTOR,
            nmp_improving_margin: NMP_IMPROVING_MARGIN,

            // Aspiration windows
            aspiration_min_depth: ASPIRATION_MIN_DEPTH,
            aspiration_base_window: ASPIRATION_BASE_WINDOW,
            aspiration_max_window: ASPIRATION_MAX_WINDOW,

            // Futility pruning
            fp_threshold: FP_THRESHOLD,
            fp_base: FP_BASE,
            fp_margin: FP_MARGIN,

            // Reverse futility pruning
            rfp_threshold: RFP_THRESHOLD,
            rfp_margin: RFP_MARGIN,

            // Late move pruning
            lmp_threshold: LMP_THRESHOLD,
            lmp_base: LMP_BASE,
            lmp_factor: LMP_FACTOR,

            // Late move reductions
            lmr_min_depth: LMR_MIN_DEPTH,
            lmr_threshold: LMR_THRESHOLD,

            delta_pruning_margin: DELTA_PRUNING_MARGIN,

            // SEE pruning
            see_pruning_threshold: SEE_PRUNING_THRESHOLD,
            see_capture_margin: SEE_CAPTURE_MARGIN,
            see_quiet_margin: SEE_QUIET_MARGIN,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
//
// Search parameters
//
////////////////////////////////////////////////////////////////////////////////

pub const DEFAULT_TT_SIZE: usize = 64;
pub const MAX_DEPTH: usize = 128;

// Null-move pruning
pub const NMP_BASE_REDUCTION: usize = 4;
pub const NMP_REDUCTION_FACTOR: usize = 4;
pub const NMP_IMPROVING_MARGIN: Score = 70;

// Aspiration search
pub const ASPIRATION_MIN_DEPTH: usize = 7;
pub const ASPIRATION_BASE_WINDOW: Score = 19;
pub const ASPIRATION_MAX_WINDOW: Score = 724;

// Futility pruning
pub const FP_THRESHOLD: usize = 4;
pub const FP_BASE: i32 = 64;
pub const FP_MARGIN: i32 = 71;

// Reverse futility pruning
pub const RFP_THRESHOLD: usize = 9;
pub const RFP_MARGIN: Score = 47;

////////////////////////////////////////////////////////////////////////////////
//
// History Tables
//
////////////////////////////////////////////////////////////////////////////////

// Killer moves
pub const MAX_KILLERS: usize = 2;

// History table
pub const HIST_AGE_DIVISOR: i16 = 2;

////////////////////////////////////////////////////////////////////////////////
//
// Late move pruning
//
////////////////////////////////////////////////////////////////////////////////

pub const LMP_THRESHOLD: usize = 5;
pub const LMP_BASE: usize = 4;
pub const LMP_FACTOR: usize = 1;

////////////////////////////////////////////////////////////////////////////////
//
// Late move reductions
//
////////////////////////////////////////////////////////////////////////////////

pub const LMR_MIN_DEPTH: usize = 1;
pub const LMR_THRESHOLD: usize = 3;

const LMR_TABLE: [[usize; 64]; 64] = unsafe { transmute(*include_bytes!("../../../bins/lmr.bin")) };

pub fn lmr_reduction(depth: usize, move_count: usize) -> usize {
    LMR_TABLE[depth.min(63)][move_count.min(63)]
}

////////////////////////////////////////////////////////////////////////////////
//
// Internal Iterative Reduction
//
////////////////////////////////////////////////////////////////////////////////

pub const IIR_THRESHOLD: usize = 4;

////////////////////////////////////////////////////////////////////////////////
//
// Delta pruning
//
////////////////////////////////////////////////////////////////////////////////

pub const DELTA_PRUNING_MARGIN: Score = 125;

////////////////////////////////////////////////////////////////////////////////
///
// Static Exchange Evaluation pruning
//
////////////////////////////////////////////////////////////////////////////////

pub const SEE_PRUNING_THRESHOLD: usize = 9;
pub const SEE_QUIET_MARGIN: Score = -40;
pub const SEE_CAPTURE_MARGIN: Score = -60;
