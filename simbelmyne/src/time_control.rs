//! This module holds all the time/progress tracking for a single game.
//!
//! A search can be performed with a bunch of different constraints, depending
//! on the situation
//!
//! 1. Fixed depth: We want to perform the search up to a (nominal) depth of N
//!   moves. This typically does not include extensions like Quiescence search
//!   (which are partial searches to increase our confidence in the actual 
//!   search)
//!
//! 2. Fixed nodes: Search until we've covered a certain number of nodes.
//!
//! 3. Fixed time: Search for at most x milliseconds. 
//!
//! 4. Clock: Given a time on the clock for white and black, and perhaps some
//!   increments, we need to try and divide this time optimally between all the
//!   moves.

use std::time::Instant;
use std::time::Duration;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::sync::atomic::AtomicBool;
use chess::board::Board;
use chess::piece::Color;
use uci::time_control::TimeControl;

use crate::search::params::base_time_frac;
use crate::search::params::hard_time_frac;
use crate::search::params::inc_frac;
use crate::search::params::limit_time_frac;
use crate::search::params::node_frac_base;
use crate::search::params::node_frac_mult;
use crate::search::params::soft_time_frac;

/// Allow an overhead to make sure we don't time out because of UCI communications
const OVERHEAD: Duration = Duration::from_millis(20);

/// How often should we check timers and atomics for stopping conditions?
const CHECKUP_WINDOW: u32 = 4096;

/// The time controller is in charge for determining when a search should 
/// continue or stop in order not to violate the requested time control.
#[derive(Debug, Clone)]
pub struct TimeController {
    /// The type of time control (depth, nodes, time or clock)
    tc: TimeControl,

    /// The instant the search was started
    start: Instant,

    /// The base time off of which we calculate the running soft time
    base_soft_time: Duration,

    /// Time limit after which it's not worth it starting a new search
    soft_time: Duration,

    /// Time limit after which we should abort an ongoing search and return ASAP
    hard_time: Duration,

    /// A global, thread-safe, stop-flag that we can use to abort the search 
    /// thread from the main thread.
    stop: Arc<AtomicBool>,

    /// The next node count when we should check the timers and atomics on 
    /// whether to continue or not.
    next_checkup: u32,

    /// Flag that allows the search to signal that we shouldn't start a new ID
    /// iteration. (E.g, when the position is forced)
    stop_early: bool,

    /// Correction factor to the soft_time derived from how stable the best move
    /// was across iterations
    bm_stability_factor: f64,

    /// Correction factor to the soft_time derived from what fraction of nodes
    /// was spent searching the current best move
    node_frac_factor: f64,

    /// Correction factor to the soft_time derived from how stable the search 
    /// score was across iterations
    score_stability_factor: f64,
}

impl TimeController {
    // Scales (as percents) by which to scale the remaining time according to 
    // the stability of `best_move` between ID iterations.
    const BESTMOVE_STABILITY: [f64; 5] = [2.50, 1.20, 0.90, 0.80, 0.75];
    const SCORE_STABILITY: [f64; 5] = [1.25, 1.15, 1.00, 0.94, 0.88];

    /// Create a new controller, and return a handle that the caller can use
    /// to abort the search.
    pub fn new(tc_type: TimeControl, side: Color) -> (Self, TimeControlHandle) {
        use TimeControl::*;

        // Create a handle that the main thread can use to abort the search.
        let stop: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
        let handle = TimeControlHandle { stop: stop.clone() };

        let mut tc = TimeController {
            tc: tc_type,
            base_soft_time: Duration::default(),
            soft_time: Duration::default(),
            hard_time: Duration::default(),
            start: Instant::now(),
            stop: stop.clone(),
            next_checkup: CHECKUP_WINDOW,
            stop_early: false,
            bm_stability_factor: 1.0,
            node_frac_factor: 1.0,
            score_stability_factor: 1.0,
        };

        // Hard time determines when we should abort an ongoing search.
        match tc_type {
            FixedTime(max_time) => {
                tc.hard_time = max_time.saturating_sub(OVERHEAD);
                tc.soft_time = tc.hard_time;
            },


            // Allocate time (inversely) proportional to the estimated number
            // of remaining moves.
            Clock { wtime, btime, winc, binc, movestogo } => {
                let time = if side.is_white() { wtime } else { btime };
                let inc = if side.is_white() { winc } else { binc };
                let inc = inc.unwrap_or_default();

                let allowed_time = time.saturating_sub(OVERHEAD);
                let limit_time = limit_time_frac() * allowed_time / 100;

                let base_time = if let Some(movestogo) = movestogo {
                    allowed_time / movestogo + inc_frac() * inc / 100
                } else {
                    base_time_frac() * allowed_time / 1000 + inc_frac() * inc / 100
                };

                tc.hard_time = (hard_time_frac() * base_time / 100).min(limit_time);
                tc.base_soft_time = (soft_time_frac() * base_time / 100).min(limit_time);
                tc.soft_time = tc.base_soft_time;
            },

            _ => {}
        };

        (tc, handle)
    }

    /// Check whether the search should continue, depending on the particular
    /// time control. This check allows for some leeway, and is only checked if
    /// we're due for a "checkup" (that is, if we've exceeded the "checkup node
    /// count".)
    pub fn should_continue(&mut self, nodes: u32) -> bool {
        // If we're not due for a checkup, simply return
        if nodes < self.next_checkup {
            return true;
        }

        // Set the next checkup point
        self.next_checkup = nodes + CHECKUP_WINDOW;

        // Always respect the global stop flag
        if self.stopped() {
            return false;
        }

        // If no global stop is detected, then respect the chosen time control
        match self.tc {
            TimeControl::Nodes(max_nodes) => {
                self.next_checkup < max_nodes as u32
            },

            TimeControl::FixedTime(_time) => {
                self.elapsed() < self.hard_time
            }, 

            TimeControl::Clock { .. } => {
                self.elapsed() < self.hard_time
            },

            _ => true,
        }
    }

    /// Check whether we should start a new iterative deepening search.
    pub fn should_start_search(&self, depth: usize) -> bool {
        // Make sure we always do at least _one_ search iteration.
        if depth <= 1 {
            return true;
        }

        // Always respect the global stop flag
        if self.stopped() {
            return false;
        }

        // If no global stop is detected, then respect the chosen time control
        match self.tc {
            TimeControl::Depth(max_depth) => {
                depth <= max_depth
            },

            TimeControl::Nodes(max_nodes) => {
                self.next_checkup < max_nodes as u32
            },

            TimeControl::FixedTime(_) => {
                self.elapsed() < self.hard_time
            },

            TimeControl::Clock { .. } => {
                // Stop early if the search signaled that there's no point 
                // searching any further.
                if self.stop_early {
                    return false;
                }

                let mut adjusted_soft_time = self.soft_time.as_millis() as f64;
                adjusted_soft_time *= self.bm_stability_factor;
                adjusted_soft_time *= self.node_frac_factor;

                if depth >= 7 {
                    adjusted_soft_time *= self.score_stability_factor;
                }

                self.elapsed().as_millis() < adjusted_soft_time as u128
            },

            _ => true,
        }
    }

    /// Update the soft time limit with additional information gathered through
    /// the search
    pub fn update(&mut self, stability: usize, node_frac: f64, score_stability: usize) {
        self.bm_stability_factor = Self::BESTMOVE_STABILITY[stability.min(4)];
        self.score_stability_factor = Self::SCORE_STABILITY[score_stability.min(4)];
        self.node_frac_factor = (node_frac_base() as f64 / 100.0 - node_frac) 
            * node_frac_mult() as f64 / 100.0;
    }

    /// Check whether the search has been aborted.
    pub fn stopped(&self) -> bool {
        self.stop.load(Ordering::SeqCst)
    }

    /// Return the time that's elapsed since the start of the search.
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    /// Signal that the search can stop early, rather than starting a new
    /// ID iteration
    pub fn stop_early(&mut self) {
        self.stop_early = true;
    }
}

/// A wrapper for easily aborting a search, even on a different thread.
#[derive(Clone)]
pub struct TimeControlHandle {
    stop: Arc<AtomicBool>,
}

impl TimeControlHandle {
    /// Stop the current search.
    pub fn stop(&self) {
        self.stop.store(true, Ordering::SeqCst);
    }
}
