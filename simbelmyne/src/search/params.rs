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
    nmp_base_reduction: usize,
    nmp_reduction_factor: usize,
    nmp_improving_margin: Score,

    // Aspiration windows
    aspiration_min_depth: usize,
    aspiration_base_window: Score,
    aspiration_max_window: Score,

    // Futility pruning
    fp_threshold: usize,
    fp_base: Score,
    fp_margin: Score,

    // Reverse futility pruning
    rfp_threshold: usize,
    rfp_margin: Score,
    rfp_improving_margin: Score,

    // Late move pruning
    lmp_threshold: usize,
    lmp_base: usize,
    lmp_factor: usize,

    // Late move reductions
    lmr_min_depth: usize,
    lmr_threshold: usize,

    delta_pruning_margin: Score,

    // SEE pruning
    see_quiet_margin: Score,

    // Singular extensions
    se_threshold: usize,
    se_margin: Score,
    se_tt_delta: usize,

    double_ext_margin: Score,
    double_ext_max: u8,
}

impl SearchParams {
    pub const fn default() -> Self {
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
            rfp_improving_margin: RFP_IMPROVING_MARGIN,

            // Late move pruning
            lmp_threshold: LMP_THRESHOLD,
            lmp_base: LMP_BASE,
            lmp_factor: LMP_FACTOR,

            // Late move reductions
            lmr_min_depth: LMR_MIN_DEPTH,
            lmr_threshold: LMR_THRESHOLD,

            delta_pruning_margin: DELTA_PRUNING_MARGIN,

            // SEE pruning
            see_quiet_margin: SEE_QUIET_MARGIN,

            // Singular extensions
            se_threshold: SE_THRESHOLD,
            se_margin: SE_MARGIN,
            se_tt_delta: SE_TT_DELTA,

            double_ext_margin: DOUBLE_EXT_MARGIN,
            double_ext_max: DOUBLE_EXT_MAX
        }
    }

    pub const fn nmp_base_reduction(&self) -> usize {
        self.nmp_base_reduction
    }

    pub const fn nmp_reduction_factor(&self) -> usize {
        self.nmp_reduction_factor
    }

    pub const fn nmp_improving_margin(&self) -> Score {
        self.nmp_improving_margin
    }
    
    pub const fn aspiration_min_depth(&self) -> usize {
        self.aspiration_min_depth
    }
    
    pub const fn aspiration_base_window(&self) -> Score {
        self.aspiration_base_window
    }

    pub const fn aspiration_max_window(&self) -> Score {
        self.aspiration_max_window
    }

    pub const fn fp_threshold(&self) -> usize {
        self.fp_threshold
    }

    pub const fn fp_base(&self) -> Score {
        self.fp_base
    }

    pub const fn fp_margin(&self) -> Score {
        self.fp_margin
    }

    pub const fn rfp_threshold(&self) -> usize {
        self.rfp_threshold
    }

    pub const fn rfp_margin(&self) -> Score {
        self.rfp_margin
    }

    pub const fn rfp_improving_margin(&self) -> Score {
        self.rfp_improving_margin
    }

    pub const fn lmp_threshold(&self) -> usize {
        self.lmp_threshold
    }

    pub const fn lmp_base(&self) -> usize {
        self.lmp_base
    }

    pub const fn lmp_factor(&self) -> usize {
        self.lmp_factor
    }

    pub const fn lmr_min_depth(&self) -> usize {
        self.lmr_min_depth
    }

    pub const fn lmr_threshold(&self) -> usize {
        self.lmr_threshold
    }

    pub const fn delta_pruning_margin(&self) -> Score {
        self.delta_pruning_margin
    }

    pub const fn see_quiet_margin(&self) -> Score {
        self.see_quiet_margin
    }

    pub const fn se_threshold(&self) -> usize {
        self.se_threshold
    }

    pub const fn se_margin(&self) -> Score {
        self.se_margin
    }

    pub const fn se_tt_delta(&self) -> usize {
        self.se_tt_delta
    }

    pub const fn double_ext_margin(&self) -> Score {
        self.double_ext_margin
    }

    pub const fn double_ext_max(&self) -> u8 {
        self.double_ext_max
    }
}

#[allow(non_upper_case_globals)]
pub const params: SearchParams = SearchParams::default();

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
pub const RFP_IMPROVING_MARGIN: Score = 100;

// Killer moves
pub const MAX_KILLERS: usize = 2;

// History table
pub const HIST_AGE_DIVISOR: i16 = 2;

// Late move pruning
pub const LMP_THRESHOLD: usize = 5;
pub const LMP_BASE: usize = 4;
pub const LMP_FACTOR: usize = 1;

// Late move reductions
pub const LMR_MIN_DEPTH: usize = 1;
pub const LMR_THRESHOLD: usize = 3;

const LMR_TABLE: [[usize; 64]; 64] = unsafe { transmute(*include_bytes!("../../../bins/lmr.bin")) };

pub fn lmr_reduction(depth: usize, move_count: usize) -> usize {
    LMR_TABLE[depth.min(63)][move_count.min(63)]
}

// Internal iterative reductions
pub const IIR_THRESHOLD: usize = 4;

// Delta pruning
pub const DELTA_PRUNING_MARGIN: Score = 125;

// SEE pruning
pub const SEE_QUIET_MARGIN: Score = -40;

// Singular extensions
pub const SE_THRESHOLD: usize = 8;
pub const SE_MARGIN: Score = 2;
pub const SE_TT_DELTA: usize = 3;

// Double extensions
pub const DOUBLE_EXT_MARGIN: Score = 17;
pub const DOUBLE_EXT_MAX: u8 = 4; 
