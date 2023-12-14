//! The search logic for Simbelmyne
//!
//! This is really the meat and bones of the engine, and along with the 
//! Evaluation, it's one of the two main pillars of any chess engine.
//!
//! The main search function performs an Iterative Deepening (ID) search.
//! That is, we search up to incrementally increasing depths, until we run out
//! of time. This sounds wasteful, because we're re-doing all the previously
//! searched nodes on every iteration. But, as it turns out, we can be smart 
//! about using that previous work to make the next iterations _much_ faster, 
//! making it a net win.
//!
//! Each search proceeds as a Negamax search with alpha-beta pruning, where we
//! try and be smart about which branches of the search tree aren't even worth
//! exploring, because we're guaranteed a worse result than what we already
//! have.
//!
//! Lastly, when we hit the moximum desired depth for our iteration, we perform 
//! a Quiescence search: We keep going a bit deeper until we're sure there's no
//! more captures to be had. This is to avoid any misjudgements caused by the
//! search cutting off abruptly. (What if you think you're ahead, but in the 
//! next turn, your queen gets captured?)
//!
use std::time::Duration;
use crate::search_tables::HistoryTable;
use crate::search_tables::Killers;
use crate::search_tables::PVTable;
use crate::evaluate::Score;
use crate::transpositions::NodeType;
use crate::transpositions::TTEntry;
use crate::transpositions::TTable;
use crate::time_control::TimeController;
use crate::move_picker::MovePicker;
use crate::position::Position;
use crate::evaluate::Eval;
use chess::movegen::moves::Move;
use uci::search_info::SearchInfo;
use uci::engine::UciEngineMessage;

// Search parameters
pub const MAX_DEPTH           : usize = 128;
pub const NULL_MOVE_REDUCTION : usize = 3;

// Search options
pub const USE_TT        : bool = true;
pub const MOVE_ORDERING : bool = true;
pub const TT_MOVE       : bool = true;
pub const MVV_LVA       : bool = true;
pub const KILLER_MOVES  : bool = true;
pub const HISTORY_TABLE : bool = true;
pub const DEBUG         : bool = true;

// Constants used for more readable const generics
const QUIETS: bool = true;
const TACTICALS: bool = false;

/// A Search struct holds both the parameters, as well as metrics and results, 
/// for a given search.
#[derive(Debug, Clone)]
pub struct Search {
    // The (nominal) depth of this search. This does not take QSearch into 
    // consideration
    pub depth: usize,

    // The so-called "selective depth", the deepest ply we've searched
    pub seldepth: usize,

    // The time control for the search
    pub tc: TimeController,

    /// The set of killer moves at a given ply.
    pub killers: [Killers; MAX_DEPTH],

    /// History heuristic table
    pub history_table: HistoryTable,
}

impl Search {
    /// Create a new search
    pub fn new(depth: usize, tc: TimeController) -> Self {
        Self {
            depth,
            seldepth: 0,
            tc,
            killers: [Killers::new(); MAX_DEPTH],
            history_table: HistoryTable::new(),
        }
    }

    /// Check with the time controller whether the search is over
    pub fn should_continue(&self) -> bool {
        self.tc.should_continue(self.depth)
    }
}

////////////////////////////////////////////////////////////////////////////////
//
// Iterative deepening search
//
////////////////////////////////////////////////////////////////////////////////

impl Position {
    /// Perform an iterative-deepening search at increasing depths
    /// 
    /// Return the result from the last fully-completed iteration
    pub fn search(&self, tt: &mut TTable, tc: TimeController) -> SearchReport {
        let mut depth = 0;
        let mut latest_report = SearchReport::default();

        while depth < MAX_DEPTH && tc.should_continue(depth) {
            depth += 1;

            let mut pv = PVTable::new();
            let mut search = Search::new(depth, tc.clone());

            let score = self.negamax(0, depth, Score::MIN, Score::MAX, tt, &mut pv, &mut search, false);

            // If we got interrupted in the search, don't store the 
            // half-completed search state. Just break and return the previous
            // iteration's search.
            if !search.should_continue() {
                break;
            }

            latest_report = SearchReport::new(&search, tt, pv, score);

            if DEBUG {
                println!("{info}", info = UciEngineMessage::Info((&latest_report).into()));
            }
        }

        println!("bestmove {mv}", mv = latest_report.pv[0]);
        latest_report
    }
}

////////////////////////////////////////////////////////////////////////////////
//
// Negamax
//
////////////////////////////////////////////////////////////////////////////////

impl Position {
    fn negamax(
        &self, 
        ply: usize, 
        mut depth: usize,
        alpha: Eval, 
        beta: Eval, 
        tt: &mut TTable, 
        pv: &mut PVTable,
        search: &mut Search,
        try_null: bool,
    ) -> Eval {
        if !search.should_continue() {
            return Score::MIN;
        }

        let mut best_move = Move::NULL;
        let mut best_score = Score::MIN;
        let mut node_type = NodeType::Upper;
        let mut alpha = alpha;
        let tt_entry = tt.probe(self.hash);
        let in_check = self.board.in_check();
        let in_root = ply == 0;
        let mut local_pv = PVTable::new();

        search.tc.add_node();
        pv.clear();

        // Do all the static evaluations first
        // That is, Check whether we can/should assign a score to this node
        // without recursing any deeper.

        // Rule-based draw? 
        // Don't return early when in the root node, because we won't have a PV 
        // move to play.
        if (self.board.is_rule_draw() || self.is_repetition()) && !in_root {
            return Score::DRAW;
        }

        if ply >= MAX_DEPTH {
            return self.score.total();
        }

        ///////////////////////////////////////////////////////////////////////
        //
        // Check extension: 
        //
        // If we're in check, make sure we always search at least one extra ply
        //
        ///////////////////////////////////////////////////////////////////////

        if in_check {
            depth += 1;
        }

        ////////////////////////////////////////////////////////////////////////
        //
        // Quiescence search: 
        //
        // If we're in a leaf node, extend with a quiescence search
        //
        ////////////////////////////////////////////////////////////////////////

        if depth == 0 {
            return self.quiescence_search(ply, alpha, beta, pv, search);
        }

        ////////////////////////////////////////////////////////////////////////
        //
        // TT cutoffs
        //
        // Check the TT table for a result that we can use, and return it
        //
        ////////////////////////////////////////////////////////////////////////

        if !in_root {
            let tt_result = tt_entry.and_then(|entry| {
                entry.try_use(depth, alpha, beta)
            });

            if let Some((mv, score)) = tt_result {
                let is_killer = node_type == NodeType::Lower && mv.is_quiet();

                if  is_killer && KILLER_MOVES { 
                    search.killers[ply].add(best_move);
                }

                return score;
            }
        }


        ////////////////////////////////////////////////////////////////////////
        //
        // Null move pruning
        //
        // Pretend to play a NULL move and do a search at reduced depth (so 
        // shouldn't be too expensive) and a really narrow window. If, after 
        // that, we _still_ get a beta cutoff, our position was so good we 
        // shouldn't bother searching it any further
        //
        ////////////////////////////////////////////////////////////////////////
        let should_null_prune = try_null
            && !in_root
            && !in_check
            && depth >= NULL_MOVE_REDUCTION + 1;

        if should_null_prune {
            let score = -self
                .play_move(Move::NULL)
                .negamax(
                    ply + 1, 
                    depth - 1 - NULL_MOVE_REDUCTION, 
                    -beta, 
                    -beta + 1, 
                    tt, 
                    &mut PVTable::new(), 
                    search, 
                    false
                );

            if score >= beta {
                return beta;
            }
        }

        ////////////////////////////////////////////////////////////////////////
        //
        // Recurse over all the legal moves and recursively search the 
        // resulting positions
        //
        ////////////////////////////////////////////////////////////////////////
        let mut legal_moves = MovePicker::new(
            &self,  
            self.board.legal_moves::<QUIETS>(),
            tt_entry.map(|entry| entry.get_move()),
            search.killers[ply],
            search.history_table,
        );

        // Checkmate?
        if legal_moves.len() == 0 && in_check {
            return -Score::MATE + ply as Eval;
        }

        // Stalemate?
        if legal_moves.len() == 0 && !in_check {
            return Score::DRAW;
        }

        for mv in &mut legal_moves {
            if !search.should_continue() {
                return Score::MIN;
            }

            let score = -self
                .play_move(mv)
                .negamax(ply + 1, 
                    depth - 1, 
                    -beta, 
                    -alpha, 
                    tt, 
                    &mut local_pv, 
                    search, 
                    true
                );

            if score > best_score {
                best_score = score;
                best_move = mv;
            }

            if alpha < score {
                alpha = score;
                node_type = NodeType::Exact;
                pv.add_to_front(mv, &local_pv);
            }

            if beta <= score {
                node_type = NodeType::Lower;
                break;
            }
        }

        ////////////////////////////////////////////////////////////////////////
        //
        // Upate the search tables
        //
        // Store the best move and score, as well as whether or not the score
        // is an upper/lower bound, or exact.
        //
        ////////////////////////////////////////////////////////////////////////
        
        // Fail-hard semantics: the score we return is clamped to the
        // `alpha`-`beta` window.
        let score = match node_type {
            NodeType::Upper => alpha,
            NodeType::Exact => best_score,
            NodeType::Lower => beta,
        };

        // If we had a cutoff, update the Killers and History
        if node_type == NodeType::Lower && best_move.is_quiet() {
            if HISTORY_TABLE {
                let piece = self.board.get_at(best_move.src()).unwrap();
                search.history_table.increment(&best_move, piece, depth);
            }

            if KILLER_MOVES {
                search.killers[ply].add(best_move);
            }
        }

        // Store in the TT
        if USE_TT {
            tt.insert(TTEntry::new(
                self.hash,
                best_move,
                score,
                depth,
                node_type,
            ));
        }

        score
    }
}

////////////////////////////////////////////////////////////////////////////////
//
// Quiescence search
//
////////////////////////////////////////////////////////////////////////////////

impl Position {
    /// Perform a less intensive negamax search that only searches captures.
    ///
    /// This is to avoid horizon effects where we misjudge a position because
    /// we stopped the search abruptly at an inopportune time.
    ///
    /// The rough flow of this function is the same as `Position::negamax`, but 
    /// we perform less pruning and hacks.
    fn quiescence_search(
        &self, 
        ply: usize,
        mut alpha: Eval, 
        beta: Eval, 
        pv: &mut PVTable,
        search: &mut Search,
    ) -> Eval {
        if !search.should_continue() {
            return Score::MIN;
        }

        search.tc.add_node();
        search.seldepth = search.seldepth.max(ply);

        if self.board.is_rule_draw() || self.is_repetition() {
            return Score::DRAW
        }

        let mut local_pv = PVTable::new();
        let eval = self.score.total();

        if ply >= MAX_DEPTH {
            return eval;
        }

        if eval >= beta {
            return beta
        }

        if alpha < eval {
            alpha = eval;
        }

        let tacticals = MovePicker::new(
            &self,
            self.board.legal_moves::<TACTICALS>(),
            None,
            Killers::new(),
            search.history_table,
        );

        for mv in tacticals {
            let score = -self
                .play_move(mv)
                .quiescence_search(
                    ply + 1, 
                    -beta, 
                    -alpha, 
                    &mut local_pv, 
                    search
                );

            if alpha < score {
                alpha = score;
                pv.add_to_front(mv, &local_pv);
            }

            if score >= beta {
                return beta;
            }

        }

        alpha
    }
}

////////////////////////////////////////////////////////////////////////////////
//
// Search Reports
//
////////////////////////////////////////////////////////////////////////////////

/// Aggregated data concerning the search, used for reporting in various places
#[derive(Debug, Clone)]
pub struct SearchReport {
    /// The nominal depth of the search
    pub depth: u8,

    /// The maximum depth searched to (in, e.g., QSearch)
    pub seldepth: u8,

    /// The number of nodes searched.
    pub nodes: u32,

    /// The total duration of the search
    pub duration: Duration,

    /// The best score found in the search
    pub score: Eval,

    /// The principal variation compiled by the search
    pub pv: Vec<Move>,

    /// The occupancy of the tranpsosition table, as a per mille value.
    pub hashfull: u32,
}

impl SearchReport {
    pub fn new(search: &Search, tt: &TTable, pv: PVTable, score: Eval) -> Self {
        Self {
            score,
            depth: search.depth as u8,
            seldepth: search.seldepth as u8,
            nodes: search.tc.nodes(),
            duration: search.tc.elapsed(),
            pv: Vec::from(pv.moves()),
            hashfull: (1000.0 * tt.occupancy()) as u32,
        }
    }

    pub fn default() -> Self {
        Self {
            depth: 0,
            seldepth: 0,
            nodes: 0,
            duration: Duration::ZERO,
            score: 0,
            pv: Vec::new(),
            hashfull: 0,
        }
    }
}

impl From<&SearchReport> for SearchInfo {
    fn from(report: &SearchReport) -> Self {
        let nps = 1000 * report.nodes
            .checked_div(report.duration.as_millis() as u32)
            .unwrap_or_default();

        Self {
            depth: Some(report.depth),
            seldepth: Some(report.seldepth),
            time: Some(report.duration.as_millis() as u64),
            nodes: Some(report.nodes),
            score: Some(report.score),
            pv: report.pv.clone(),
            hashfull: Some(report.hashfull),
            nps: Some(nps),
            currmove: None,
            currmovenumber: None,
        }
    }
}
