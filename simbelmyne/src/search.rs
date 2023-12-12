use crate::search_tables::HistoryTable;
use crate::search_tables::Killers;
use crate::search_tables::PVTable;
use std::time::Duration;

use chess::movegen::moves::Move;
use shared::uci::SearchInfo;
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

const QUIETS: bool = true;
const TACTICALS: bool = false;

/// A Search struct holds both the parameters, as well as metrics and results, 
/// for a given search.
#[derive(Debug, Clone)]
pub struct Search {
    // The (nominal) depth of this search. This does not take QSearch into 
    // consideration
    pub depth: usize,

    // The so-called "selective depth", the deepest ply we've searched
    pub seldepth: usize,

    // The time control for the search
    pub tc: TimeControl,

    /// The set of killer moves at a given ply.
    pub killers: [Killers; MAX_DEPTH],

    /// History heuristic table
    pub history_table: HistoryTable,
}

impl Search {
    pub fn new(depth: usize, tc: TimeControl) -> Self {
        Self {
            depth,
            seldepth: 0,
            tc,
            killers: [Killers::new(); MAX_DEPTH],
            history_table: HistoryTable::new(),
        }
    }

    pub fn should_continue(&self) -> bool {
        self.tc.should_continue(self.depth)
    }
}


#[derive(Debug, Clone)]
pub struct SearchReport {
    pub depth: u8,
    pub seldepth: u8,
    pub nodes: u32,
    pub duration: Duration,
    pub score: Eval,
    pub pv: Vec<Move>,
    pub hashfull: u32,
}

impl SearchReport {
    pub fn new(search: &Search, tt: &TTable, pv: PVTable, score: Eval) -> Self {
        Self {
            score,
            depth: search.depth as u8,
            seldepth: search.seldepth as u8,
            nodes: search.tc.nodes(),
            duration: search.tc.elapsed(),
            pv: pv.into(),
            hashfull: (1000.0 * tt.occupancy()) as u32,
        }
    }

    pub fn default() -> Self {
        Self {
            depth: 0,
            seldepth: 0,
            nodes: 0,
            duration: Duration::ZERO,
            score: 0,
            pv: Vec::new(),
            hashfull: 0,
        }
    }
}

impl From<&SearchReport> for SearchInfo {
    fn from(report: &SearchReport) -> Self {
        let nps = 1000 * report.nodes
            .checked_div(report.duration.as_millis() as u32)
            .unwrap_or_default();

        Self {
            depth: Some(report.depth),
            seldepth: Some(report.seldepth),
            time: Some(report.duration.as_millis() as u64),
            nodes: Some(report.nodes),
            score: Some(report.score),
            pv: report.pv.clone(),
            hashfull: Some(report.hashfull),
            nps: Some(nps),
            currmove: None,
            currmovenumber: None,
        }
    }
}

impl Position {
    pub fn search(&self, tt: &mut TTable, tc: TimeControl) -> SearchReport {
        let mut depth = 0;
        let mut latest_report = SearchReport::default();

        while depth < MAX_DEPTH && tc.should_continue(depth) {
            depth += 1;

            let mut pv = PVTable::new();
            let mut search = Search::new(depth, tc.clone());

            let score = self.negamax(0, depth, Score::MIN, Score::MAX, tt, &mut pv, &mut search, false);

            // If we got interrupted in the search, don't store the 
            // half-completed search state. Just break and return the previous
            // iteration's search.
            if !search.should_continue() {
                break;
            }

            latest_report = SearchReport::new(&search, tt, pv, score);

            if DEBUG {
                println!("{info}", info = UciEngineMessage::Info((&latest_report).into()));
            }
        }

        latest_report
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
        let in_root = ply == 0;
        search.tc.add_node();
        let mut local_pv = PVTable::new();
        pv.clear();

        // Do all the static evaluations first
        // That is, Check whether we can/should assign a score to this node
        // without recursing any deeper.

        // Rule-based draw? Don't return early when in the root node, because 
        // we won't have a PV move to play.
        if (self.board.is_rule_draw() || self.is_repetition()) && !in_root {
            return Score::DRAW;
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
        if !in_root {
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
            && !in_root
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
            self.board.legal_moves::<QUIETS>(),
            tt_entry.map(|entry| entry.get_move()),
            search.killers[ply],
            search.history_table,
        );

        // Checkmate?
        if legal_moves.len() == 0 && in_check {
            return -Score::MATE + ply as Eval;
        }

        // Stalemate
        if legal_moves.len() == 0 && !in_check {
            return Score::DRAW;
        }

        for mv in &mut legal_moves {
            if !search.should_continue() {
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

        search.tc.add_node();
        search.seldepth = search.seldepth.max(ply);

        if self.board.is_rule_draw() || self.is_repetition() {
            return Score::DRAW
        }

        let mut local_pv = PVTable::new();
        let eval = self.score.total();

        if ply >= MAX_DEPTH {
            return eval;
        }

        if eval >= beta {
            return beta
        }

        if alpha < eval {
            alpha = eval;
        }

        let tacticals = MovePicker::new(
            &self,
            self.board.legal_moves::<TACTICALS>(),
            None,
            Killers::new(),
            search.history_table,
        );

        let in_check = self.board.in_check();

        // Checkmate?
        if tacticals.len() == 0 && in_check {
            return -Score::MATE + ply as Eval;
        }

        // Stalemate
        if tacticals.len() == 0 && !in_check {
            return Score::DRAW;
        }

        for mv in tacticals {
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
