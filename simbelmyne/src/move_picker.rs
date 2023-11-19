use chess::{movegen::moves::Move, piece::PieceType};

use crate::{position::Position, search::{Killers, KillersIter, Opts}};

#[rustfmt::skip]
const VICTIM_VALS: [i32; PieceType::COUNT] = 
    // Pawn, Knight, Bishop, Rook, Queen, King
    [  100,  300,    300,    500,  900,   50000];

#[rustfmt::skip]
const ATTACKER_VALS: [i32; PieceType::COUNT] = 
    // Pawn, Knight, Bishop, Rook, Queen, King
    [  10,  30,    30,    50,  90,   50];

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum Stage {
    TTMove,
    ScoreTacticals,
    Tacticals,
    BoostKillers,
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
    opts: Opts
}

impl<'a> MovePicker<'a> {
    pub fn new(
        position: &'a Position, 
        moves: Vec<Move>, 
        tt_move: Option<Move>,
        killers: Killers,
        opts: Opts
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
    pub fn find_swap<T: Fn(Move) -> bool>(&mut self, start: usize, end: usize, pred: T) -> Option<Move>{
        for i in start..end {
            if pred(self.moves[i]) {
                self.moves.swap(start, i); 
                return Some(self.moves[start])
            }
        }

        None
    }

    /// Do a pass over all the moves, starting at the current `self.index` up 
    /// till `end` (exclusive). Find the largest scoring move, and swap it 
    /// to `self.index`, then return it.
    pub fn partial_sort(&mut self,start: usize, end: usize) -> Option<Move> {
        if self.index == end {
            return None;
        }

        let mut best_index = self.index;
        let mut best_score = self.scores[self.index];

        for i in start..end {
            if self.scores[i] > best_score {
                best_index = i;
                best_score = self.scores[i];
            }
        }

        let best_move = self.moves[best_index];

        self.moves.swap(self.index, best_index);
        self.scores.swap(self.index, best_index);

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
                self.scores[i] += VICTIM_VALS[captured.piece_type() as usize];
                self.scores[i] -= ATTACKER_VALS[attacker.piece_type() as usize];

                is_tactical = true
            }

            if mv.is_promotion() {
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
    fn boost_killers(&mut self) {
        let mut killer_index = self.index;

        for killer in self.killers {
            let found = self.find_swap(killer_index, self.moves.len(), |mv| mv == killer);

            if found.is_some() {
                killer_index += 1;
            }
        }
    }
}

impl<'a> Iterator for MovePicker<'a> {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.opts.ordering {
            self.stage = Stage::Quiets
        }

        // Check if we've reached the end of the move list
        if self.index == self.moves.len() {
            return None;
        }

        // Play TT move first (principal variation move)
        if self.stage == Stage::TTMove {
            let tt_move = self.tt_move.and_then(|tt| {
                self.find_swap(self.index, self.moves.len(), |mv| mv == tt)
            });

            if self.opts.mvv_lva {
                self.stage = Stage::ScoreTacticals;
            } else {
                self.stage = Stage::BoostKillers;
            }

            if tt_move.is_some() {
                self.index += 1;
                return tt_move;
            }
        }

        if self.stage == Stage::ScoreTacticals {
            self.score_tacticals();
            self.stage = Stage::Tacticals;
        }

        // Run over the move list, return the highest scoring move, but do a 
        // partial sort on every run, so we do progressively less work on these
        // scans
        if self.stage == Stage::Tacticals {
            if self.index < self.quiet_index {
                let tactical = self.partial_sort(self.index, self.quiet_index);
                assert!(tactical.is_some(), "There should always be tacticals up until `quiet_index`");

                self.index += 1;
                return tactical;
            }

            self.stage = Stage::BoostKillers;
        }

        // Play killer moves
        if self.stage == Stage::BoostKillers {
            if self.opts.killers {
                self.boost_killers();
            }

            self.stage = Stage::Quiets;
        }

        // Play quiet moves (no sorting required!)
        if self.stage == Stage::Quiets {
            let quiet = self.moves[self.index];
            self.index += 1;
            return Some(quiet);
        }

        None
    }
}
