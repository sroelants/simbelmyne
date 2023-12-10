use std::{time::{Instant, Duration}, sync::{atomic::{AtomicBool, Ordering}, Arc}};
use shared::uci::TCType;

use chess::piece::Color;

const OVERHEAD: Duration = Duration::from_millis(50);
const DEFAULT_MOVES: u32 = 30;

#[derive(Debug, Clone, )]
pub struct TimeControl {
    tc: TCType,
    start: Instant,
    max_time: Duration,
    stop: Arc<AtomicBool>,
    nodes: u32,
}

impl TimeControl {
    pub fn new(tc_type: TCType, side: Color) -> (Self, TimeControlHandle) {
        use TCType::*;
        let stop: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));

        let max_time = match tc_type {
            FixedTime(max_time) => max_time.saturating_sub(OVERHEAD),

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

        let tc = TimeControl {
            tc: tc_type,
            max_time,
            start: Instant::now(),
            stop: stop.clone(),
            nodes: 0,
        };

        let handle = TimeControlHandle { 
            stop: stop.clone()
        };

        (tc, handle)
    }

    pub fn fixed_depth(depth: usize) -> (TimeControl, TimeControlHandle) {
        Self::new(TCType::Depth(depth), Color::White)
    }

    pub fn should_continue(&self, depth: usize) -> bool {
        // Always respect the global stop flag
        if self.stopped() {
            return false;
        }     

        // If no global stop is detected, then respect the chosen time control
        match self.tc {
            TCType::Depth(max_depth) => depth < max_depth,
            TCType::Nodes(max_nodes) => self.nodes < max_nodes as u32,
            TCType::FixedTime(_) | TCType::VariableTime { .. } =>
                self.start.elapsed() < self.max_time,
            _ => true,
        }
    }

    pub fn stopped(&self) -> bool {
        self.stop.load(Ordering::SeqCst)
    }

    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    pub fn add_node(&mut self) {
        self.nodes += 1;
    }

    pub fn nodes(&self) -> u32 {
        self.nodes
    }
}

pub struct TimeControlHandle {
    stop: Arc<AtomicBool>,
}

impl TimeControlHandle {
    pub fn stop(&self) {
        self.stop.store(true, Ordering::SeqCst);
    }
}
