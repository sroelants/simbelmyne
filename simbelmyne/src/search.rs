use std::{ops::Deref, time::Duration};

use chess::movegen::moves::Move;
use crate::{evaluate::Score, position::Position, transpositions::{TTable, TTEntry, NodeType}, move_picker::MovePicker, time_control::TimeControl};

pub const MAX_DEPTH : usize = 48;

const MAX_KILLERS: usize = 2;

/// A Search struct holds both the parameters, as well as metrics and results, for a given search.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Search {
    // Information
    pub depth: usize,
    
    // Search results
    /// Best move found at each ply of the search
    pub best_moves: [Move; MAX_DEPTH],

    /// The scores found for said best move, at each ply of the search
    pub scores: [i32; MAX_DEPTH],

    /// The set of killer moves at a given ply.
    /// Killer moves are quiet moves (non-captures/promotions) that caused a 
    /// beta-cutoff in that ply.
    pub killers: [Killers; MAX_DEPTH],

    // Stats
    /// The total number of nodes visited in this search
    pub nodes_visited: usize,

    /// The total number of leaf nodes visited in this search
    pub leaf_nodes: usize,

    /// The total number of TT hits for the search
    pub tt_hits: usize,

    /// The time the search took at any given ply
    pub duration: Duration,

    /// The number of beta-cutoffs we found at any given ply;
    pub beta_cutoffs: [usize; MAX_DEPTH],

    // Controls
    /// Options that control what kinds of optimizations should be enabled.
    /// Mostly for debugging purposes
    pub opts: SearchOpts,

    /// Whether or not the search was aborted midway because of TC
    pub aborted: bool
}

impl Search {
    pub fn new(depth: usize, opts: SearchOpts) -> Self {
        Self {
            depth,
            best_moves: [Move::NULL; MAX_DEPTH],
            scores: [i32::default(); MAX_DEPTH],
            killers: [Killers::new(); MAX_DEPTH],
            nodes_visited: 0,
            leaf_nodes: 0,
            tt_hits: 0,
            duration: Duration::default(),
            beta_cutoffs: [0; MAX_DEPTH],
            opts,
            aborted: false,
        }

    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct SearchOpts {
    pub tt: bool,
    pub ordering: bool,
    pub tt_move: bool,
    pub mvv_lva: bool,
    pub killers: bool,
}

impl SearchOpts {
    pub const ALL: Self = {
        Self {
            tt: true,
            tt_move: true,
            ordering: true,
            mvv_lva: true,
            killers: true,
        }
    };

    pub const NONE: Self = {
        Self {
            tt: false,
            tt_move: false,
            ordering: false,
            mvv_lva: false,
            killers: false,
        }
    };

    pub fn new() -> Self {
        Self {
            tt: true,
            tt_move: true,
            ordering: true,
            mvv_lva: true,
            killers: true,
        }
    }
}

pub const DEFAULT_OPTS: SearchOpts = SearchOpts::ALL;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Killers([Move; MAX_KILLERS]);

impl Deref for Killers {
    type Target = [Move; MAX_KILLERS];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Killers {
    fn new() -> Self {
        Killers([Move::NULL; MAX_KILLERS])
    }

    fn add(&mut self, mv: Move) {
        // Make sure we only add distinct moves
        if !self.contains(&mv) {
            self.0.rotate_right(1);
            self.0[0] = mv;
        }
    }
}

pub struct KillersIter {
    killers: Killers,
    index: usize,
}

impl Iterator for KillersIter {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.killers.len() {
            return None;
        }

        let mv = self.killers.0[self.index];
        self.index += 1;

        if mv == Move::NULL {
            return None;
        }

        Some(mv)
    }
}


impl IntoIterator for Killers {
    type Item = Move;
    type IntoIter = KillersIter;

    fn into_iter(self) -> Self::IntoIter {
        KillersIter {
            killers: self,
            index: 0,
        }
    }
}

impl Position {
    pub fn search(&self, tt: &mut TTable, opts: SearchOpts, tc: TimeControl) -> Search {
        let mut result: Search = Search::new(0, opts);
        let mut depth = 0;

        loop {
            depth += 1;
            let mut search = Search::new(depth, opts);

            let start = std::time::Instant::now();
            self.negamax(0, i32::MIN + 1, i32::MAX, tt, &mut search, &tc);
            search.duration = start.elapsed();

            // If we got interrupted in the search, don't store the 
            // half-completed search state. Just break and return the previous
            // iteration's search.
            if search.aborted {
                break;
            } else {
                result = search;
            }
        }

        result
    }

    fn negamax(
        &self, 
        ply: usize, 
        alpha: i32, 
        beta: i32, 
        tt: &mut TTable, 
        search: &mut Search,
        tc: &TimeControl
    ) -> i32 {
        if !tc.should_continue(search.depth, search.nodes_visited) {
            search.aborted = true;
            return i32::MIN;
        }

        let mut best_move = Move::NULL;
        let mut score = Score::MIN + 1;
        let mut node_type = NodeType::Exact;
        let remaining_depth = search.depth - ply;

        let tt_entry = tt.probe(self.hash);


        let tt_usable = tt_entry.is_some_and(|entry| {
            entry.get_depth() == remaining_depth 
            && entry.get_type() == NodeType::Exact
        });

        // 1. Can we use an existing TT entry?
        if tt_usable {
            let tt_entry = tt_entry.unwrap();
            score = tt_entry.get_score();
            best_move = tt_entry.get_move();

            search.tt_hits += 1;
        } else 

        // 2. Is this a leaf node?
        if ply == search.depth {
            score = self.score.total();
            search.leaf_nodes += 1;

        //3. Recurse over all the child nodes
        } else {
            let mut alpha = alpha;

            let legal_moves = self.board.legal_moves();

            let legal_moves = MovePicker::new(
                &self,  
                legal_moves, 
                tt_entry.map(|entry| entry.get_move()),
                search.killers[ply],
                search.opts
            );

            for mv in legal_moves {
                let new_score = -self
                    .play_move(mv)
                    .negamax(ply + 1, -beta, -alpha, tt, search, tc);

                if new_score > score {
                    score = new_score;
                    best_move = mv;
                    node_type = NodeType::Exact;
                }

                if new_score > alpha {
                    alpha = new_score;
                    node_type = NodeType::Upper;
                }

                if new_score > beta {
                    node_type = NodeType::Lower;
                    search.beta_cutoffs[ply] += 1;

                    // Check that it isn't the tt_move, or a capture or a promotion...
                    if mv.is_quiet() && search.opts.killers { 
                        search.killers[ply].add(mv);
                    }

                    break;
                }
            }
        }

        // Store this entry in the TT
        if search.opts.tt {
            tt.insert(TTEntry::new(
                self.hash,
                best_move,
                score,
                remaining_depth,
                node_type,
            ));
        }

        // Propagate up the results
        search.best_moves[ply] = best_move;
        search.scores[ply] = score;
        search.nodes_visited += 1;

        score
    }
}
