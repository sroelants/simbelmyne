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
use shared::uci::TCType;
use chess::piece::Color;

/// Allow an overhead to make sure we don't time out because of UCI communications
const OVERHEAD: Duration = Duration::from_millis(50);

/// If no moves are provided in the time control, assume we're playing with a 
/// 30 move limit.
const DEFAULT_MOVES: u32 = 30;

/// The time controller is in charge for determining when a search should 
/// continue or stop in order not to violate the requested time control.
#[derive(Debug, Clone)]
pub struct TimeController {
    /// The type of time control (depth, nodes, time or clock)
    tc: TCType,

    /// The instant the search was started
    start: Instant,

    /// The maximum alloted time for the search.
    /// Will be set to `Duration::ZERO` for time controls that don't require it.
    max_time: Duration,

    /// A global, thread-safe, stop-flag that we can use to abort the search 
    /// thread from the main thread.
    stop: Arc<AtomicBool>,

    /// The amount of nodes processed so far
    nodes: u32,
}

impl TimeController {
    /// Create a new controller, and return a handle that the caller can use
    /// to abort the search.
    pub fn new(tc_type: TCType, side: Color) -> (Self, TimeControlHandle) {
        use TCType::*;

        // Create a handle that the main thread can use to abort the search.
        let stop: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
        let handle = TimeControlHandle { stop: stop.clone() };

        // Calculate the maximum alotted time for the search
        let max_time = match tc_type {
            FixedTime(max_time) => max_time.saturating_sub(OVERHEAD),

            // Right now, this just divides up the available time (minus the 
            // overhead) by the number of allowed moves.
            VariableTime { wtime, btime, winc, binc, movestogo } => {
                let time = if side.is_white() { wtime } else { btime };
                let inc = if side.is_white() { winc } else { binc };
                let movestogo = movestogo.unwrap_or(DEFAULT_MOVES);

                let mut max_time = time / movestogo;

                if let Some(inc) = inc {
                    max_time += inc
                }

                max_time.saturating_sub(OVERHEAD)
            },

            _ => Duration::ZERO
        };

        let tc = TimeController {
            tc: tc_type,
            max_time,
            start: Instant::now(),
            stop: stop.clone(),
            nodes: 0,
        };

        (tc, handle)
    }

    /// Check whether the search should continue, depending on the particular
    /// time control.
    pub fn should_continue(&self, depth: usize) -> bool {
        // Always respect the global stop flag
        if self.stopped() {
            return false;
        }     

        // If no global stop is detected, then respect the chosen time control
        match self.tc {
            TCType::Depth(max_depth) => depth <= max_depth,
            TCType::Nodes(max_nodes) => self.nodes < max_nodes as u32,
            TCType::FixedTime(_) | TCType::VariableTime { .. } =>
                self.start.elapsed() < self.max_time,
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
