use crate::search_tables::HistoryTable;
use std::time::Duration;

use chess::movegen::moves::Move;
use shared::uci::Info;
use shared::uci::UciEngineMessage;
use crate::evaluate::Score;
use crate::transpositions::NodeType;
use crate::transpositions::TTEntry;
use crate::transpositions::TTable;
use crate::time_control::TimeControl;
use crate::move_picker::MovePicker;
use crate::position::Position;
use crate::evaluate::Eval;

pub const MAX_DEPTH : usize = 128;
const NULL_MOVE_REDUCTION: usize = 3;

/// A Search struct holds both the parameters, as well as metrics and results, for a given search.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Search {
    // Information
    pub depth: usize,
    
    // Search results
    /// Best move found at each ply of the search
    pub best_moves: [Move; MAX_DEPTH],

    /// The scores found for said best move, at each ply of the search
    pub scores: [Eval; MAX_DEPTH],

    /// The set of killer moves at a given ply.
    /// Killer moves are quiet moves (non-captures/promotions) that caused a 
    /// beta-cutoff in that ply.
    pub killers: [Killers; MAX_DEPTH],

    /// History heuristic table
    pub history_table: HistoryTable,

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
            scores: [Eval::default(); MAX_DEPTH],
            killers: [Killers::new(); MAX_DEPTH],
            history_table: HistoryTable::new(),
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
        self.to_string()
    }
}

impl From<Search> for UciEngineMessage {
    fn from(value: Search) -> Self {
        let info = Info {
            depth: Some(value.depth as u8),
            seldepth: Some(value.depth as u8),
            score: Some(value.scores[0]),
            time: Some(value.duration.as_millis() as u64),
            nps: (1_000 * value.nodes_visited as u32).checked_div(value.duration.as_millis() as u32),
            nodes: Some(value.nodes_visited as u32),
            currmove: None,
            currmovenumber: None,
            hashfull: None
        };

        UciEngineMessage::Info(info)
    }
}

impl ToString for Search {
    fn to_string(&self) -> String {
        <Search as Into<UciEngineMessage>>::into(*self).to_string()
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


impl Position {
    pub fn search(&self, tt: &mut TTable, opts: SearchOpts, tc: TimeControl) -> Search {
        let mut result: Search = Search::new(0, opts);
        let mut depth = 0;

        while depth < MAX_DEPTH && tc.should_continue(depth, result.nodes_visited) {
            depth += 1;
            let mut search = Search::new(depth, opts);

            self.negamax(0, Score::MIN, Score::MAX, tt, &mut search, &tc, true);
            search.duration = tc.elapsed();

            // If we got interrupted in the search, don't store the 
            // half-completed search state. Just break and return the previous
            // iteration's search.
            if search.aborted {
                break;
            } else {
                result = search;
            }

            if opts.debug {
                println!("{info}", info = UciEngineMessage::from(search));
            }
        }

        result
    }

    fn negamax(
        &self, 
        ply: usize, 
        alpha: Eval, 
        beta: Eval, 
        tt: &mut TTable, 
        search: &mut Search,
        tc: &TimeControl,
        try_nmp: bool,
    ) -> Eval {
        if !tc.should_continue(search.depth, search.nodes_visited) {
            search.aborted = true;
            return Score::MIN;
        }
        let mut best_move = Move::NULL;
        let mut best_score = Score::MIN + 1;
        let mut node_type = NodeType::Upper;
        let mut alpha = alpha;
        let remaining_depth = search.depth - ply;
        let tt_entry = tt.probe(self.hash);
        let in_check = self.board.in_check();
        search.nodes_visited += 1;

        if ply >= MAX_DEPTH {
            let score = self.score.total();

            if score <= alpha {
                return alpha;
            } else if score > beta {
                return beta;
            } else {
                return score;
            }
        }

        // Do all the static evaluations first
        // That is, Check whether we can/should assign a score to this node
        // without recursing any deeper.
        
        // Check the TT table for a result that we can use
        if let Some((best_move, score)) = tt_entry.and_then(|entry| entry.try_use(remaining_depth, alpha, beta)) {
            search.best_moves[ply] = best_move;
            search.scores[ply] = score;
            if node_type == NodeType::Lower 
                && best_move.is_quiet() 
                && search.opts.killers { 
                search.killers[ply].add(best_move);
            }
            return score;
        }

        // If we're in a leaf node, extend with a quiescence search
        if remaining_depth == 0 {
            return self.quiescence_search(ply, alpha, beta, search, tc);
        }

        // Rule-based draw?
        if self.board.is_rule_draw() {
            return 0;
        }

        // Null move pruning
        let should_null_prune = try_nmp
            && ply > 0
            && !in_check
            && remaining_depth >= NULL_MOVE_REDUCTION + 1;

        if should_null_prune {
            let score = -self
                .play_move(Move::NULL)
                .negamax(ply + 1 + NULL_MOVE_REDUCTION, -beta, -beta + 1, tt, search, tc, false);

            if score >= beta {
                return beta;
            }
        }

        // Recurse over all the legal moves and recursively search the 
        // resulting positions
        let mut legal_moves = MovePicker::new(
            &self,  
            self.board.legal_moves(),
            tt_entry.map(|entry| entry.get_move()),
            search.killers[ply],
            search.history_table,
            search.opts
        );

        // Checkmate?
        if in_check && legal_moves.len() == 0 {
            return Score::MIN;
        }

        // Stalemate
        if !in_check && legal_moves.len() == 0 {
            return 0;
        }

        for mv in &mut legal_moves {
            if search.aborted {
                return Score::MIN;
            }

            let score = -self
                .play_move(mv)
                .negamax(ply + 1, -beta, -alpha, tt, search, tc, true);

            if score > best_score {
                best_score = score;
                best_move = mv;
            }

            if alpha <= score {
                alpha = score;
                node_type = NodeType::Exact;
            }

            if beta <= score {
                let piece = self.board.get_at(mv.src()).unwrap();
                search.history_table.increment(&mv, piece, remaining_depth);
                node_type = NodeType::Lower;
                break;
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

        // If, for some reason, we haven't actually updated the best move at this
        // point (i.e., it's still an uninitialized NULL move, then choose the
        // first legal move as a backup move;
        if best_move == Move::NULL {
            best_move = legal_moves.get_first()
        }

        // Propagate up the results
        search.best_moves[ply] = best_move;
        search.scores[ply] = score;
        if node_type == NodeType::Lower 
            && best_move.is_quiet() 
            && search.opts.killers { 
            search.killers[ply].add(best_move);
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

        score
    }

    fn quiescence_search(
        &self, 
        ply: usize,
        mut alpha: Eval, 
        beta: Eval, 
        search: &mut Search,
        tc: &TimeControl,
    ) -> Eval {
        search.nodes_visited += 1;
        let eval = self.score.total();

        if eval >= beta {
            return beta
        }

        if eval > alpha {
            alpha = eval;
        }

        if self.board.is_rule_draw() {
            return 0;
        }

        if ply >= MAX_DEPTH {
            return alpha;
        }

        let legal_moves = MovePicker::new(
            &self,
            self.board.legal_moves(),
            None,
            Killers::new(),
            search.history_table,
            search.opts
        );

        for mv in legal_moves.captures() {
            if search.aborted {
                return Score::MIN;
            }

            let score = -self
                .play_move(mv)
                .quiescence_search(ply + 1, -beta, -alpha, search, tc);

            if score >= beta {
                return beta;
            }

            if score > alpha {
                alpha = score;
            }
        }

        alpha
    }
}
