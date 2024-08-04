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
use std::io::IsTerminal;
use std::time::Duration;
use crate::evaluate::ScoreExt;
use crate::history_tables::pv::PVTable;
use crate::history_tables::History;
use crate::search::params::MAX_DEPTH;
use crate::transpositions::TTable;
use crate::time_control::TimeController;
use crate::position::Position;
use crate::evaluate::Score;
use chess::movegen::legal_moves::All;
use chess::movegen::moves::Move;
use uci::search_info::SearchInfo;
use uci::search_info::Score as UciScore;
use uci::wdl::WdlModel;

pub(crate) mod params;
mod zero_window;
mod negamax;
mod quiescence;
mod aspiration;


const WDL_MODEL: WdlModel = WdlModel {
    a: [-1687.03839457, 4936.97013397, -4865.11135831, 1907.15036483],
    b: [-62.39623703, 287.82241928, -379.70952976, 345.03030228],
};

/// A Search struct holds both the parameters, as well as metrics and results, 
/// for a given search.
#[derive(Debug)]
pub struct Search<'a> {
    // The (nominal) depth of this search. This does not take QSearch into 
    // consideration
    pub depth: usize,

    // The so-called "selective depth", the deepest ply we've searched
    pub seldepth: usize,

    // The time control for the search
    pub tc: &'a mut TimeController,

    /// Whether the search was aborted half-way
    aborted: bool,

    // Search stack
    stack: [SearchStackEntry; MAX_DEPTH],

    /// All of the history tables and related quantities
    pub history: &'a mut History,

    /// The total number of nodes searched so far, across iterations
    pub nodes: u32,
}

impl<'a> Search<'a> {
    /// Create a new search
    pub fn new(
        depth: usize, 
        history: &'a mut History,
        tc: &'a mut TimeController, 
    ) -> Self {
        Self {
            depth,
            seldepth: 0,
            tc,
            history,
            aborted: false,
            stack: [SearchStackEntry::default(); MAX_DEPTH],
            nodes: 0,
        }
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
    pub fn search<const DEBUG: bool>(
        &self, 
        tt: &mut TTable, 
        history: &mut History,
        tc: &mut TimeController, 
    ) -> SearchReport {
        let mut latest_report = SearchReport::default();
        let mut pv = PVTable::new();
        let mut prev_best_move = None;
        let mut best_move_stability = 0;
        history.clear_nodes();

        // If there is only one legal move, notify the the time controller that
        // we don't want to waste any more time here.
        if self.board.legal_moves::<All>().len() == 1 {
            tc.stop_early();
        }

        let mut search = Search::new(1, history, tc);

        while search.depth <= MAX_DEPTH && search.tc.should_start_search(search.depth) {
            pv.clear();
            search.history.clear_all_killers();
            
            ////////////////////////////////////////////////////////////////////
            //
            // Aspiration window search
            //
            ////////////////////////////////////////////////////////////////////

            let score = self.aspiration_search(
                search.depth, 
                latest_report.score, 
                tt, 
                &mut pv, 
                &mut search
            );

            // If we got interrupted in the search, don't store the 
            // half-completed search state. Just break and return the previous
            // iteration's search.
            if search.aborted {
                break;
            }

            latest_report = SearchReport::new(&search, tt, pv, score);

            ////////////////////////////////////////////////////////////////////
            //
            // Update the time controller with gathered search statistics
            //
            ////////////////////////////////////////////////////////////////////

            // Best move stability
            if prev_best_move == Some(pv.pv_move()) {
                best_move_stability += 1;
            } else {
                best_move_stability = 0;
            }

            // Store the new best move for the next iteration
            prev_best_move = Some(pv.pv_move());

            // Calculate the fraction of nodes spent on the current best move
            let bm_nodes = search.history.get_nodes(pv.pv_move());
            let node_frac = bm_nodes as f64 / search.nodes as f64;

            search.tc.update(best_move_stability, node_frac);

            ////////////////////////////////////////////////////////////////////
            //
            // Print search output
            //
            ////////////////////////////////////////////////////////////////////

            if DEBUG {
                let wdl_params = WDL_MODEL.params(&self.board);
                let info = SearchInfo::from(&latest_report);

                // When the output is a terminal, we pretty-print the output
                // and include WDL stats.
                if std::io::stdout().is_terminal() {
                    println!("{}", info.to_pretty(wdl_params));
                } 

                // If we're talking to another process, _and we're not in wdl
                // mode_, we print UCI compliant output, but with the eval 
                // rescaled according to the WDL model.
                else if !cfg!(feature = "wdl") {
                    println!("info {}", info.to_uci(wdl_params));
                } 

                // If we're talking to a process, _and_ we're in WDL mode, we
                // output the score in internal, unscaled, values.
                else {
                    println!("info {info}");
                }

            }

            search.depth += 1;
        }

        latest_report
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
    pub score: Score,

    /// The principal variation compiled by the search
    pub pv: Vec<Move>,

    /// The occupancy of the tranpsosition table, as a per mille value.
    pub hashfull: u32,
}

impl SearchReport {
    pub fn new(search: &Search, tt: &TTable, pv: PVTable, score: Score) -> Self {
        Self {
            score,
            depth: search.depth as u8,
            seldepth: search.seldepth as u8,
            nodes: search.nodes,
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
        let nps = (1_000_000 * report.nodes as u64)
            .checked_div(report.duration.as_micros() as u64)
            .unwrap_or_default();

        Self {
            depth: Some(report.depth),
            seldepth: Some(report.seldepth),
            time: Some(report.duration.as_millis() as u64),
            nodes: Some(report.nodes),
            score: Some(report.score.to_uci()),
            pv: report.pv.clone(),
            hashfull: Some(report.hashfull),
            nps: Some(nps),
            currmove: None,
            currmovenumber: None,
        }
    }
}

trait ScoreUciExt {
    fn to_uci(self) -> UciScore;
}

impl ScoreUciExt for Score {
    fn to_uci(self) -> UciScore {
        if self.is_mate() {
            UciScore::Mate(self.signum() * (self.mate_distance() + 1) / 2)
        } else {
            UciScore::Cp(self)
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
//
// Search Stack Entry
//
// Keep track of search information about a given ply that we want to share
// between plies.
//
////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Copy, Clone, Default)]
struct SearchStackEntry {
    /// The eval for the last position in this ply
    pub eval: Score,

    /// A move to be excluded from the search at this ply (used for singular
    /// extensions
    pub excluded: Option<Move>,

    pub double_exts: u8
}
