use chess::{movegen::moves::Move, piece::PieceType};
use itertools::Itertools;

use crate::position::Position;

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
    ScoreMoves,
    ReturnMoves,
    Done,
}

pub struct MovePicker<'a> {
    stage: Stage,
    position: &'a Position,
    moves: Vec<Move>,
    scores: Vec<i32>,
    tt_move: Option<Move>,
    index: usize,
}

impl<'a> MovePicker<'a> {
    pub fn new(position: &'a Position, moves: Vec<Move>, tt_move: Option<Move>) -> MovePicker {
        let mut scores = Vec::new();
        scores.resize_with(moves.len(), i32::default);

        MovePicker {
            stage: Stage::TTMove,
            position,
            scores,
            moves,
            tt_move,
            index: 0
        }
    }

    /// Search the move list starting at `start`, up until `end`, exclusive, and
    /// swap the first move that satisfies the predicate with the element at 
    /// `start`.
    pub fn find_swap<T: Fn(Move) -> bool>(&mut self, start: usize, end: usize, pred: T) -> Option<Move>{
        if let Some((idx, _mv)) = self.moves
            .iter()
            .skip(start)
            .take(end - start)
            .find_position(|&&mv| pred(mv)) {
            self.moves.swap(idx, start);
            Some(self.moves[start])
        } else {
            None
        }
    }

    /// Do a pass over all the moves, starting at the current `self.index` up 
    /// till `end` (exclusive). Find the largest scoring move, and swap it 
    /// to `self.index`, then return it.
    pub fn partial_sort(&mut self, end: usize) -> Option<Move> {
        if self.index == end {
            return None;
        }

        let mut best_index = self.index;
        let mut best_score = self.scores[self.index];

        for i in (self.index+1)..end {
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
}

impl<'a> Iterator for MovePicker<'a> {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        // No moves left, start returning `None`s
        if self.stage == Stage::Done {
            return None;
        }

        if self.tt_move.is_none() {
            self.stage = Stage::ScoreMoves;
        }

        // Play TT move first (principal variation move)
        if self.stage == Stage::TTMove {
            self.stage = Stage::ScoreMoves;

            let tt_move = self.tt_move.unwrap();

            if let Some(mv) = self.find_swap(self.index, self.moves.len(), |mv| mv == tt_move) {
                self.index += 1;
                return Some(mv);
            }
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
        if self.stage == Stage::ScoreMoves {
            for i in 0..self.moves.len() {
                let mv = self.moves[i];

                if mv.is_capture() && !mv.is_en_passant() { 
                    let captured = self.position.board.get_at(mv.tgt()).unwrap();

                    self.scores[i] += VICTIM_VALS[captured.piece_type() as usize]
                } 

                if mv.is_promotion() {
                    self.scores[i] += ATTACKER_VALS[mv.get_promo_type().unwrap() as usize]
                }
            }

            self.stage = Stage::ReturnMoves;
        }

        // Run over the move list, return the highest scoring move, but do a 
        // partial sort on every run, so we do progressively less work on these
        // scans
        if self.stage == Stage::ReturnMoves {
            if let Some(mv) = self.partial_sort(self.moves.len()) {
                self.index += 1;
                return Some(mv)
            }
        }

        // Check if we've reached the end of the move list
        if self.index + 1 == self.moves.len() {
            self.stage = Stage::Done;
        }

        None
    }
}
