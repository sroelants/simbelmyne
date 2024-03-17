//! A Move Picker is a lazy wrapper around a Vec of moves that sorts and yields
//! moves as lazily as possible. It is a highly simplified version of 
//! Stockfish's strategy.
//!
//! Moves are sorted and yielded in stages, and only when requested. The stages
//! our:
//!
//! 1. TTMove: If we were provided a move from the transposition table, return
//!    this first without sorting any further
//!
//! 2. ScoreTacticals: Assign scores to all captures and promotions. The 
//! captures ase scored according to a Most Valuable Victim / Least Valuable 
//! Attacker scheme: a capture of a more valuable piece is scored more highly,
//! and, at the same time, a capture by a less valuable piece is preferred.
//! (Capturing a queen with a knight is preferred over capturing a bishop with a 
//! knight, but also capturing a queen with a pawn is considered a safer bet 
//! than capturing a queen with a quen)
//!
//! We're not doing any Static Exchange Evaluation, so we can't tell whether or 
//! not a capture is _actually_ good. (Capturing a bishop with a queen, only to
//! have your queen captured by a pawn in the turn after, should be put in the 
//! back of the move list, but is currently _not_.
//!
//! 3. Tacticals: Yield the tacticals one by one, doing a partial insertion
//! sort on every pass, so the move list gradually gets sorted.
//!
//! 4. ScoreQuiets: Score non-captures using various other heuristics (killer
//! moves, history tables).
//!
//! 5. Quiets: Play the quiet moves in sorted order, again doing a partial sort
//! on every pass until we reach the end.

use chess::movegen::legal_moves::MAX_MOVES;
use chess::movegen::legal_moves::MoveList;
use chess::movegen::moves::Move;
use chess::piece::PieceType;
use crate::search::params::MOVE_ORDERING;
use crate::search::params::MVV_LVA;
use crate::search::params::SEE_ORDERING;
use crate::search::params::TT_MOVE;
use crate::search_tables::HistoryTable;
use crate::search_tables::Killers;
use crate::position::Position;

/// Relative piece values used for MVV-LVA scoring.
#[rustfmt::skip]
const PIECE_VALS: [i32; PieceType::COUNT] = 
    // Pawn, Knight, Bishop, Rook, Queen, King
    [  100,  200,    300,    500,  900,   900];

/// The bonus score used to place killer moves ahead of the other quiet moves
const KILLER_BONUS: i32 = 10000;

/// The stages of move ordering
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum Stage {
    TTMove,
    ScoreTacticals,
    GoodTacticals,
    ScoreQuiets,
    Quiets,
    BadTacticals,
    Done,
}

/// A Move Picker is a lazy wrapper around a Vec of moves that sorts and yields
/// moves as lazily as possible.
pub struct MovePicker<'pos> {
    /// The current stage the move picker is in
    stage: Stage,

    /// The stored moves in the move picker
    moves: MoveList,

    /// The index of the move up to which we have already yielded.
    index: usize,

    /// The index of the first quiet move
    quiet_index: usize,

    /// The index of the first bad tactical move
    bad_tactical_index: usize,

    /// The (optional) hash table move we were provided with
    tt_move: Option<Move>,

    /// The scores associated with every move, using the same indexing
    scores: [i32; MAX_MOVES],

    /// The current board position
    position: &'pos Position,

    // A set of "Killer moves" for the current ply. These are quiet moves that 
    // were still good enough to cause a beta cutoff elsewhere in the search 
    // tree.
    killers: Killers,

    /// Whether or not to skip quiet moves and bad tacticals
    /// Can be set dynamically after we've already started iterating the moves.
    pub only_good_tacticals: bool,
}

impl<'pos> MovePicker<'pos> {
    pub fn new(
        position: &'pos Position, 
        moves: MoveList,
        tt_move: Option<Move>,
        killers: Killers,
    ) -> MovePicker<'pos> {
        let scores = [0; MAX_MOVES];

        // If the move list is empty, we're done here.
        let initial_stage = if moves.len() == 0 { 
            Stage::Done 
        } else { 
            Stage::TTMove 
        };


        MovePicker {
            stage: initial_stage,
            quiet_index: 0,
            bad_tactical_index: moves.len(),
            position,
            scores,
            moves,
            tt_move,
            index: 0,
            killers,
            only_good_tacticals: false,
        }
    }

    /// Return the number of moves stored in the move picker
    pub fn len(&self) -> usize {
        self.moves.len()
    }

    /// Return the number of tacticals in the set of moves
    pub fn count_tacticals(&self) -> usize {
        self.moves.iter()
            .filter(|mv| mv.is_capture() || mv.is_castle())
            .count()
    }

    /// Swap moves at provided indices, and update their associated scores.
    fn swap_moves(&mut self, i: usize, j: usize) {
        self.moves.swap(i, j); 
        self.scores.swap(i, j);
    }

    /// Search the move list starting at `start`, up until `end`, exclusive, and
    /// swap the first move that satisfies the predicate with the element at 
    /// `start`.
    pub fn find_swap<T: Fn(Move) -> bool>(
        &mut self, 
        start: usize, 
        end: usize,
        pred: T
    ) -> Option<Move> {
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

    /// Score captures according to MVV-LVA (Most Valuable Victim, Least 
    /// Valuable Attacker)
    fn score_tacticals(&mut self) {
        let mut i = self.index;

        while i < self.bad_tactical_index {
            let mv = self.moves[i];
            let mut is_good_tactical = false;
            let mut is_bad_tactical = false;

            // Score captures according to MVV-LVA
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

                // If SEE comes out negative, the capture is considered a bad
                // capture, and should be moved to the back of the list
                if SEE_ORDERING && !self.position.board.see(mv, 0) {
                    is_bad_tactical = true;
                } else {
                    is_good_tactical = true;
                }
            }

            // Score promotians according to their LVA values as well. They
            // always end up _after_ captures, but before the quiets.
            if mv.is_promotion() {
                self.scores[i] += PIECE_VALS[mv.get_promo_type().unwrap() as usize];

                // If the promotion is an underpromotion, the move is considered
                // a bad tactical, and is moved to the back of the list
                if SEE_ORDERING && mv.get_promo_type().unwrap() != PieceType::Queen {
                    is_bad_tactical = true;
                } else {
                    is_good_tactical = true;
                }
            }

            // Move good tactical to the front, and bump up the quiet_index
            if is_good_tactical {
                self.swap_moves(i, self.quiet_index);
                self.quiet_index += 1;
            }

            // Move bad tactical to the back, and bump the bad_tactical_index
            if is_bad_tactical {
                self.bad_tactical_index -= 1;
                self.swap_moves(i, self.bad_tactical_index);

                // Important that we  don't increment the counter just yet, but
                // process the i'th position again, since we don't know what
                // move we just put there.
                continue;
            }

            i += 1;
        }
    }

    /// Score quiet moves according to the killer move and history tables
    fn score_quiets(&mut self, history_table: &HistoryTable) {
        for i in self.quiet_index..self.bad_tactical_index {
            let mv = &self.moves[i];

            if self.killers.moves().contains(mv) {
                self.scores[i] += KILLER_BONUS;
            } 

            let piece = self.position.board.get_at(mv.src()).unwrap();
            self.scores[i] += history_table.get(mv, piece) as i32;
        }
    }
}

// impl<'pos, 'hist> Iterator for MovePicker<'pos, 'hist> {
//     type Item = Move;

impl<'a> MovePicker<'a> {
    pub fn next(&mut self, history: &HistoryTable) -> Option<Move> {

        // Check if we've reached the end of the move list
        if self.stage == Stage::Done {
            return None;
        }

        // In case move ordering is disabled, simply iterate over the moves as 
        // is. (mostly for debugging purposes, this will probably be removed at
        // some point).
        if !MOVE_ORDERING {
            let mv = self.moves[self.index];
            self.index += 1;

            if self.index == self.moves.len() {
                self.stage = Stage::Done;
            }

            return Some(mv);

        }

        ////////////////////////////////////////////////////////////////////////
        //
        // Transposition table move
        //
        ////////////////////////////////////////////////////////////////////////

        // Play TT move first (principal variation move)
        if self.stage == Stage::TTMove {
            self.stage = Stage::ScoreTacticals;

            if TT_MOVE {
                let tt_move = self.tt_move.and_then(|tt| {
                    self.find_swap(self.index, self.moves.len(), |mv| mv == tt)
                });

                if tt_move.is_some() {
                    self.index += 1;
                    self.quiet_index += 1;
                    return tt_move;
                }
            }
        } 

        ////////////////////////////////////////////////////////////////////////
        //
        // Score tacticals
        //
        ////////////////////////////////////////////////////////////////////////

        if self.stage == Stage::ScoreTacticals {
            if MVV_LVA {
                self.score_tacticals();
            }

            self.stage = Stage::GoodTacticals;
        }

        ////////////////////////////////////////////////////////////////////////
        //
        // Yield good tacticals
        //
        ////////////////////////////////////////////////////////////////////////

        // Run over the move list, return the highest scoring move, but do a 
        // partial sort on every run, so we do progressively less work on these
        // scans
        if self.stage == Stage::GoodTacticals {
            if self.index < self.quiet_index {
                let tactical = self.partial_sort(self.index, self.quiet_index);

                self.index += 1;
                return tactical;
            } else if self.only_good_tacticals {
                self.stage = Stage::Done
            } else {
                self.stage = Stage::ScoreQuiets;
            } 
        }

        ////////////////////////////////////////////////////////////////////////
        //
        // Score Quiets
        //
        ////////////////////////////////////////////////////////////////////////

        if self.stage == Stage::ScoreQuiets {
            self.score_quiets(history);
            self.stage = Stage::Quiets;
        }

        ////////////////////////////////////////////////////////////////////////
        //
        // Yield quiets
        //
        ////////////////////////////////////////////////////////////////////////

        if self.stage == Stage::Quiets {
            if !self.only_good_tacticals && self.index < self.bad_tactical_index {
                let quiet = self.partial_sort(self.index, self.bad_tactical_index);

                self.index += 1;
                return quiet;
            } else {
                self.stage = Stage::BadTacticals;
            }
        }

        ////////////////////////////////////////////////////////////////////////
        //
        // Yield bad tacticals
        //
        ////////////////////////////////////////////////////////////////////////

        if self.stage == Stage::BadTacticals {
            if !self.only_good_tacticals && self.index < self.moves.len() {
                let tactical = self.partial_sort(self.index, self.moves.len());

                self.index += 1;
                return tactical;
            } else {
                self.stage = Stage::Done;
            }
        }

        ////////////////////////////////////////////////////////////////////////
        //
        // All done 👋
        //
        ////////////////////////////////////////////////////////////////////////

        None
    }
}

////////////////////////////////////////////////////////////////////////////////
//
// Tests
//
////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use chess::board::Board;
    use crate::position::Position;
    use super::*;

    #[test]
    fn test_move_picker() {
        // kiwipete
        let board: Board = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1".parse().unwrap();
        let position = Position::new(board);
        let legal_moves = board.legal_moves::<true>();
        let history = HistoryTable::new();

        let mut picker = MovePicker::new(
            &position, 
            legal_moves.clone(), 
            None, 
            Killers::new(), 
        ); 

        picker.only_good_tacticals = true;

        while let Some(mv) = picker.next(&history) {
            println!("Yielded {mv}");
        }
    }
}

