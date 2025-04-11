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
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering;
use std::time::Duration;
use crate::evaluate::kp_cache::KingPawnCache;
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
use chess::piece::Color;
use uci::search_info::SearchInfo;
use uci::search_info::Score as UciScore;
use uci::time_control::TimeControl;
use uci::wdl::WDL_MODEL;

pub mod params;
mod zero_window;
mod negamax;
mod quiescence;
mod aspiration;

const KP_CACHE_SIZE: usize = 2;

pub struct SearchRunner<'a> {
    pub id: usize,
    pub depth: usize,
    pub seldepth: usize,
    pub tt: &'a TTable,
    pub history: Box<History>,
    pub kp_cache: KingPawnCache,
    pub nodes: NodeCounter<'a>,
    pub tc: TimeController,
    stack: [SearchStackEntry; MAX_DEPTH],
    aborted: bool,
}

impl<'a> SearchRunner<'a> {
    pub fn new(id: usize, tt: &'a TTable, nodes: NodeCounter<'a>) -> Self {
        // Just a placeholder TC. TC will get populated when search() is called.
        let (tc, _) = TimeController::new(TimeControl::Infinite, Color::White);

        Self {
            id,
            depth: 1,
            seldepth: 1,
            tt,
            history: History::boxed(),
            kp_cache: KingPawnCache::with_capacity(KP_CACHE_SIZE),
            nodes,
            stack: [SearchStackEntry::default(); MAX_DEPTH],
            tc,
            aborted: false,
        }
    }

    pub fn reinit(&mut self) {
        self.depth = 1;
        self.seldepth = 1;
        self.nodes.clear_local();
        self.stack = [SearchStackEntry::default(); MAX_DEPTH];
        self.aborted = false;
        self.history.clear_nodes();
    }
}

////////////////////////////////////////////////////////////////////////////////
//
// Iterative deepening search
//
////////////////////////////////////////////////////////////////////////////////

impl<'a> SearchRunner<'a> {
    pub fn search<const DEBUG: bool>(
        &mut self, 
        mut pos: Position, 
        tc: TimeController,
    ) -> SearchReport {
        let mut latest_report = SearchReport::default();
        let mut pv = PVTable::new();
        let mut prev_best_move = None;
        let mut best_move_stability = 0;
        let mut previous_score = 0;
        let mut score_stability = 0;
        self.reinit(); // Clear previous search data
        self.tc = tc;

        // If there is only one legal move, notify the the time controller that
        // we don't want to waste any more time here.
        if pos.board.legal_moves::<All>().len() == 1 {
            self.tc.stop_early();
        }

        while self.depth <= MAX_DEPTH && self.tc.should_start_search(self.depth) {
            pv.clear();
            self.history.clear_all_killers();

            ////////////////////////////////////////////////////////////////////
            //
            // Aspiration window search
            //
            ////////////////////////////////////////////////////////////////////

            let score = self.aspiration_search(&mut pos, latest_report.score, &mut pv);

            // If we got interrupted in the search, don't store the 
            // half-completed search state. Just break and return the previous
            // iteration's search.
            if self.aborted {
                break;
            }

            latest_report = SearchReport::new(&self, score, &pv);

            ////////////////////////////////////////////////////////////////////
            //
            // Update the time controller with gathered search statistics
            //
            ////////////////////////////////////////////////////////////////////

            if self.id == 0 {
                // Best move stability
                if prev_best_move == Some(pv.pv_move()) {
                    best_move_stability += 1;
                } else {
                    best_move_stability = 0;
                }
                prev_best_move = Some(pv.pv_move());

                if score >= previous_score - 10 && score <= previous_score + 10 {
                    score_stability += 1;
                } else {
                    score_stability = 0;
                }
                previous_score = score;

                // Calculate the fraction of nodes spent on the current best move
                let bm_nodes = self.history.get_nodes(pv.pv_move());
                let node_frac = bm_nodes as f64 / self.nodes.local() as f64;

                self.tc.update(best_move_stability, node_frac, score_stability);
            }

            ////////////////////////////////////////////////////////////////////
            //
            // Print search output
            //
            ////////////////////////////////////////////////////////////////////

            if DEBUG && self.id == 0 {
                let wdl_params = WDL_MODEL.params(&pos.board);
                let info = SearchInfo::from(&latest_report);

                // When the output is a terminal, we pretty-print the output
                // and include WDL stats.
                if std::io::stdout().is_terminal() {
                    println!("{}", info.to_pretty(&pos.board, wdl_params));
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

            self.depth += 1;
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
    pub fn new(thread: &SearchRunner, score: Score, pv: &PVTable) -> Self {
        Self {
            score,
            depth: thread.depth as u8,
            seldepth: thread.seldepth as u8,
            nodes: thread.nodes.global(),
            duration: thread.tc.elapsed(),
            pv: Vec::from(pv.moves()),
            hashfull: (1000.0 * thread.tt.occupancy()) as u32,
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

////////////////////////////////////////////////////////////////////////////////
//
// Node counter
//
////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct NodeCounter<'a> {
    local: u32,
    buffer: u32,
    global: &'a AtomicU32,
}

impl<'a> NodeCounter<'a> {
    const INTERVAL: u32 = 2048;
    pub fn new(global: &'a AtomicU32) -> Self {
        Self {
            global,
            local: global.load(Ordering::Relaxed),
            buffer: 0,
        }
    }

    pub fn increment(&mut self) {
        self.local += 1;
        self.buffer += 1;

        if self.buffer >= Self::INTERVAL {
            self.global.fetch_add(self.buffer, Ordering::Relaxed);
            self.buffer = 0;
        }
    }

    pub fn clear_global(&self) {
        self.global.store(0, Ordering::Relaxed);
    }

    pub fn clear_local(&mut self) {
        self.local = 0;
        self.buffer = 0;
    }

    pub fn local(&self) -> u32 {
        self.local
    }

    pub fn global(&self) -> u32 {
        self.global.load(Ordering::Relaxed)
    }
}
