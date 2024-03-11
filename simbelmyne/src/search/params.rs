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
    pub lmr_max_moves: usize,
    pub lmr_table: [[usize; LMR_MAX_MOVES]; MAX_DEPTH + 1],
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
            lmr_max_moves: LMR_MAX_MOVES,
            lmr_table: LMR_TABLE
        }
    }
}


////////////////////////////////////////////////////////////////////////////////
//
// Search options
//
////////////////////////////////////////////////////////////////////////////////

pub const USE_TT           : bool = true;
pub const MOVE_ORDERING    : bool = true;
pub const TT_MOVE          : bool = true;
pub const MVV_LVA          : bool = true;
pub const KILLER_MOVES     : bool = true;
pub const HISTORY_TABLE    : bool = true;
pub const NULL_MOVE_PRUNING: bool = true;
pub const QUIESCENCE_SEARCH: bool = true;
pub const SEE_ORDERING     : bool = true;
pub const DEBUG            : bool = true;

////////////////////////////////////////////////////////////////////////////////
//
// Search parameters
//
////////////////////////////////////////////////////////////////////////////////

pub const MAX_DEPTH           : usize = 128;

// Null-move pruning
const NMP_BASE_REDUCTION: usize = 4;
const NMP_REDUCTION_FACTOR: usize = 4;

// Aspiration search
const ASPIRATION_MIN_DEPTH: usize = 4;
const ASPIRATION_BASE_WINDOW: Score = 30;
const ASPIRATION_MAX_WINDOW: Score = 900;

// Futility pruning
const FP_THRESHOLD: usize = 8;
const FP_MARGINS: [Score; 9] = [0, 100, 160, 220, 280, 340, 400, 460, 520];

// Reverse futility pruning
const RFP_THRESHOLD: usize = 8;
const RFP_MARGIN: Score = 80;

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

const LMP_THRESHOLD: usize = 8;
const LMP_MOVE_THRESHOLDS: [usize; 9] = [0, 5, 8, 13, 20, 29, 40, 53, 68];


////////////////////////////////////////////////////////////////////////////////
//
// Late move reductions
//
////////////////////////////////////////////////////////////////////////////////

const LMR_MIN_DEPTH: usize = 3;
const LMR_THRESHOLD: usize = 3;

const LMR_MAX_MOVES: usize = 256;
const LMR_TABLE: [[usize; LMR_MAX_MOVES]; MAX_DEPTH + 1] = lmr_table();

const fn lmr_table() -> [[usize; LMR_MAX_MOVES]; MAX_DEPTH + 1] {
    let mut lmr_table = [[0; LMR_MAX_MOVES]; MAX_DEPTH + 1];
    let mut depth = 0;
    let mut move_count = 0;

    while depth < MAX_DEPTH + 1 {
        while move_count < LMR_MAX_MOVES {
            lmr_table[depth][move_count] = 
                move_count / 12 + if depth < 8 { 2 } else { depth / 4 };

            move_count += 1;
        }
        depth += 1;
    }

    lmr_table
}
