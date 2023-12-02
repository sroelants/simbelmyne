use std::{time::{Instant, Duration}, sync::{atomic::{AtomicBool, Ordering}, Arc}};


#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TCType {
    Depth(usize),
    Nodes(usize),
    MoveTime(Duration),
    Infinite,
}


#[derive(Debug, Clone, )]
pub struct TimeControl {
    tc: TCType,
    start: Instant,
    stop: Arc<AtomicBool>,
}

impl TimeControl {
    pub fn new(tc_type: TCType) -> (Self, TimeControlHandle) {
        let stop: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));

        let tc = TimeControl {
            tc: tc_type,
            start: Instant::now(),
            stop: stop.clone(),
        };

        let handle = TimeControlHandle { 
            stop: stop.clone()
        };

        (tc, handle)
    }

    pub fn fixed_depth(depth: usize) -> (TimeControl, TimeControlHandle) {
        Self::new(TCType::Depth(depth))
    }

    pub fn fixed_nodes(nodes: usize) -> (TimeControl, TimeControlHandle) {
        Self::new(TCType::Nodes(nodes))
    }

    pub fn fixed_time(duration: Duration) -> (TimeControl, TimeControlHandle) {
        Self::new(TCType::MoveTime(duration))
    }

    pub fn infinite() -> (TimeControl, TimeControlHandle) {
        Self::new(TCType::Infinite)
    }

    pub fn should_continue(&self, depth: usize, nodes: usize) -> bool {
        // Always respect the global stop flag
        if self.stopped() {
            return false;
        }     

        // If no global stop is detected, then respect the chosen time control
        match self.tc {
            TCType::Depth(max_depth) => depth <= max_depth,
            TCType::Nodes(max_nodes) => nodes <= max_nodes,
            TCType::MoveTime(duration) => self.start.elapsed() <= duration,
            TCType::Infinite => true,
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
