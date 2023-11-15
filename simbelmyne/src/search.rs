use chess::movegen::moves::Move;
use crate::{evaluate::Score, position::Position, transpositions::{TTable, TTEntry, NodeType}};

impl Position {
    pub fn search(&self, depth: usize, tt: &mut TTable) -> SearchResult {
        let mut result = SearchResult::default();
        self.negamax(depth, Score::MIN+1, Score::MAX, tt, &mut result);

        result
    }

    fn negamax(
        &self, 
        depth: usize, 
        alpha: i32, 
        beta: i32, 
        tt: &mut TTable, 
        result: &mut SearchResult
    ) -> i32 {
        let mut best_move = Move::NULL;
        let mut score = Score::MIN + 1;
        let mut node_type = NodeType::Exact;
        let tt_entry = tt.probe(self.hash);

        // 1. Can we use an existing TT entry?
        if tt_entry.is_some() && tt_entry.unwrap().get_depth() >= depth {
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

            for mv in self.board.legal_moves() {
                let new_score = -self
                    .play_move(mv)
                    .negamax(depth - 1, -beta, -alpha, tt, result);

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
}

