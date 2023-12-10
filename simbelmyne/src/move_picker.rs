use chess::{movegen::moves::Move, piece::PieceType};

use crate::search::{KILLER_MOVES, HISTORY_TABLE, MOVE_ORDERING, TT_MOVE, MVV_LVA};
use crate::search_tables::HistoryTable;
use crate::{position::Position, search_tables::Killers};

#[rustfmt::skip]
const PIECE_VALS: [i32; PieceType::COUNT] = 
    // Pawn, Knight, Bishop, Rook, Queen, King
    [  100,  200,    300,    500,  900,   900];

const KILLER_BONUS: i32 = 10000;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum Stage {
    TTMove,
    ScoreTacticals,
    Tacticals,
    ScoreQuiets,
    Quiets,
    Done,
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
    history_table: HistoryTable,
}

impl<'a> MovePicker<'a> {
    pub fn new(
        position: &'a Position, 
        moves: Vec<Move>, 
        tt_move: Option<Move>,
        killers: Killers,
        history_table: HistoryTable,
    ) -> MovePicker {
        let mut scores = Vec::new();
        scores.resize_with(moves.len(), i32::default);
        let initial_stage = if moves.len() > 0 { 
            Stage::TTMove 
        } else { 
            Stage::Done 
        };


        MovePicker {
            stage: initial_stage,
            position,
            scores,
            moves,
            tt_move,
            index: 0,
            quiet_index: 0,
            killers,
            history_table,
        }
    }

    pub fn len(&self) -> usize {
        self.moves.len()
    }

    // pub fn get_first(&self) -> Move {
    //     if let Some(tt_move) = self.tt_move {
    //         tt_move
    //     } else {
    //         self.moves[0]
    //     }
    // }

    pub fn captures(self) -> CapturePicker<'a> {
        CapturePicker(self)
    }

    /// Swap moves at provided indices, and update their associated scores.
    fn swap_moves(&mut self, i: usize, j: usize) {
        self.moves.swap(i, j); 
        self.scores.swap(i, j);
    }

    /// Search the move list starting at `start`, up until `end`, exclusive, and
    /// swap the first move that satisfies the predicate with the element at 
    /// `start`.
    pub fn find_swap<T: Fn(Move) -> bool>(&mut self, start: usize, end: usize, pred: T) -> Option<Move> {
        for i in start..end {
            if pred(self.moves[i]) {
                let found = self.moves[i];
                self.swap_moves(start, i);
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

        self.swap_moves(start, best_index);

        return Some(best_move);
    }

    /// Score captures according to MVV-LVA (Most Valuable Victim, Least Valuable Attacker)
    fn score_tacticals(&mut self) {
        self.quiet_index = self.index;

        for i in self.index..self.moves.len() {
            let mv = self.moves[i];
            let mut is_tactical = false;

            if mv.is_capture() {
                let victim_sq = if mv.is_en_passant() {
                    let side = self.position.board.current;
                    let ep_sq = self.position.board.en_passant.unwrap();
                    ep_sq.backward(side).unwrap()
                } else {
                    mv.tgt()
                };

                let victim = self.position.board.get_at(victim_sq).unwrap();
                let attacker = self.position.board.get_at(mv.src()).unwrap();
                self.scores[i] += 100 * PIECE_VALS[victim.piece_type() as usize];
                self.scores[i] -= PIECE_VALS[attacker.piece_type() as usize];

                is_tactical = true
            }

            if mv.is_promotion() {
                self.scores[i] += PIECE_VALS[mv.get_promo_type().unwrap() as usize];
                is_tactical = true
            }

            // Move tactical to the front, and bump up the quiet_index
            if is_tactical {
                self.swap_moves(i, self.quiet_index);
                self.quiet_index += 1;
            }
        }
    }

    // Go over the killers list and move all of them to the front of the quiet
    // moves
    fn score_quiets(&mut self) {
        for i in self.quiet_index..self.moves.len() {
            let mv = &self.moves[i];

            if KILLER_MOVES && self.killers.contains(mv) {
                self.scores[i] += KILLER_BONUS;
            } else if HISTORY_TABLE {
                let piece = self.position.board.get_at(mv.src()).unwrap();
                self.scores[i] += self.history_table.get(mv, piece);
            }
        }
    }
}

impl<'a> Iterator for MovePicker<'a> {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        // Check if we've reached the end of the move list
        if self.stage == Stage::Done {
            return None;
        }

        if !MOVE_ORDERING {
            let mv = self.moves[self.index];

            self.index += 1;

            if self.index == self.moves.len() {
                self.stage = Stage::Done;
            }

            return Some(mv);

        }

        // Play TT move first (principal variation move)
        if self.stage == Stage::TTMove {
            self.stage = Stage::ScoreTacticals;

            if TT_MOVE {
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
            if MVV_LVA {
                self.score_tacticals();
            }

            self.stage = Stage::Tacticals;
        }

        // Run over the move list, return the highest scoring move, but do a 
        // partial sort on every run, so we do progressively less work on these
        // scans
        if self.stage == Stage::Tacticals {
            if self.index < self.quiet_index {
                let tactical = self.partial_sort(self.index, self.quiet_index);

                self.index += 1;
                return tactical;
            } else {
                self.stage = Stage::ScoreQuiets;
            } 
        }

        // Play killer moves
        if self.stage == Stage::ScoreQuiets {
            self.score_quiets();

            self.stage = Stage::Quiets;
        }

        if self.stage == Stage::Quiets {
            if self.index < self.moves.len() {
                let quiet = self.partial_sort(self.index, self.moves.len());

                self.index += 1;
                return quiet;
            } else {
                self.stage = Stage::Done;
            }
        }

        None
    }
}

pub struct CapturePicker<'a>(MovePicker<'a>);

impl<'a> Iterator for CapturePicker<'a> {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0.next() {
            Some(mv) if mv.is_capture() => Some(mv),
            _ => None
        }
    }
}
