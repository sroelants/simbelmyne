use chess::{movegen::moves::Move, piece::PieceType};

use crate::{position::Position, search::Killers};
use crate::search::SearchOpts;

#[rustfmt::skip]
const VICTIM_VALS: [i32; PieceType::COUNT] = 
    // Pawn, Knight, Bishop, Rook, Queen, King
    [  1000,  3000,    3000,    5000,  9000,   500000];

#[rustfmt::skip]
const ATTACKER_VALS: [i32; PieceType::COUNT] = 
    // Pawn, Knight, Bishop, Rook, Queen, King
    [  100,  300,    300,    500,  900,   500];

const TACTICAL_OFFSET: i32 = 1000000;

const KILLER_BONUS: i32 = 10000;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum Stage {
    TTMove,
    ScoreTacticals,
    Tacticals,
    ScoreQuiets,
    Quiets,
}

pub struct MovePicker<'a> {
    stage: Stage,
    position: &'a Position,
    moves: Vec<Move>,
    scores: Vec<i32>,
    tt_move: Option<Move>,
    index: usize,
    quiet_index: usize,
    killers: Killers,
    opts: SearchOpts
}

impl<'a> MovePicker<'a> {
    pub fn new(
        position: &'a Position, 
        moves: Vec<Move>, 
        tt_move: Option<Move>,
        killers: Killers,
        opts: SearchOpts
    ) -> MovePicker {
        let mut scores = Vec::new();
        scores.resize_with(moves.len(), i32::default);

        MovePicker {
            stage: Stage::TTMove,
            position,
            scores,
            moves,
            tt_move,
            index: 0,
            quiet_index: 0,
            killers,
            opts,
        }
    }

    /// Search the move list starting at `start`, up until `end`, exclusive, and
    /// swap the first move that satisfies the predicate with the element at 
    /// `start`.
    pub fn find_swap<T: Fn(Move) -> bool>(&mut self, start: usize, end: usize, pred: T) -> Option<Move> {
        for i in start..end {
            if pred(self.moves[i]) {
                let found = self.moves[i];
                self.moves.swap(start, i); 
                return Some(found)
            }
        }

        None
    }

    /// Do a pass over all the moves, starting at the `start` up 
    /// till `end` (exclusive). Find the largest scoring move, and swap it 
    /// to `start`, then return it.
    pub fn partial_sort(&mut self,start: usize, end: usize) -> Option<Move> {
        if start == end {
            return None;
        }

        let mut best_index = start;
        let mut best_score = self.scores[start];

        for i in start..end {
            if self.scores[i] > best_score {
                best_index = i;
                best_score = self.scores[i];
            }
        }

        let best_move = self.moves[best_index];

        self.moves.swap(start, best_index);
        self.scores.swap(start, best_index);

        return Some(best_move);
    }

    // Give all the moves a rough score
    // Captures are scored acording to a rough MVV-LVA (Most Valuable 
    // Victim - Least Valuable Attacker) scheme, by subtracting the
    // victim's value from the attacker's (so, a Queen captured by a  
    // pawn is great, a Pawn captured by a Queen is meh) (Should it 
    // really be _negative_, though?)
    // NOTE: Unintuitively, I kept getting _better_ results when omitting the
    // LVA part of MVV-LVA. Hence, we only score the captures by looking at
    // the captured piece for now, until I figure out why this is happening.
    fn score_tacticals(&mut self) {
        self.quiet_index = self.index;

        for i in self.index..self.moves.len() {
            let mv = self.moves[i];
            let mut is_tactical = false;

            if mv.is_capture() {
                let capture_sq = if mv.is_en_passant() {
                    let side = self.position.board.current;
                    let ep_sq = self.position.board.en_passant.unwrap();
                    ep_sq.backward(side).unwrap()
                } else {
                    mv.tgt()
                };

                let attacker = self.position.board.get_at(mv.src()).unwrap();
                let captured = self.position.board.get_at(capture_sq).unwrap();
                self.scores[i] += TACTICAL_OFFSET;
                self.scores[i] += VICTIM_VALS[captured.piece_type() as usize];
                self.scores[i] -= ATTACKER_VALS[attacker.piece_type() as usize];

                is_tactical = true
            }

            if mv.is_promotion() {
                self.scores[i] += TACTICAL_OFFSET;
                self.scores[i] += ATTACKER_VALS[mv.get_promo_type().unwrap() as usize];
                is_tactical = true
            }

            // Move tactical to the front, and bump up the quiet_index
            if is_tactical {
                self.moves.swap(i, self.quiet_index);
                self.quiet_index += 1;
            }
        }
    }

    // Go over the killers list and move all of them to the front of the quiet
    // moves
    fn score_quiets(&mut self) {
        for i in self.quiet_index..self.moves.len() {
            let mv = &self.moves[i];

            if self.killers.contains(mv) {
                self.scores[i] += KILLER_BONUS;
            } else {
                let board = self.position.board;
                let &piece = board.get_at(mv.src()).unwrap();
                let mut score = self.position.score.clone();

                score.remove(board.current, piece, mv.src());
                score.add(board.current, piece, mv.tgt());

                self.scores[i] += score.total();
            }
        }
    }
}

impl<'a> Iterator for MovePicker<'a> {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        // Check if we've reached the end of the move list
        if self.index == self.moves.len() {
            return None;
        }

        if !self.opts.ordering {
            let mv = self.moves[self.index];
            self.index += 1;
            return Some(mv);
        }

        // Play TT move first (principal variation move)
        if self.stage == Stage::TTMove {
            self.stage = Stage::ScoreTacticals;

            if self.opts.tt_move {
                let tt_move = self.tt_move.and_then(|tt| {
                    self.find_swap(self.index, self.moves.len(), |mv| mv == tt)
                });

                if tt_move.is_some() {
                    self.index += 1;
                    return tt_move;
                }
            }
        } 

        if self.stage == Stage::ScoreTacticals {
            self.stage = Stage::Tacticals;

            if self.opts.mvv_lva {
                self.score_tacticals();
            }
        }

        // Run over the move list, return the highest scoring move, but do a 
        // partial sort on every run, so we do progressively less work on these
        // scans
        if self.stage == Stage::Tacticals {
            if self.index < self.quiet_index && self.opts.mvv_lva {
                let tactical = self.partial_sort(self.index, self.quiet_index);
                assert!(tactical.is_some(), "There should always be tacticals up until `quiet_index`");

                self.index += 1;
                return tactical;
            } else {
                self.stage = Stage::ScoreQuiets;
            }
        }

        // Play killer moves
        if self.stage == Stage::ScoreQuiets {
            self.stage = Stage::Quiets;

            if self.opts.killers {
                self.score_quiets();
            }
        }

        if self.stage == Stage::Quiets {
            if self.index < self.moves.len() {
                let quiet = self.partial_sort(self.index, self.moves.len());
                assert!(quiet.is_some(), "There should always be a quiet up until `moves.len()`");

                self.index += 1;
                return quiet;
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use crate::search::SearchOpts;
    use crate::tests::run_test_suite;

    #[test]
    /// Move ordering should _never_ change the outcome of the search
    fn ordering_move_picker() {
        const DEPTH: usize = 5;
        let mut without_move_picker = SearchOpts::NONE;
        without_move_picker.tt = true;

        let mut with_move_picker = SearchOpts::NONE;
        with_move_picker.tt = true;
        with_move_picker.ordering = true;

        run_test_suite(without_move_picker, with_move_picker, DEPTH);
    }

    #[test]
    #[ignore] // Don't want these running on every single test run
    /// Move ordering should _never_ change the outcome of the search
    fn ordering_tt_move() {
        const DEPTH: usize = 5;
        let mut without_tt_move = SearchOpts::NONE;
        without_tt_move.tt = true;
        without_tt_move.ordering = true;

        let mut with_tt_move = SearchOpts::NONE;
        with_tt_move.tt = true;
        with_tt_move.ordering = true;
        with_tt_move.tt_move = true;

        run_test_suite(without_tt_move, with_tt_move, DEPTH);
    }

    #[test]
    #[ignore] // Don't want these running on every single test run
    /// Move ordering should _never_ change the outcome of the search
    fn ordering_mvv_lva() {
        const DEPTH: usize = 5;
        let mut without_mvv_lva = SearchOpts::NONE;
        without_mvv_lva.tt = true;
        without_mvv_lva.ordering = true;
        without_mvv_lva.tt_move = true;

        let mut with_mvv_lva = SearchOpts::NONE;
        with_mvv_lva.tt = true;
        with_mvv_lva.ordering = true;
        with_mvv_lva.tt_move = true;
        with_mvv_lva.mvv_lva = true;

        run_test_suite(without_mvv_lva, with_mvv_lva, DEPTH);
    }

    #[test]
    #[ignore] // Don't want these running on every single test run
    /// Move ordering should _never_ change the outcome of the search
    fn ordering_killers() {
        const DEPTH: usize = 5;
        let mut without_killers = SearchOpts::NONE;
        without_killers.tt = false;
        without_killers.ordering = true;
        without_killers.tt_move = true;
        without_killers.mvv_lva = true;

        let mut with_killers = SearchOpts::NONE;
        with_killers.tt = false;
        with_killers.ordering = true;
        with_killers.tt_move = true;
        with_killers.mvv_lva = true;
        with_killers.killers = true;

        run_test_suite(without_killers, with_killers, DEPTH);
    }
}
