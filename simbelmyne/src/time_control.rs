use std::{time::{Instant, Duration}, sync::{atomic::{AtomicBool, Ordering}, Arc}};

use chess::piece::Color;

const OVERHEAD: Duration = Duration::from_millis(50);
const DEFAULT_MOVES: u32 = 30;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TCType {
    Infinite,
    Depth(usize),
    Nodes(usize),
    FixedTime(Duration),
    VariableTime { 
        wtime: Duration, 
        btime: Duration, 
        winc: Duration, 
        binc: Duration, 
        movestogo: Option<u32> 
    }
}

impl TCType {
    pub fn max_time(&self, side: Color) -> Duration {
        use TCType::*;

        match &self {
            FixedTime(max_time) => max_time.saturating_sub(OVERHEAD),

            VariableTime { wtime, btime, winc, binc, movestogo } => {
                let time = if side.is_white() { wtime } else { btime };
                let inc = if side.is_white() { winc } else { binc };
                let movestogo = movestogo.unwrap_or(DEFAULT_MOVES);

                (*time / movestogo + *inc).saturating_sub(OVERHEAD)
            },

            _ => Duration::ZERO
        }
    }
}

#[derive(Debug, Clone, )]
pub struct TimeControl {
    tc: TCType,
    start: Instant,
    max_time: Duration,
    stop: Arc<AtomicBool>,
}

impl TimeControl {
    pub fn new(tc_type: TCType, side: Color) -> (Self, TimeControlHandle) {
        let stop: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));

        let tc = TimeControl {
            tc: tc_type,
            start: Instant::now(),
            max_time: tc_type.max_time(side),
            stop: stop.clone(),
        };

        let handle = TimeControlHandle { 
            stop: stop.clone()
        };

        (tc, handle)
    }

    pub fn fixed_depth(depth: usize) -> (TimeControl, TimeControlHandle) {
        Self::new(TCType::Depth(depth), Color::White)
    }

    pub fn fixed_nodes(nodes: usize) -> (TimeControl, TimeControlHandle) {
        Self::new(TCType::Nodes(nodes), Color::White)
    }

    pub fn infinite() -> (TimeControl, TimeControlHandle) {
        Self::new(TCType::Infinite, Color::White)
    }

    pub fn fixed_time(duration: Duration, side: Color) -> (TimeControl, TimeControlHandle) {
        Self::new(TCType::FixedTime(duration), side)
    }


    pub fn should_continue(&self, depth: usize, nodes: usize) -> bool {
        // Always respect the global stop flag
        if self.stopped() {
            return false;
        }     

        // If no global stop is detected, then respect the chosen time control
        match self.tc {
            TCType::Depth(max_depth) => depth < max_depth,
            TCType::Nodes(max_nodes) => nodes < max_nodes,
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
}

pub struct TimeControlHandle {
    stop: Arc<AtomicBool>,
}

impl TimeControlHandle {
    pub fn stop(&self) {
        self.stop.store(true, Ordering::SeqCst);
    }
}
