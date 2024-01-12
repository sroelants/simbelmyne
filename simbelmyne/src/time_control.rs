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
use uci::time_control::TimeControl;

/// Allow an overhead to make sure we don't time out because of UCI communications
const OVERHEAD: Duration = Duration::from_millis(20);

/// How often should we check timers and atomics for stopping conditions?
const CHECKUP_WINDOW: u32 = 4096;

/// If no moves are provided in the time control, assume we're playing with a 
/// 30 move limit.
const DEFAULT_MOVES: u32 = 20;

/// The time controller is in charge for determining when a search should 
/// continue or stop in order not to violate the requested time control.
#[derive(Debug, Clone)]
pub struct TimeController {
    /// The type of time control (depth, nodes, time or clock)
    tc: TimeControl,

    /// The instant the search was started
    start: Instant,

    /// Time limit after which it's not worth it starting a new search
    soft_time: Duration,

    /// Time limit after which we should abort an ongoing search and return ASAP
    hard_time: Duration,

    /// A global, thread-safe, stop-flag that we can use to abort the search 
    /// thread from the main thread.
    stop: Arc<AtomicBool>,

    /// The amount of nodes processed so far
    nodes: u32,

    /// The next node count when we should check the timers and atomics on 
    /// whether to continue or not.
    next_checkup: u32,
}

impl TimeController {
    /// Create a new controller, and return a handle that the caller can use
    /// to abort the search.
    pub fn new(tc_type: TimeControl, board: Board) -> (Self, TimeControlHandle) {
        use TimeControl::*;
        let side = board.current;
        let moves_played = board.full_moves as u32;

        // Create a handle that the main thread can use to abort the search.
        let stop: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
        let handle = TimeControlHandle { stop: stop.clone() };

        // Hard time determines when we should abort an ongoing search.
        let hard_time = match tc_type {
            FixedTime(max_time) => max_time.saturating_sub(OVERHEAD),

            // Allocate time (inversely) proportional to the estimated number
            // of remaining moves.
            Clock { wtime, btime, winc, binc, movestogo } => {
                let time = if side.is_white() { wtime } else { btime };
                let inc = if side.is_white() { winc } else { binc };

                let movestogo = movestogo
                    .unwrap_or(DEFAULT_MOVES)
                    .saturating_sub(moves_played)
                    .max(10);

                let mut max_time = time / movestogo;
                max_time += 3 / 4 * inc.unwrap_or_default();

                Duration::min(time, max_time).saturating_sub(OVERHEAD)
            },

            _ => Duration::ZERO
        };

        // Soft time determines when it's no longer worth starting a fresh 
        // search, but it's not quite time to abort an ongoing search.
        let soft_time = hard_time * 6 / 10;

        let tc = TimeController {
            tc: tc_type,
            hard_time,
            soft_time,
            start: Instant::now(),
            stop: stop.clone(),
            nodes: 0,
            next_checkup: CHECKUP_WINDOW,
        };

        (tc, handle)
    }

    /// Update the checkup node count, when we check whether to continue 
    /// searching or not
    fn reset_checkup(&mut self) {
        self.next_checkup = self.nodes + CHECKUP_WINDOW;
    }

    /// Check whether the search should continue, depending on the particular
    /// time control. This check allows for some leeway, and is only checked if
    /// we're due for a "checkup" (that is, if we've exceeded the "checkup node
    /// count".)
    pub fn should_continue(&mut self) -> bool {
        // If we're not due for a checkup, simply return
        if self.nodes < self.next_checkup {
            return true;
        }

        // Set the next checkup point
        self.reset_checkup();

        // Always respect the global stop flag
        if self.stopped() {
            return false;
        }     

        // If no global stop is detected, then respect the chosen time control
        match self.tc {
            TimeControl::Nodes(max_nodes) => {
                self.nodes + CHECKUP_WINDOW < max_nodes as u32
            },

            TimeControl::FixedTime(max_time) => {
                max_time < self.hard_time
            }, 

            TimeControl::Clock { .. } => {
                self.elapsed() < self.soft_time
            },

            _ => true,
        }
    }

    /// Check whether we should start a new iterative deepening search.
    pub fn should_start_search(&self, depth: usize) -> bool {
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
                self.nodes + CHECKUP_WINDOW < max_nodes as u32
            },

            TimeControl::FixedTime(_) | TimeControl::Clock { .. } => {
                self.elapsed() < self.hard_time
            },

            _ => true,
        }

    }

    /// Check whether the search has been aborted.
    pub fn stopped(&self) -> bool {
        self.stop.load(Ordering::SeqCst)
    }

    /// Return the time that's elapsed since the start of the search.
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    /// Increment the counter of visited nodes
    pub fn add_node(&mut self) {
        self.nodes += 1;
    }

    /// Return the number of visited nodes
    pub fn nodes(&self) -> u32 {
        self.nodes
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
