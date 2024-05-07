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

    // Aspiration windows
    pub aspiration_min_depth: usize,
    pub aspiration_base_window: Score,
    pub aspiration_max_window: Score,

    // Futility pruning
    pub fp_threshold: usize,
    pub fp_margins: [Score; 9],

    // Reverse futility pruning
    pub rfp_threshold: usize,
    pub rfp_margin: Score,

    // Late move pruning
    pub lmp_threshold: usize,
    pub lmp_move_thresholds: [usize; 9],

    // Late move reductions
    pub lmr_min_depth: usize,
    pub lmr_threshold: usize,

    pub delta_pruning_margin: Score,
}

impl Default for SearchParams {
    fn default() -> Self {
        Self {
            // Null move pruning
            nmp_base_reduction: NMP_BASE_REDUCTION,
            nmp_reduction_factor: NMP_REDUCTION_FACTOR,

            // Aspiration windows
            aspiration_min_depth: ASPIRATION_MIN_DEPTH,
            aspiration_base_window: ASPIRATION_BASE_WINDOW,
            aspiration_max_window: ASPIRATION_MAX_WINDOW,

            // Futility pruning
            fp_threshold: FP_THRESHOLD,
            fp_margins: FP_MARGINS,

            // Reverse futility pruning
            rfp_threshold: RFP_THRESHOLD,
            rfp_margin: RFP_MARGIN,

            // Late move pruning
            lmp_threshold: LMP_THRESHOLD,
            lmp_move_thresholds: LMP_MOVE_THRESHOLDS,

            // Late move reductions
            lmr_min_depth: LMR_MIN_DEPTH,
            lmr_threshold: LMR_THRESHOLD,

            delta_pruning_margin: DELTA_PRUNING_MARGIN,
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
pub const NMP_BASE_REDUCTION: usize = 3;
pub const NMP_REDUCTION_FACTOR: usize = 5;

// Aspiration search
pub const ASPIRATION_MIN_DEPTH: usize = 7;
pub const ASPIRATION_BASE_WINDOW: Score = 14;
pub const ASPIRATION_MAX_WINDOW: Score = 521;

// Futility pruning
pub const FP_THRESHOLD: usize = 8;
pub const FP_MARGINS: [Score; 9] = [0, 103, 160, 226, 276, 336, 402, 462, 520];

// Reverse futility pruning
pub const RFP_THRESHOLD: usize = 6;
pub const RFP_MARGIN: Score = 58;

////////////////////////////////////////////////////////////////////////////////
//
// History Tables
//
////////////////////////////////////////////////////////////////////////////////

// Killer moves
pub const MAX_KILLERS: usize = 2;

// History table
pub const HIST_AGE_DIVISOR: i16 = 4;

////////////////////////////////////////////////////////////////////////////////
//
// Late move pruning
//
////////////////////////////////////////////////////////////////////////////////

pub const LMP_THRESHOLD: usize = 7;
pub const LMP_MOVE_THRESHOLDS: [usize; 9] = [0, 1, 3, 11, 24, 40, 45, 59, 63];

////////////////////////////////////////////////////////////////////////////////
//
// Late move reductions
//
////////////////////////////////////////////////////////////////////////////////

pub const LMR_MIN_DEPTH: usize = 3;
pub const LMR_THRESHOLD: usize = 4;

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

pub const DELTA_PRUNING_MARGIN: Score = 160;
