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
use crate::evaluate::ScoreExt;
use crate::history_tables::conthist::ContHist;
use crate::history_tables::countermoves::CountermoveTable;
use crate::history_tables::history::HistoryIndex;
use crate::history_tables::history::HistoryTable;
use crate::history_tables::killers::Killers;
use crate::history_tables::pv::PVTable;
use crate::search::params::MAX_DEPTH;
use crate::transpositions::TTable;
use crate::time_control::TimeController;
use crate::position::Position;
use crate::evaluate::Score;
use chess::movegen::legal_moves::All;
use chess::movegen::moves::Move;
use uci::search_info::SearchInfo;
use uci::search_info::Score as UciScore;
use uci::engine::UciEngineMessage;

use self::params::SearchParams;

pub(crate) mod params;
mod zero_window;
mod negamax;
mod quiescence;
mod aspiration;

/// A Search struct holds both the parameters, as well as metrics and results, 
/// for a given search.
#[derive(Debug)]
pub struct Search<'a> {
    // The (nominal) depth of this search. This does not take QSearch into 
    // consideration
    pub depth: usize,

    // The so-called "selective depth", the deepest ply we've searched
    pub seldepth: usize,

    /// Values for the various search parameters
    pub search_params: &'a SearchParams,

    // The time control for the search
    pub tc: &'a mut TimeController,

    /// Whether the search was aborted half-way
    aborted: bool,

    // Search stack
    stack: [SearchStackEntry; MAX_DEPTH],

    // History tables

    /// The set of killer moves at a given ply.
    pub killers: [Killers; MAX_DEPTH],

    /// The countermove table
    pub countermoves: Box<CountermoveTable>,

    /// Main history table
    pub history_table: &'a mut HistoryTable,

    /// Continuation history table
    pub conthist_tables: &'a mut [Box<ContHist>; 2],
}

impl<'a> Search<'a> {
    /// Create a new search
    pub fn new(
        depth: usize, 
        history_table: &'a mut HistoryTable, 
        conthist_tables: &'a mut [Box<ContHist>; 2],
        tc: &'a mut TimeController, 
        search_params: &'a SearchParams
    ) -> Self {
        Self {
            depth,
            seldepth: 0,
            tc,
            killers: [Killers::new(); MAX_DEPTH],
            countermoves: CountermoveTable::boxed(),
            history_table,
            conthist_tables,
            search_params,
            aborted: false,
            stack: [SearchStackEntry::default(); MAX_DEPTH],
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
        history: &mut HistoryTable, 
        conthists: &mut [Box<ContHist>; 2], 
        tc: &mut TimeController, 
        search_params: &SearchParams
    ) -> SearchReport {
        let mut depth = 1;
        let mut latest_report = SearchReport::default();
        let mut pv = PVTable::new();

        if self.board.legal_moves::<All>().len() == 1 {
            tc.stop_early();
        }

        while depth <= MAX_DEPTH && tc.should_start_search(depth) {
            pv.clear();
            let mut search = Search::new(
                depth, 
                history, 
                conthists, 
                tc, 
                search_params
            );

            let score = self.aspiration_search(
                depth, 
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

            if DEBUG {
                println!("{}", UciEngineMessage::Info((&latest_report).into()));
            }

            depth += 1;
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
    /// The last index for this ply that can be used to index into history tables
    pub history_index: HistoryIndex,

    /// The eval for the last position in this ply
    pub eval: Score,
}
