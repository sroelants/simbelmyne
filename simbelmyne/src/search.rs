use chess::board::Board;
use chess::movegen::moves::Move;
use crate::evaluate::Score;


#[derive(Debug, Copy, Clone)]
pub(crate) struct BoardState {
    pub(crate) board: Board,
    pub(crate) score: Score,
}

impl BoardState {
    pub fn new(board: Board) -> Self {
        BoardState {
            board, 
            score: Score::new(&board),
        }
    }

    pub fn play_move(&self, mv: Move) -> Self {
        let us = self.board.current;
        
        // Update board
        let new_board = self.board.play_move(mv);
        let mut new_score= self.score.clone();

        // Remove piece from score
        let old_piece = self.board.piece_list[mv.src() as usize]
            .expect("The source target of a move has a piece");

        new_score.remove(us, old_piece.piece_type(), old_piece.color(), mv.src());

        // Add back value of new position. This takes care of promotions too
        let new_piece = new_board.piece_list[mv.tgt() as usize]
          .expect("The target square of a move is occupied after playing");

        new_score.add(us, new_piece.piece_type(), new_piece.color(), mv.tgt());

        // If capture: remove value
        if mv.is_capture() {
            if mv.is_en_passant() {
                let ep_sq = mv.tgt().backward(old_piece.color()).unwrap();

                let captured = self.board.get_at(ep_sq)
                    .expect("A capture has a piece on the tgt square before playing");

                new_score.remove(
                    us,
                    captured.piece_type(), 
                    captured.color(), 
                    ep_sq
                );
            } else {
                let captured = self.board.get_at(mv.tgt())
                    .expect("A capture has a piece on the tgt square before playing");

                new_score.remove(
                    us,
                    captured.piece_type(), 
                    captured.color(), 
                    mv.tgt()
                );
            }
        }

        Self {
            board: new_board,
            score: new_score.flipped()
        }
    }

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
