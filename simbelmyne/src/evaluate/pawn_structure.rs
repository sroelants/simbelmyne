use chess::bitboard::Bitboard;
use chess::board::Board;
use chess::piece::Color;
use chess::piece::Color::*;

use crate::evaluate::lookups::PASSED_PAWN_MASKS;

use super::lookups::{DOUBLED_PAWN_MASKS, FILES};
use super::params::{DOUBLED_PAWN_PENALTY, ISOLATED_PAWN_PENALTY, PASSED_PAWN_TABLE, PHALANX_PAWN_BONUS, PROTECTED_PAWN_BONUS};
use super::tuner::Trace;
use super::{Score, S};

const WHITE: bool = true;
const BLACK: bool = false;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct PawnStructure {
    /// The score associated with the pawn structure
    pub score: S,

    /// Passed pawn bitboards for White and Black
    pub passed_pawns: [Bitboard; Color::COUNT],

    /// Semi-open file bitboards for White and Black
    pub semi_open_files: [Bitboard; Color::COUNT],

    /// Outpost squares
    /// Squares that can't be attacked (easily) by opponent pawns, and are
    /// defended by one of our pawns
    pub outposts: [Bitboard; Color::COUNT],
}

impl PawnStructure {
    pub fn new(board: &Board, mut trace: &mut impl Trace) -> Self {
        // Pawn bitboardds
        let white_pawns = board.pawns(White);
        let black_pawns = board.pawns(Black);

        // Pawns attacks
        let white_attacks = board.pawn_attacks(White);
        let black_attacks = board.pawn_attacks(Black);

        // Passed pawns
        let white_passers = white_pawns
            .filter(|&pawn| {
                let mask = PASSED_PAWN_MASKS[White][pawn];
                (mask & black_pawns).is_empty()
            })
            .collect::<Bitboard>();

        let black_passers = black_pawns
            .filter(|&pawn| {
                let mask = PASSED_PAWN_MASKS[Black][pawn];
                (mask & white_pawns).is_empty()
            })
            .collect::<Bitboard>();

        // Semi-open files
        let white_semi_open_files = FILES.iter()
            .filter(|&&file| {
                (file & white_pawns).is_empty()
            })
            .collect::<Bitboard>();

        let black_semi_open_files = FILES.iter()
            .filter(|&&file| {
                (file & black_pawns).is_empty()
            })
            .collect::<Bitboard>();

        // Blocked pawns
        let white_blocked_pawns = white_pawns & black_pawns.backward::<WHITE>();
        let black_blocked_pawns = black_pawns & white_pawns.backward::<BLACK>();

        // Outposts
        let white_outposts = white_attacks & !(
            black_attacks 
            | black_attacks.forward_by::<BLACK>(1) 
            | black_attacks.forward_by::<BLACK>(2)
            | black_attacks.forward_by::<BLACK>(3)
        );
        let black_outposts = black_attacks & !(
            white_attacks 
            | white_attacks.forward_by::<WHITE>(1) 
            | white_attacks.forward_by::<WHITE>(2)
            | white_attacks.forward_by::<WHITE>(3)
        );

        let mut pawn_structure = Self {
            score: S::default(),
            passed_pawns: [white_passers, black_passers],
            semi_open_files: [white_semi_open_files, black_semi_open_files],
            outposts: [white_outposts, black_outposts]
        };

        pawn_structure.score = pawn_structure.compute_score::<WHITE>(board, trace) 
            - pawn_structure.compute_score::<BLACK>(board, trace);

        pawn_structure
    }

    pub fn score(&self) -> S {
        self.score
    }

    pub fn passed_pawns(&self, us: Color) -> Bitboard {
        self.passed_pawns[us]
    }

    pub fn semi_open_files(&self, us: Color) -> Bitboard {
        self.semi_open_files[us]
    }

    pub fn open_files(&self) -> Bitboard {
        self.semi_open_files(White) & self.semi_open_files(Black)
    }

    pub fn outposts(&self, us: Color) -> Bitboard {
        self.outposts[us]
    }

    pub fn compute_score<const WHITE: bool>(&self, board: &Board, trace: &mut impl Trace) -> S {
        let mut total = S::default();
        let us = if WHITE { White } else { Black };
        let perspective = if WHITE { 1 } else { -1 };
        let our_pawns = board.pawns(us);

        // Passed pawns
        for sq in self.passed_pawns(us) {
            let sq = if WHITE { sq.flip() } else { sq };
            total += PASSED_PAWN_TABLE[sq];

            trace.add(|t| t.passed_pawn[sq] += perspective);
        }

        // Doubled pawns
        for mask in DOUBLED_PAWN_MASKS {
            let doubled = (our_pawns & mask).count().saturating_sub(1) as Score;
            total += DOUBLED_PAWN_PENALTY * doubled;

            trace.add(|t| t.doubled_pawn += perspective * doubled);
        }

        // Phalanx pawns
        let phalanx_pawns = our_pawns & (our_pawns.left() | our_pawns.right());
        let phalanx_count = phalanx_pawns.count() as i32;
        total += PHALANX_PAWN_BONUS * phalanx_count;

        // Connected pawns
        let protected_pawns = our_pawns & board.pawn_attacks(us);
        let protected_count = protected_pawns.count() as i32;
        total += PROTECTED_PAWN_BONUS * protected_count;

        // Isolated pawns
        let isolated = our_pawns 
            & (self.semi_open_files(us).left() | FILES[7])
            & (self.semi_open_files(us).right() | FILES[0]);
        let isolated_count = isolated.count() as i32;
        total += ISOLATED_PAWN_PENALTY * isolated_count;

        trace.add(|t| {
            t.phalanx_pawn += perspective * phalanx_count;
            t.protected_pawn += perspective * protected_count;
            t.isolated_pawn += perspective * isolated_count
        });

        total
    }
}

#[cfg(test)]
mod tests {
    use crate::evaluate::tuner::NullTrace;

    use super::*;
    use chess::square::Square::*;

    #[test]
    fn passers() {
        let board: Board = "8/8/8/p3kPp1/6P1/4K3/8/8 w - - 0 1".parse().unwrap();
        let pawn_structure = PawnStructure::new(&board, &mut NullTrace);
        assert_eq!(pawn_structure.passed_pawns(White), Bitboard::from(F5));
        assert_eq!(pawn_structure.passed_pawns(Black), Bitboard::from(A5));
    }

    #[test]
    fn passers2() {
        let board: Board = "r1bq1bnr/p1pp1kpp/p7/8/1n2P3/8/PPP2PPP/RNBQK1NR w KQ - 0 7".parse().unwrap();
        let pawn_structure = PawnStructure::new(&board, &mut NullTrace);
        assert_eq!(pawn_structure.passed_pawns(White), Bitboard::EMPTY);
        assert_eq!(pawn_structure.passed_pawns(Black), Bitboard::EMPTY);
    }
}
