use chess::movegen::moves::Move;
use crate::{evaluate::Score, position::Position};

impl Position {
    pub fn search(&self, depth: usize) -> SearchResult {
        self.negamax(depth, Score::MIN+1, Score::MAX)
    }

    fn negamax(&self, depth: usize, alpha: i32, beta: i32) -> SearchResult {
        let mut alpha = alpha;
        let mut nodes_visited = 0;
        let mut checkmates = 0;
        let mut best_move = Move::NULL;
        let mut score = Score::MIN + 1;

        if depth == 0 {
            return SearchResult {
                best_move,
                nodes_visited: 1,
                checkmates: 0,
                score: self.score.total(),
            }
        }

        let legal_moves = self.board.legal_moves();

        if legal_moves.len() == 0 {
            if self.board.compute_checkers(self.board.current.opp()).is_empty() {
                // Checkmate
                return SearchResult {
                    best_move,
                    nodes_visited: 1,
                    checkmates: 1,
                    score: Score::MIN,
                }
            } else {
                //Stalemate
                return SearchResult {
                    best_move,
                    nodes_visited: 1,
                    checkmates: 0,
                    score: 0
                }
            }
        }

        for mv in legal_moves {
            let result = self.play_move(mv).negamax(depth - 1, -beta, -alpha);

            // Negamax negation of the child nodes' score
            let corrected_score = -result.score;

            checkmates += result.checkmates;
            nodes_visited += result.nodes_visited;

            // If the returned score for this moves the score we currently 
            // have, update the current score and best move to be this score 
            // and move
            if corrected_score > score {
                score = corrected_score;
                best_move = mv;
            }

            if corrected_score > alpha {
                alpha = corrected_score;
            }

            if corrected_score >= beta {
                break;
            }
        }

        SearchResult {
            best_move,
            score,
            nodes_visited,
            checkmates
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct SearchResult {
    pub best_move: Move,
    pub nodes_visited: usize,
    pub checkmates: usize,
    pub score: i32,
}
