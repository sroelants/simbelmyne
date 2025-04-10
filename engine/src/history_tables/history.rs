/// History tables assign scores to quiet moves that caused beta cutoffs 
/// _anywhere_ in the search tree.
///
/// That means that, if TT tables are the most "accurate", and Killer moves are
/// slightly less so, then History tables are the least precise: They don't even
/// relate to moves _in the current ply_. They're literally moves played 
/// _anywhere_ in the search tree. As such, they're typically the least valuable 
/// way or sorting quiet, but still somewhat better than not sorting at all!
///
/// The tables are indexed by piece type and target square (rather than, say,
/// source square and target square) to reduce the footprint of the table.
///
/// The scores are scaled by the remaining search depth (how high up in the 
/// tree the move was played), so we don't flood the scores with moves played at
/// the leaves, which are inherently less valuable.

use crate::search::params::hist_bonus_const;
use crate::search::params::hist_bonus_const_cutoff;
use crate::search::params::hist_bonus_linear;
use crate::search::params::hist_bonus_quadratic;
use std::ops::{Add, AddAssign, Index, IndexMut, Neg, Sub, SubAssign};
use chess::board::Board;
use chess::square::Square;
use chess::piece::Piece;
use chess::movegen::moves::Move;

////////////////////////////////////////////////////////////////////////////////
//
// History table
//
////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct HistoryTable {
    scores: [[HistoryScore; Square::COUNT]; Piece::COUNT]
}

pub const MAX_HIST_SCORE: i16 = i16::MAX/2;

impl HistoryTable {
    /// Create a new HistoryTable
    pub fn new() -> Self {
        Self {
            scores: [[HistoryScore(0); Square::COUNT]; Piece::COUNT]
        }
    }
}

impl Index<HistoryIndex> for HistoryTable {
    type Output = HistoryScore;

    fn index(&self, index: HistoryIndex) -> &Self::Output {
        &self.scores[index.moved][index.tgt()]
    }
}

impl IndexMut<HistoryIndex> for HistoryTable {
    fn index_mut(&mut self, index: HistoryIndex) -> &mut Self::Output {
        &mut self.scores[index.moved][index.tgt()]
    }
}

////////////////////////////////////////////////////////////////////////////////
//
// History table index
//
////////////////////////////////////////////////////////////////////////////////

/// A History index is a convenient wrapper used to index into a History table,
/// comprising of a Piece and a destination Square
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct HistoryIndex{
    pub mv: Move,
    pub moved: Piece,
    pub captured: Option<Piece>,
}

impl HistoryIndex {
    pub fn new(board: &Board, mv: Move) -> Self {
        let captured = if mv.is_capture() { 
                board.get_at(mv.get_capture_sq())
            } else {
                None
            };

        Self {
            mv,
            moved: board.get_at(mv.src()).unwrap(),
            captured,
        }
    }
}

impl Default for HistoryIndex {
    fn default() -> Self {
        Self {
            mv: Move::NULL,
            moved: Piece::WP,
            captured: None,
        }
    }
}

impl HistoryIndex {
    pub fn src(&self) -> Square {
        self.mv.src()
    }

    pub fn tgt(&self) -> Square {
        self.mv.tgt()
    }
}

////////////////////////////////////////////////////////////////////////////////
//
// History score
//
////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct HistoryScore(i16);

impl HistoryScore {
    /// Compute the appropriate history bonus for a given `depth`
    pub fn bonus(depth: usize) -> Self {
        let bonus: i16 = if depth > hist_bonus_const_cutoff() {
            hist_bonus_const()
        } else {
            hist_bonus_quadratic() * (depth * depth) as i16
            + hist_bonus_linear() * usize::max(depth - 1, 0) as i16
        };

        Self(bonus)
    }
}

impl Neg for HistoryScore {
    type Output = HistoryScore;

    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

impl Add for HistoryScore {
    type Output = HistoryScore;

    /// Add a value to a history score
    ///
    /// Saturates smoothly as the entry approaches `MAX_HIST_SCORE`
    ///
    /// "tapered" addition: (1 +- current / MAX_SCORE) * bonus
    /// boosted by 2x when adding negative value to high positive value,
    /// tapered to 0 when adding positive value to high positive value
    fn add(self, rhs: Self) -> Self::Output {
        let tapered = rhs.0 - ((self.0 as i32) * rhs.0.abs() as i32 / MAX_HIST_SCORE as i32) as i16;

        Self(self.0 + tapered)
    }
}

impl AddAssign for HistoryScore {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Sub for HistoryScore {
    type Output = HistoryScore;

    fn sub(self, rhs: Self) -> Self::Output {
        self + (-rhs)
    }
}

impl SubAssign for HistoryScore {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl From<HistoryScore> for i16 {
    fn from(value: HistoryScore) -> Self {
        value.0
    }
}

impl From<HistoryScore> for i32 {
    fn from(value: HistoryScore) -> Self {
        value.0 as i32
    }
}
