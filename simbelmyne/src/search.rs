use std::ops::Deref;

use chess::movegen::moves::Move;
use crate::{evaluate::Score, position::Position, transpositions::{TTable, TTEntry, NodeType}, move_picker::MovePicker};

const MAX_DEPTH: usize = 50;
const MAX_KILLERS: usize = 2;

#[derive(Debug, Copy, Clone)]
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
    pub fn search(&self, max_depth: usize, tt: &mut TTable) -> SearchResult {
        let mut result = SearchResult::default();
        let mut killer_moves: KillerTable = [Killers::new(); MAX_DEPTH];

        for depth in 1..=max_depth {
            // Clear results before every search so the cumulative counts don't
            // include lower-ply results
            result = SearchResult::default();
            self.negamax(depth, Score::MIN+1, Score::MAX, tt, &mut result, &mut killer_moves);
        }

        result
    }

    fn negamax(
        &self, 
        depth: usize, 
        alpha: i32, 
        beta: i32, 
        tt: &mut TTable, 
        result: &mut SearchResult,
        killer_moves: &mut KillerTable
    ) -> i32 {
        let mut best_move = Move::NULL;
        let mut score = Score::MIN + 1;
        let mut node_type = NodeType::Exact;
        let tt_entry = tt.probe(self.hash);

        // 1. Can we use an existing TT entry?
        if tt_entry.is_some() && tt_entry.unwrap().get_depth() == depth {
            let tt_entry = tt_entry.unwrap();
            score = tt_entry.get_score();
            best_move = tt_entry.get_move();

            result.tt_hits += 1;

        // 2. Is this a leaf node?
        } else if depth == 0 {
            score = self.score.total();

        //3. Recurse over all the child nodes
        } else {
            let mut alpha = alpha;

            let legal_moves = self.board.legal_moves();
            let legal_moves = MovePicker::new(
                &self,  
                legal_moves, 
                tt_entry.map(|entry| entry.get_move()),
                killer_moves[depth]
            );

            for mv in legal_moves {
                let new_score = -self
                    .play_move(mv)
                    .negamax(depth - 1, -beta, -alpha, tt, result, killer_moves);

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
                    result.beta_cutoffs += 1;

                    // Check that it isn't the tt_move, or a capture or a promotion...
                    if !mv.is_capture() 
                    && !mv.is_promotion() {
                        killer_moves[depth].add(mv);
                    }
                    break;
                }
            }
        }

        // Store this entry in the TT
        // TODO: Have more sensible eviction strategy than "just clear anything"
        // Easiest options: Deeper searches should get priority
        // Though, should that logic live here, or in `TTable::insert()` ?
        tt.insert(TTEntry::new(
            self.hash,
            best_move,
            score,
            depth,
            node_type,
        ));

        // Propagate up the results
        result.best_move = best_move;
        result.score = score;
        result.nodes_visited += 1;

        score
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct SearchResult {
    pub best_move: Move,
    pub nodes_visited: usize,
    pub checkmates: usize,
    pub score: i32,
    pub tt_hits: usize,
    pub beta_cutoffs: usize,
}

