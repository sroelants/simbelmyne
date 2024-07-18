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

use chess::movegen::legal_moves::Quiets;
use chess::movegen::legal_moves::Tacticals;
use chess::movegen::legal_moves::MAX_MOVES;
use chess::movegen::legal_moves::MoveList;
use chess::movegen::moves::Move;
use chess::piece::PieceType;
use crate::history_tables::capthist::TacticalHistoryTable;
use crate::history_tables::history::HistoryIndex;
use crate::history_tables::history::HistoryTable;
use crate::history_tables::killers::Killers;
use crate::history_tables::threats::ThreatIndex;
use crate::history_tables::threats::ThreatsHistoryTable;
use crate::history_tables::History;
use crate::position::Position;

/// The bonus score used to place killer moves ahead of the other quiet moves
const KILLER_BONUS: i32 = 30000;
const COUNTERMOVE_BONUS: i32 = 20000;

/// The stages of move ordering
#[derive(Debug, PartialEq, Eq, Copy, Clone, PartialOrd)]
pub enum Stage {
    TTMove,
    GenerateTacticals,
    ScoreTacticals,
    GoodTacticals,
    GenerateQuiets,
    ScoreQuiets,
    Quiets,
    BadTacticals,
    Done,
}

/// A Move Picker is a lazy wrapper around a Vec of moves that sorts and yields
/// moves as lazily as possible.
pub struct MovePicker<'pos, const ALL: bool = true> {
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

    /// A move that caused a beta cutoff before when played right after the move 
    /// that was just played.
    countermove: Option<Move>,

    /// Whether or not to skip quiet moves and bad tacticals
    /// Can be set dynamically after we've already started iterating the moves.
    pub only_good_tacticals: bool,

    ply: usize,
}

impl<'pos, const ALL: bool> MovePicker<'pos, ALL> {
    pub fn new(
        position: &'pos Position, 
        tt_move: Option<Move>,
        killers: Killers,
        countermove: Option<Move>,
        ply: usize
    ) -> MovePicker<'pos, ALL> {
        let scores = [0; MAX_MOVES];

        // If we're only interested in tacticals, but the TT move is
        // quiet, just clear it and forget about it.
        let tt_move = tt_move.filter(|mv| ALL || mv.is_tactical());

        MovePicker {
            stage: Stage::TTMove,
            quiet_index: 0,
            bad_tactical_index: 0,
            position,
            scores,
            moves: MoveList::new(),
            tt_move,
            index: 0,
            killers,
            countermove,
            only_good_tacticals: false,
            ply,
        }
    }

    /// Return the stage of movegen
    pub fn stage(&self) -> Stage {
        self.stage
    }

    pub fn current_score(&self) -> i32 {
        self.scores[self.index]
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
    fn score_tacticals(&mut self, tactical_history: &TacticalHistoryTable, history: &History) {
        use PieceType::*;
        let mut i = self.index;

        while i < self.bad_tactical_index {
            let mv = self.moves[i];
            let idx = HistoryIndex::new(&self.position.board, mv);

            let is_bad_tactical = if mv.is_capture() {
                !self.position.board.see(mv, 0)
            } else {
                mv.get_promo_type().is_some_and(|pt| pt != Queen)
            };

            ////////////////////////////////////////////////////////////////////
            //
            // Score captures according to MVV + Capthist
            //
            ////////////////////////////////////////////////////////////////////

            if mv.is_capture() {
                let victim = self.position.board
                    .get_at(mv.get_capture_sq())
                    .unwrap();

                // MVV-LVA
                self.scores[i] += 32 * piece_vals(victim.piece_type());

                // Capthist
                self.scores[i] += i32::from(history.get_hist_score(mv, &self.position.board));
            }

            ////////////////////////////////////////////////////////////////////
            //
            // Score promotians according to their Capthist
            //
            // Promotions are stored in the history table under "pawn capture",
            // since those entries are never used for actual captures.
            // (There's never any pawns on the back rank)
            //
            ////////////////////////////////////////////////////////////////////

            else if mv.is_promotion() {
                self.scores[i] += i32::from(history.get_hist_score(mv, &self.position.board));
            }

            ////////////////////////////////////////////////////////////////////
            //
            // Move bad tactical to the back, and bump the bad_tactical_index
            //
            ////////////////////////////////////////////////////////////////////

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
    fn score_quiets(
        &mut self, 
        history_table: &ThreatsHistoryTable, 
        oneply: Option<&HistoryTable>,
        twoply: Option<&HistoryTable>,
        fourply: Option<&HistoryTable>,
        history: &History,
    ) {
        for i in self.quiet_index..self.moves.len() {
            let mv = &self.moves[i];

            // let threat_idx = ThreatIndex::new(
            //     self.position.board.get_threats(), 
            //     *mv
            // );
            //
            if self.killers.len() > 0 && mv == &self.killers.moves()[0] {
                self.scores[i] += 2 * KILLER_BONUS;
            }

            if self.killers.len() > 1 && mv == &self.killers.moves()[1] {
                self.scores[i] += KILLER_BONUS;
            }

            if self.countermove == Some(*mv) {
                    self.scores[i] += COUNTERMOVE_BONUS;
            }

            // if history.killers[self.ply].len() > 0 
            //     && mv == &history.killers[self.ply].moves()[0] {
            //     self.scores[i] += 2 * KILLER_BONUS;
            // }
            //
            // if history.killers[self.ply].len() > 1 
            //     && mv == &history.killers[self.ply].moves()[1] {
            //     self.scores[i] += KILLER_BONUS;
            // }
            //
            // if history.get_countermove() == Some(*mv) {
            //     self.scores[i] += COUNTERMOVE_BONUS;
            // }

            // let idx = HistoryIndex::new(&self.position.board, *mv);
            // self.scores[i] += i32::from(history_table[threat_idx][idx]);
            //
            // if let Some(conthist) = oneply.as_ref() {
            //     self.scores[i] += i32::from(conthist[idx]);
            // }
            //
            // if let Some(conthist) = twoply.as_ref() {
            //     self.scores[i] += i32::from(conthist[idx]);
            // }
            //
            // if let Some(conthist) = fourply.as_ref() {
            //     self.scores[i] += i32::from(conthist[idx]);
            // }

            self.scores[i] += i32::from(history.get_hist_score(*mv, &self.position.board));
        }
    }
}

impl<'a, const ALL: bool> MovePicker<'a, ALL> {
    pub fn next(
        &mut self, 
        main_history: &ThreatsHistoryTable, 
        tactical_history: &TacticalHistoryTable,
        oneply: Option<&HistoryTable>, 
        twoply: Option<&HistoryTable>,
        fourply: Option<&HistoryTable>,
        history: &History,
    ) -> Option<Move> {
        const WHITE: bool = true;
        const BLACK: bool = false;

        // Check if we've reached the end of the move list
        if self.stage == Stage::Done {
            return None;
        }

        ////////////////////////////////////////////////////////////////////////
        //
        // Transposition table move
        //
        // Play the TT move before even generating legal moves.
        // If we're lucky, the first move is a cutoff, and we saved ourselves
        // the trouble of generating the legal moves.
        //
        ////////////////////////////////////////////////////////////////////////

        if self.stage == Stage::TTMove {
            self.stage = Stage::GenerateTacticals;

            if let Some(tt_move) = self.tt_move {
                return Some(tt_move)
            }
        } 

        ////////////////////////////////////////////////////////////////////////
        //
        // Generate tactical moves
        //
        ////////////////////////////////////////////////////////////////////////

        if self.stage == Stage::GenerateTacticals {
            if self.position.board.current.is_white() {
                self.position.board.legal_moves_for::<WHITE, Tacticals>(&mut self.moves);
            } else {
                self.position.board.legal_moves_for::<BLACK, Tacticals>(&mut self.moves);
            }

            self.bad_tactical_index = self.moves.len();
            self.quiet_index = self.moves.len();

            // If we played a TT move, move it to the front straight away
            // and update the indices, so we don't treat it during the scoring
            // phase.
            if let Some(tt_move) = self.tt_move.filter(|mv| mv.is_tactical()) {
                let found = self.find_swap(
                    self.index, 
                    self.moves.len(), 
                    |mv| mv == tt_move
                );

                if found.is_some() {
                    self.index += 1;
                }
            }

            self.stage = Stage::ScoreTacticals;
        }

        ////////////////////////////////////////////////////////////////////////
        //
        // Score tacticals
        //
        ////////////////////////////////////////////////////////////////////////

        if self.stage == Stage::ScoreTacticals {
            self.score_tacticals(tactical_history, history);

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
            if self.index < self.bad_tactical_index {
                let tactical = self.partial_sort(self.index, self.bad_tactical_index);

                self.index += 1;
                return tactical;
            } else if self.only_good_tacticals {
                self.stage = Stage::Done
            } else {
                self.stage = Stage::GenerateQuiets;
            } 
        }

        ////////////////////////////////////////////////////////////////////////
        //
        // Generate quiets
        //
        ////////////////////////////////////////////////////////////////////////
        
        if self.stage == Stage::GenerateQuiets {
            if self.position.board.current.is_white() {
                self.position.board.legal_moves_for::<WHITE, Quiets>(&mut self.moves);
            } else {
                self.position.board.legal_moves_for::<BLACK, Quiets>(&mut self.moves);
            }

            self.index = self.quiet_index;

            // If we played a TT move, move it to the front straight away
            // and update the indices, so we don't treat it during the scoring
            // phase.
            if let Some(tt_move) = self.tt_move.filter(|mv| mv.is_quiet()) {
                let found = self.find_swap(
                    self.index, 
                    self.moves.len(), 
                    |mv| mv == tt_move
                );

                if found.is_some() {
                    self.index += 1;
                }
            }


            self.stage = Stage::ScoreQuiets;
        }

        ////////////////////////////////////////////////////////////////////////
        //
        // Score Quiets
        //
        ////////////////////////////////////////////////////////////////////////

        if self.stage == Stage::ScoreQuiets {
            self.score_quiets(main_history, oneply, twoply, fourply, history);
            self.stage = Stage::Quiets;
        }

        ////////////////////////////////////////////////////////////////////////
        //
        // Yield quiets
        //
        ////////////////////////////////////////////////////////////////////////

        if self.stage == Stage::Quiets {
            if !self.only_good_tacticals && self.index < self.moves.len() {
                let quiet = self.partial_sort(self.index, self.moves.len());

                self.index += 1;
                return quiet;
            } else {
                self.index = self.bad_tactical_index;
                self.stage = Stage::BadTacticals;
            }
        }

        ////////////////////////////////////////////////////////////////////////
        //
        // Yield bad tacticals
        //
        ////////////////////////////////////////////////////////////////////////

        if self.stage == Stage::BadTacticals {
            if !self.only_good_tacticals && self.index < self.quiet_index {
                let tactical = self.partial_sort(self.index, self.quiet_index);

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
// Helpers
//
////////////////////////////////////////////////////////////////////////////////

/// Return the MVV value for a given piece type
#[inline(always)]
fn piece_vals(pt: PieceType) -> i32 {
    use PieceType::*;
    use crate::search::params::*;

    match pt {
        Pawn => pawn_value(),
        Knight => knight_value(),
        Bishop => bishop_value(),
        Rook => rook_value(),
        Queen => queen_value(),
        King => 0
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
        let main_history = ThreatsHistoryTable::boxed();
        let tactical_history = TacticalHistoryTable::boxed();
        let history = History::new();

        let mut picker = MovePicker::<true>::new(
            &position, 
            None, 
            Killers::new(), 
            None,
            0
        ); 

        picker.only_good_tacticals = true;

        while let Some(mv) = picker.next(
            &main_history, 
            &tactical_history, 
            None, 
            None,
            None,
            &history
        ) {
            println!("Yielded {mv}");
        }
    }
}

