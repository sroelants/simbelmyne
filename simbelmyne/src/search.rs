use crate::search_tables::HistoryTable;
use crate::search_tables::Killers;
use crate::search_tables::PVTable;
use std::time::Duration;

use chess::movegen::moves::Move;
use shared::uci::SearchReport;
use shared::uci::UciEngineMessage;
use crate::evaluate::Score;
use crate::transpositions::NodeType;
use crate::transpositions::TTEntry;
use crate::transpositions::TTable;
use crate::time_control::TimeControl;
use crate::move_picker::MovePicker;
use crate::position::Position;
use crate::evaluate::Eval;

// Search parameters
pub const MAX_DEPTH           : usize = 128;
pub const NULL_MOVE_REDUCTION : usize = 3;

// Search options
pub const USE_TT        : bool = true;
pub const MOVE_ORDERING : bool = true;
pub const TT_MOVE       : bool = true;
pub const MVV_LVA       : bool = true;
pub const KILLER_MOVES  : bool = true;
pub const HISTORY_TABLE : bool = true;
pub const DEBUG         : bool = true;

/// A Search struct holds both the parameters, as well as metrics and results, 
/// for a given search.
#[derive(Debug, Clone)]
pub struct Search {
    // The (nominal) depth of this search. This does not take QSearch into 
    // consideration
    pub depth: usize,

    // The so-called "selective depth", the deepest ply we've searched
    pub seldepth: usize,

    // The principal variation so far
    pub pv: PVTable,

    // The time control for the search
    pub tc: TimeControl,

    // The score for the search
    pub score: Eval,
    
    /// The set of killer moves at a given ply.
    pub killers: [Killers; MAX_DEPTH],

    /// History heuristic table
    pub history_table: HistoryTable,

    // Stats
    /// The total number of nodes visited in this search
    pub nodes_visited: usize,

    /// The time the search took at any given ply
    pub duration: Duration,
}

impl Search {
    pub fn new(depth: usize, tc: TimeControl) -> Self {
        Self {
            depth,
            seldepth: 0,
            pv: PVTable::new(),
            tc,
            score: 0,
            killers: [Killers::new(); MAX_DEPTH],
            history_table: HistoryTable::new(),
            nodes_visited: 0,
            duration: Duration::default(),
        }
    }

    pub fn reset(&mut self) {
        self.seldepth = 0;
        self.pv = PVTable::new();
        self.score = 0;
        self.killers = [Killers::new(); MAX_DEPTH];
        self.history_table = HistoryTable::new();
        self.duration = Duration::default();
    }

    pub fn should_continue(&self) -> bool {
        self.tc.should_continue(self.depth, self.nodes_visited)
    }

    pub fn as_uci(&self) -> String {
        self.to_string()
    }

    pub fn report(&self) -> SearchReport {
        let nps = (self.nodes_visited as u32)
            .checked_div(self.duration.as_millis() as u32);

        SearchReport {
            depth: Some(self.depth as u8),
            seldepth: Some(self.seldepth as u8),
            score: Some(self.score),
            time: Some(self.duration.as_millis() as u64),
            nps,
            nodes: Some(self.nodes_visited as u32),
            currmove: None,
            currmovenumber: None,
            hashfull: None,
            pv: self.pv.into()
        }
    }
}

impl From<&Search> for UciEngineMessage {
    fn from(search: &Search) -> Self {
        UciEngineMessage::Info(search.report())
    }
}

impl ToString for Search {
    fn to_string(&self) -> String {
        <&Search as Into<UciEngineMessage>>::into(self).to_string()
    }
}

impl Position {
    pub fn search(&self, tt: &mut TTable, tc: TimeControl) -> SearchReport {
        let mut depth = 0;
        let mut latest_search = Search::new(0, tc);

        while depth < MAX_DEPTH && latest_search.tc.should_continue(depth, latest_search.nodes_visited) {
            depth += 1;

            let mut pv = PVTable::new();
            let mut search = latest_search.clone();
            search.reset();

            search.score = self.negamax(0, depth, Score::MIN, Score::MAX, tt, &mut pv, &mut search, false);
            search.duration = search.tc.elapsed();
            search.pv = pv;
            search.depth = depth;

            // If we got interrupted in the search, don't store the 
            // half-completed search state. Just break and return the previous
            // iteration's search.
            if search.tc.stopped() {
                break;
            } else {
                if DEBUG {
                    println!("{info}", info = UciEngineMessage::from(&search));
                }

                latest_search = search;
            }

        }

        latest_search.report()
    }

    fn negamax(
        &self, 
        ply: usize, 
        mut depth: usize,
        alpha: Eval, 
        beta: Eval, 
        tt: &mut TTable, 
        pv: &mut PVTable,
        search: &mut Search,
        try_null: bool,
    ) -> Eval {
        if !search.should_continue() {
            return Score::MIN;
        }

        let mut best_move = Move::NULL;
        let mut best_score = Score::MIN;
        let mut node_type = NodeType::Upper;
        let mut alpha = alpha;
        let tt_entry = tt.probe(self.hash);
        let in_check = self.board.in_check();
        search.nodes_visited += 1;
        let mut local_pv = PVTable::new();
        pv.clear();

        // Do all the static evaluations first
        // That is, Check whether we can/should assign a score to this node
        // without recursing any deeper.

        // Rule-based draw?
        if self.board.is_rule_draw() || self.is_repetition() {
            return 0;
        }

        if ply >= MAX_DEPTH {
            return self.score.total();
        }

        if in_check {
            depth += 1;
        }

        // If we're in a leaf node, extend with a quiescence search
        if depth == 0 {
            return self.quiescence_search(ply, alpha, beta, pv, search);
        }

        // Check the TT table for a result that we can use
        if ply > 0 {
            if let Some((best_move, score)) = tt_entry.and_then(|entry| entry.try_use(depth, alpha, beta)) {
                if node_type == NodeType::Lower 
                    && best_move.is_quiet() 
                    && KILLER_MOVES { 
                    search.killers[ply].add(best_move);
                }

                return score;
            }
        }

        // Null move pruning
        let should_null_prune = try_null
            && ply > 0
            && !in_check
            && depth >= NULL_MOVE_REDUCTION + 1;

        if should_null_prune {
            let score = -self
                .play_move(Move::NULL)
                .negamax(
                    ply + 1, 
                    depth - 1 - NULL_MOVE_REDUCTION, 
                    -beta, 
                    -beta + 1, 
                    tt, 
                    &mut PVTable::new(), 
                    search, 
                    false
                );

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
        );

        // Checkmate?
        if legal_moves.len() == 0 && in_check {
            return Score::MIN + ply as i32;
        }

        // Stalemate
        if legal_moves.len() == 0 && !in_check {
            return 0;
        }

        for mv in &mut legal_moves {
            if search.tc.stopped() {
                return Score::MIN;
            }

            let score = -self
                .play_move(mv)
                .negamax(ply + 1, depth - 1, -beta, -alpha, tt, &mut local_pv, search, true);

            if score > best_score {
                best_score = score;
                best_move = mv;
            }

            if alpha < score {
                alpha = score;
                node_type = NodeType::Exact;
                pv.add_to_front(mv, &local_pv);
            }

            if beta <= score {
                let piece = self.board.get_at(mv.src()).unwrap();
                search.history_table.increment(&mv, piece, depth);
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

        if node_type == NodeType::Lower 
            && best_move.is_quiet() 
            && KILLER_MOVES { 
            search.killers[ply].add(best_move);
        }

        // Store this entry in the TT
        if USE_TT {
            tt.insert(TTEntry::new(
                self.hash,
                best_move,
                score,
                depth,
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
        pv: &mut PVTable,
        search: &mut Search,
    ) -> Eval {
        if !search.should_continue() {
            return Score::MIN;
        }

        search.nodes_visited += 1;
        search.seldepth = search.seldepth.max(ply);

        let mut local_pv = PVTable::new();

        if self.board.is_rule_draw() || self.is_repetition() {
            return 0;
        }

        if ply >= MAX_DEPTH {
            return self.score.total();
        }

        let eval = self.score.total();

        if eval >= beta {
            return beta
        }

        if alpha < eval {
            alpha = eval;
        }

        let legal_moves = MovePicker::new(
            &self,
            self.board.legal_moves(),
            None,
            Killers::new(),
            search.history_table,
        );

        for mv in legal_moves.captures() {
            if search.tc.stopped() {
                return Score::MIN;
            }

            let score = -self
                .play_move(mv)
                .quiescence_search(ply + 1, -beta, -alpha, &mut local_pv , search);

            if score >= beta {
                return beta;
            }

            if alpha < score {
                alpha = score;
                pv.add_to_front(mv, &local_pv);
            }
        }

        alpha
    }
}
