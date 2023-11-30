use std::{ops::Deref, time::Duration};

use chess::movegen::moves::Move;
use shared::uci::{Info, UciEngineMessage};
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


    pub fn as_uci(&self) -> String {
        let info = Info {
            depth: Some(self.depth as u8),
            seldepth: Some(self.depth as u8),
            score: Some(self.scores[0]),
            time: Some(self.duration.as_millis() as u64),
            nps: (1_000 * self.nodes_visited as u32).checked_div(self.duration.as_millis() as u32),
            nodes: Some(self.nodes_visited as u32),
            currmove: None,
            currmovenumber: None,
            hashfull: None
        };


        UciEngineMessage::Info(info).to_string()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct SearchOpts {
    pub tt: bool,
    pub ordering: bool,
    pub tt_move: bool,
    pub mvv_lva: bool,
    pub killers: bool,
    pub debug: bool,
}

impl SearchOpts {
    pub const ALL: Self = {
        Self {
            tt: true,
            tt_move: true,
            ordering: true,
            mvv_lva: true,
            killers: true,
            debug: true,
        }
    };

    #[allow(dead_code)]
    pub const NONE: Self = {
        Self {
            tt: false,
            tt_move: false,
            ordering: false,
            mvv_lva: false,
            killers: false,
            debug: false,
        }
    };
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
            self.negamax(0, Score::MIN, Score::MAX, tt, &mut search, &tc);
            search.duration = start.elapsed();

            // If we got interrupted in the search, don't store the 
            // half-completed search state. Just break and return the previous
            // iteration's search.
            if search.aborted {
                break;
            } else {
                result = search;
            }

            if opts.debug {
                println!("{info}", info = search.as_uci());

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
            return Score::MIN;
        }
        let mut best_move = Move::NULL;
        let mut best_score = Score::MIN + 1;
        let mut backup_move = Move::NULL;
        let mut node_type = NodeType::Upper;
        let mut alpha = alpha;
        let remaining_depth = search.depth - ply;

        let tt_entry = tt.probe(self.hash);

        let tt_usable = tt_entry.is_some_and(|entry| {
            let tt_depth = entry.get_depth();
            let tt_type = entry.get_type();
            let tt_score = entry.get_score();

            tt_depth >= remaining_depth && (
                tt_type == NodeType::Exact
                || tt_type == NodeType::Upper && tt_score <= alpha
                || tt_type == NodeType::Lower && tt_score >= beta
            )
        });

        // 1. Can we use an existing TT entry?
        if tt_usable {
            let tt_entry = tt_entry.unwrap();
            best_move = tt_entry.get_move();
            best_score = tt_entry.get_score();
            node_type = tt_entry.get_type();

            search.tt_hits += 1;
        } else if self.board.checkmate() {
            best_score = Score::MIN;
        } else if self.board.is_draw() {
            best_score = 0;
        } else 

        // 2. Is this a leaf node?
        if ply == search.depth {
            best_score = self.score.total();
            node_type = NodeType::Exact;
            search.leaf_nodes += 1;

        //3. Recurse over all the child nodes
        } else {
            let legal_moves = self.board.legal_moves();
            backup_move = legal_moves[0];

            let legal_moves = MovePicker::new(
                &self,  
                legal_moves,
                tt_entry.map(|entry| entry.get_move()),
                search.killers[ply],
                search.opts
            );

            for mv in legal_moves {
                if search.aborted {
                    return Score::MIN;
                }

                let score = -self
                    .play_move(mv)
                    .negamax(ply + 1, -beta, -alpha, tt, search, tc);


                if score > best_score {
                    best_score = score;
                    best_move = mv;
                }

                if alpha <= score {
                    alpha = score;
                    node_type = NodeType::Exact;
                }

                if beta <= score {
                    node_type = NodeType::Lower;
                    break;
                }
            }
        }

        // Additional bookkeeping for cutoffs
        if node_type == NodeType::Lower {
            // Increment the cutoff counter
            search.beta_cutoffs[ply] += 1;

            // If the move was quiet, store it as a killer move
            if best_move.is_quiet() && search.opts.killers { 
                search.killers[ply].add(best_move);
            }
        }

        // Fail-hard semantics: the score we return is clamped to the
        // `alpha`-`beta` window. (Note that, if we increased alpha in this node,
        // then returning `alpha` amounts to returning the actual score.
        let score = match node_type {
            NodeType::Upper => alpha,
            NodeType::Exact => best_score,
            NodeType::Lower => beta,
        };

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

        let best_move = if best_move != Move::NULL { best_move } else { backup_move };

        // Propagate up the results
        search.best_moves[ply] = best_move;
        search.scores[ply] = score;
        search.nodes_visited += 1;

        score
    }
}
