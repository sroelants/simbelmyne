use std::{ops::Deref, time::Duration};

use chess::movegen::moves::Move;
use crate::{evaluate::Score, position::Position, transpositions::{TTable, TTEntry, NodeType}, move_picker::MovePicker};

const MAX_DEPTH: usize = 48;
const MAX_PLY : usize = 48;

const MAX_KILLERS: usize = 2;

/// A Search struct holds both the parameters, as well as metrics and
/// results, for a given search.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Search {
    // Information
    /// The depth of the search
    pub depth: usize,

    // Search results
    /// Best move found at each ply of the search
    pub best_moves: [Move; MAX_PLY],

    /// The scores found for said best move, at each ply of the search
    pub scores: [i32; MAX_PLY],

    /// The static evaluation of the board position, at a given ply
    pub eval: [i32; MAX_PLY],

    /// The set of killer moves at a given ply.
    /// Killer moves are quiet moves (non-captures/promotions) that caused a 
    /// beta-cutoff in that ply.
    pub killers: [Killers; MAX_PLY],

    // Stats
    /// The total number of nodes visited from any given ply onward
    pub nodes_visited: [usize; MAX_PLY],

    /// The total number of TT hits for the search
    pub tt_hits: usize,

    /// The time the search took at any given ply
    pub durations: [Duration; MAX_PLY],

    /// The number of beta-cutoffs we found at any given ply;
    pub beta_cutoffs: [usize; MAX_PLY],

    // Controls
    /// Options that control what kinds of optimizations should be enabled.
    /// Mostly for debugging purposes
    pub opts: SearchOpts,
    //TODO: time control
}

impl Search {
    pub fn new(depth: usize) -> Self {
        Self {
            depth,
            best_moves: [Move::NULL; MAX_PLY],
            scores: [i32::default(); MAX_PLY],
            killers: [Killers::new(); MAX_PLY],
            eval: [0; MAX_PLY],
            nodes_visited: [0; MAX_PLY],
            tt_hits: 0,
            durations: [Duration::from_millis(0); MAX_PLY],
            beta_cutoffs: [0; MAX_PLY],
            opts: SearchOpts::new(),
        }

    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct SearchOpts {
    pub tt: bool,
    pub iterative: bool,
    pub ordering: bool,
    pub mvv_lva: bool,
    pub killers: bool,
}

impl SearchOpts {
    pub fn new() -> Self {
        Self {
            tt: true,
            iterative: true,
            ordering: true,
            mvv_lva: true,
            killers: true,
        }
    }
}

pub const DEFAULT_OPTS: SearchOpts = SearchOpts {
    tt: true,
    iterative: true,
    ordering: true,
    mvv_lva: true,
    killers: true,
};

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
        self.0.rotate_right(1);

        // Make sure we only add distinct moves
        if mv != self.0[0] {
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

pub type KillerTable =[Killers; MAX_DEPTH];

impl Position {
    pub fn search(&self, max_depth: usize, tt: &mut TTable, opts: SearchOpts) -> Search {
        let start_depth = if opts.iterative { 1 } else { max_depth };
        let mut search = Search::new(max_depth);

        for depth in start_depth..=max_depth {
            // Clear results before every search so the cumulative counts don't
            // include lower-ply results
            search = Search::new(depth);

            self.negamax(0, Score::MIN+1, Score::MAX, tt, &mut search);
        }

        search
    }

    fn negamax(
        &self, 
        ply: usize, 
        alpha: i32, 
        beta: i32, 
        tt: &mut TTable, 
        search: &mut Search
    ) -> i32 {
        let mut best_move = Move::NULL;
        let mut score = Score::MIN + 1;
        let mut node_type = NodeType::Exact;
        let tt_entry = tt.probe(self.hash);
        let remaining_depth = search.depth - ply;
        let eval = self.score.total();
        let start = std::time::Instant::now();

        // 1. Can we use an existing TT entry?
        if tt_entry.is_some() && tt_entry.unwrap().get_depth() >= remaining_depth {
            let tt_entry = tt_entry.unwrap();
            score = tt_entry.get_score();
            best_move = tt_entry.get_move();

            search.tt_hits += 1;

        } else 

        // 2. Is this a leaf node?
        if ply == search.depth {
            score = eval;

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
                    .negamax(ply + 1, -beta, -alpha, tt, search);

                if new_score > score {
                    score = new_score;
                    best_move = mv;
                    node_type = NodeType::Exact;
                }

                if new_score > alpha {
                    alpha = new_score;
                    node_type = NodeType::Upper;
                }

                if alpha >= beta {
                    node_type = NodeType::Lower;
                    search.beta_cutoffs[ply] += 1;

                    // Check that it isn't the tt_move, or a capture or a promotion...
                    if mv.is_quiet() {
                        search.killers[ply].add(mv);
                    }

                    break;
                }
            }
        }

        // Store this entry in the TT
        // TODO: Have more sensible eviction strategy than "just clear anything"
        // Easiest options: Deeper searches should get priority
        // Though, should that logic live here, or in `TTable::insert()` ?
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
        search.eval[ply] = eval;
        search.nodes_visited[ply] += 1;
        search.durations[ply] += start.elapsed();

        score
    }
}
