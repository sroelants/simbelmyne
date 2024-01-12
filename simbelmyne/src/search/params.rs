use crate::evaluate::Eval;

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
pub const NULL_MOVE_REDUCTION : usize = 3;

// Aspiration search
pub const ASPIRATION_MIN_DEPTH: usize = 4;
pub const ASPIRATION_BASE_WINDOW: Eval = 30;
pub const ASPIRATION_MAX_WINDOW: Eval = 900;

// Futility pruning
pub const FP_THRESHOLD: usize = 8;
pub const FP_MARGINS: [Eval; 9] = [0, 100, 160, 220, 280, 340, 400, 460, 520];

// Reverse futility pruning
pub const RFP_THRESHOLD: usize = 8;
pub const RFP_MARGIN: Eval = 80;

////////////////////////////////////////////////////////////////////////////////
//
// History Tables
//
////////////////////////////////////////////////////////////////////////////////

// Killer moves
pub const MAX_KILLERS: usize = 2;

// History table
pub const HIST_AGE_DIVISOR: i32 = 4;

// Late move pruning
pub const LMP_THRESHOLD: usize = 8;
pub const LMP_MOVE_THRESHOLDS: [usize; 9] = [0, 5, 8, 13, 20, 29, 40, 53, 68];


////////////////////////////////////////////////////////////////////////////////
//
// Late move reductions
//
////////////////////////////////////////////////////////////////////////////////

pub const LMR_MIN_DEPTH: usize = 3;
pub const LMR_THRESHOLD: usize = 3;

pub const LMR_MAX_MOVES: usize = 256;
pub const LMR_TABLE: [[usize; LMR_MAX_MOVES]; MAX_DEPTH + 1] = lmr_table();

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
