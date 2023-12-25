use crate::evaluate::Eval;

// Search parameters
pub const MAX_DEPTH           : usize = 128;
pub const NULL_MOVE_REDUCTION : usize = 3;

// Aspiration search parameters
pub const ASPIRATION_MIN_DEPTH: usize = 4;
pub const ASPIRATION_BASE_WINDOW: Eval = 30;
pub const ASPIRATION_MAX_WINDOW: Eval = 900;

// Search options
pub const USE_TT           : bool = true;
pub const MOVE_ORDERING    : bool = true;
pub const TT_MOVE          : bool = true;
pub const MVV_LVA          : bool = true;
pub const KILLER_MOVES     : bool = true;
pub const HISTORY_TABLE    : bool = true;
pub const NULL_MOVE_PRUNING: bool = true;
pub const QUIESCENCE_SEARCH: bool = true;
pub const DEBUG            : bool = true;

