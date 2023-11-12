use std::fmt::Display;

use chess::{board::Board, movegen::moves::Move};
use colored::Colorize;
use crate::evaluate::Eval;

pub type Score = i32;

pub(crate) struct BoardState {
    pub(crate) board: Board,
    pub(crate) score: Score
}

impl BoardState {
    pub fn new(board: Board) -> BoardState {
        BoardState { 
            board, 
            score: 0 //board.eval(),
        }
    }

    pub fn play_move(&self, mv: Move) -> BoardState {
        let new_board = self.board.play_move(mv);
        BoardState {
            board: new_board,
            score: 0 //new_board.eval()
        }

    }

    pub fn search(&self, depth: usize) -> SearchResult {
        self.negamax(depth, Score::MIN+1, Score::MAX)
    }

    pub fn best_move(&self, depth: usize) -> Move {
        self.search(depth).best_move
    }

    fn negamax(&self, depth: usize, alpha: Score, beta: Score) -> SearchResult {
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
                score: self.board.eval()
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

#[derive(Debug, Copy, Clone)]
pub struct SearchResult {
    pub best_move: Move,
    pub score: Score,
    pub nodes_visited: usize,
    pub checkmates: usize
}
