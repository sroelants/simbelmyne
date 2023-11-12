use chess::board::Board;
use chess::movegen::moves::Move;
use crate::evaluate::Score;


#[derive(Debug, Copy, Clone)]
pub(crate) struct BoardState {
    pub(crate) board: Board,
    pub(crate) score: Score,
}

impl BoardState {
    pub fn new(board: Board) -> BoardState {
        BoardState {
            board, 
            score: Score::new(&board),
        }
    }

    pub fn play_move(&self, mv: Move) -> BoardState {
        let mut new = self.to_owned();
        
        // Update board
        new.board = self.board.play_move(mv);

        // Remove piece from score
        let piece = self.board.piece_list[mv.src() as usize]
            .expect("The source target of a move has a piece");

        new.score.remove(piece.piece_type(), piece.color(), mv.src());

        // If capture: remove value
        if mv.is_capture() {
            let captured = self.board.piece_list[mv.tgt() as usize]
                .expect("A capture has a piece on the tgt square");

            if mv.is_en_passant() {
                new.score.remove(
                    captured.piece_type(), 
                    captured.color(), 
                    mv.tgt().backward(piece.color()).unwrap() // En-passant can't go off the board
                );
            } else {
                new.score.remove(
                    captured.piece_type(), 
                    captured.color(), 
                    mv.tgt()
                );
            }

            // Add back value of new position
            if mv.is_promotion() {
                let ptype = mv.get_promo_type().unwrap(); // we know it's a promotion
                
                new.score.add(ptype, piece.color(), mv.tgt());
            } else {
                new.score.add(piece.piece_type(), piece.color(), mv.tgt());
            }
        }

        new
    }

    pub fn search(&self, depth: usize) -> SearchResult {
        self.negamax(depth, Score::MIN+1, Score::MAX)
    }

    // pub fn best_move(&self, depth: usize) -> Move {
    //     self.search(depth).best_move
    // }

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
                score: self.score.score(),
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
