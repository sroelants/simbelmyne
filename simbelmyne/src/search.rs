use chess::movegen::moves::Move;
use crate::{evaluate::Score, position::Position, transpositions::{TTable, TTEntry, NodeType}};

impl Position {
    pub fn search(&self, depth: usize, tt: &mut TTable) -> SearchResult {
        self.negamax(depth, Score::MIN+1, Score::MAX, tt)
    }

    fn negamax(&self, depth: usize, alpha: i32, beta: i32, tt: &mut TTable) -> SearchResult {
        let mut alpha = alpha;
        let mut nodes_visited = 1;
        let mut checkmates = 0;
        let mut best_move = Move::NULL;
        let mut score = Score::MIN + 1;
        let mut node_type = NodeType::Exact;
        let mut tt_hits: usize = 0;

        if let Some(tt_entry) = tt.probe(self.hash) {
            if tt_entry.get_depth() == depth {
                return SearchResult {
                    checkmates: 0,
                    nodes_visited:  tt_entry.get_nodes_visited(),
                    score: tt_entry.get_score(),
                    best_move: tt_entry.get_move(),
                    tt_hits: 1,
                };
            }
        }

        if depth == 0 {
            score = self.score.total();

            tt.insert(TTEntry::new(
                self.hash,
                best_move,
                score,
                depth,
                node_type,
                nodes_visited,
            ));

            return SearchResult {
                best_move,
                nodes_visited,
                checkmates,
                score,
                tt_hits,
            }
        }

        let legal_moves = self.board.legal_moves();

        // if legal_moves.len() == 0 {
        //     if self.board.compute_checkers(self.board.current.opp()).is_empty() {
        //         // Checkmate
        //         return SearchResult {
        //             best_move,
        //             nodes_visited: 1,
        //             checkmates: 1,
        //             score: Score::MIN,
        //         }
        //     } else {
        //         //Stalemate
        //         return SearchResult {
        //             best_move,
        //             nodes_visited: 1,
        //             checkmates: 0,
        //             score: 0
        //         }
        //     }
        // }


        for mv in legal_moves {
            let result = self.play_move(mv).negamax(depth - 1, -beta, -alpha, tt);

            // Negamax negation of the child nodes' score
            let corrected_score = -result.score;

            checkmates += result.checkmates;
            nodes_visited += result.nodes_visited;
            tt_hits += result.tt_hits;

            // If the returned score for this moves the score we currently 
            // have, update the current score and best move to be this score 
            // and move
            if corrected_score > score {
                score = corrected_score;
                best_move = mv;
                node_type = NodeType::Exact;
            }

            if corrected_score > alpha {
                alpha = corrected_score;
                node_type = NodeType::Upper;
            }

            if alpha >= beta {
                node_type = NodeType::Lower;
                break;
            }
        }

        tt.insert(TTEntry::new(
            self.hash,
            best_move,
            score,
            depth,
            node_type,
            nodes_visited,
        ));

        SearchResult {
            best_move,
            score,
            nodes_visited,
            checkmates,
            tt_hits,
        }
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

