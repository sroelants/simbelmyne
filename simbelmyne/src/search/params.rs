// Search parameters
pub const MAX_DEPTH           : usize = 128;
pub const NULL_MOVE_REDUCTION : usize = 3;

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
