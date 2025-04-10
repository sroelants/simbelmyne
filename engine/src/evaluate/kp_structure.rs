use chess::bitboard::Bitboard;
use chess::board::Board;
use chess::piece::Color;
use chess::piece::Color::*;

use crate::evaluate::lookups::PASSED_PAWN_MASKS;
use super::params::PARAMS;

use super::lookups::FILES;
use super::tuner::{EvalTrace, Tracer};
use super::S;

const WHITE: bool = true;
const BLACK: bool = false;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct KingPawnStructure {
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

impl Default for KingPawnStructure {
    fn default() -> Self {
        Self {
            score: S::default(),
            passed_pawns: [Bitboard::EMPTY, Bitboard::EMPTY],
            semi_open_files: [!Bitboard::EMPTY, !Bitboard::EMPTY],
            outposts: [Bitboard::EMPTY, Bitboard::EMPTY]
        }
    }
}

impl KingPawnStructure {
    pub fn new(board: &Board, mut trace: &mut impl Tracer<EvalTrace>) -> Self {
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

        let mut kp_structure = Self {
            score: S::default(),
            passed_pawns: [white_passers, black_passers],
            semi_open_files: [white_semi_open_files, black_semi_open_files],
            outposts: [white_outposts, black_outposts]
        };

        kp_structure.score = kp_structure.compute_score::<WHITE>(board, trace) 
            - kp_structure.compute_score::<BLACK>(board, trace);

        kp_structure
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

    pub fn compute_score<const WHITE: bool>(&self, board: &Board, trace: &mut impl Tracer<EvalTrace>) -> S {
        let mut total = S::default();
        let us = if WHITE { White } else { Black };
        let perspective = if WHITE { 1 } else { -1 };
        let our_pawns = board.pawns(us);
        let their_pawns = board.pawns(!us);
        let our_king = board.kings(us).first();
        let their_king = board.kings(!us).first();

        let shield_mask = PASSED_PAWN_MASKS[us][our_king];
        let storm_mask = PASSED_PAWN_MASKS[!us][their_king];
        let doubled_mask = our_pawns.backward::<WHITE>() & !board.pawn_attacks(!us);
        let phalanx_mask = our_pawns.left() | our_pawns.right();
        let protected_mask = board.pawn_attacks(us);
        let isolated_mask = (self.semi_open_files(us).left() | FILES[7])
            & (self.semi_open_files(us).right() | FILES[0]);


        for sq in our_pawns {
            let rank = sq.relative_rank::<WHITE>();

            if self.passed_pawns(us).contains(sq) {
                let push_sq = sq.forward(us).unwrap();

                // Passed pawn bonus
                let rel_sq = if WHITE { sq.flip() } else { sq };
                total += PARAMS.passed_pawn[rel_sq];
                trace.add(|t| t.passed_pawn[rel_sq] += perspective);

                // Distance to friendly king
                let our_king_dist = push_sq.max_dist(our_king);
                total += PARAMS.passers_friendly_king[our_king_dist];
                trace.add(|t| t.passers_friendly_king[our_king_dist] += perspective);

                // Distance to enemy king
                let their_king_dist = push_sq.max_dist(their_king);
                total += PARAMS.passers_enemy_king[their_king_dist];
                trace.add(|t| t.passers_enemy_king[their_king_dist] += perspective);
            }

            if storm_mask.contains(sq) {
                let king_dist = sq.vdistance(their_king).min(3);
                total += PARAMS.pawn_storm[king_dist - 1];
                trace.add(|t| t.pawn_storm[king_dist - 1] += perspective);
            }

            if shield_mask.contains(sq) {
                let king_dist = sq.vdistance(our_king).min(3);
                total += PARAMS.pawn_shield[king_dist - 1];
                trace.add(|t| t.pawn_shield[king_dist - 1] += perspective);
            }

            if doubled_mask.contains(sq) {
                total += PARAMS.doubled_pawn[rank];
                trace.add(|t| t.doubled_pawn[rank] += perspective);
            }

            if phalanx_mask.contains(sq) {
                total += PARAMS.phalanx_pawn[rank];
                trace.add(|t| t.phalanx_pawn[rank] += perspective);
            }

            if protected_mask.contains(sq) {
                total += PARAMS.protected_pawn[rank];
                trace.add(|t| t.protected_pawn[rank] += perspective);
            }

            if isolated_mask.contains(sq) {
                total += PARAMS.isolated_pawn[rank];
                trace.add(|t| t.isolated_pawn[rank] += perspective);
            }
        }

        total
    }
}

#[cfg(test)]
mod tests {
    use crate::evaluate::tuner::NullTracer;

    use super::*;
    use chess::square::Square::*;

    #[test]
    fn passers() {
        let board: Board = "8/8/8/p3kPp1/6P1/4K3/8/8 w - - 0 1".parse().unwrap();
        let kp_structure = KingPawnStructure::new(&board, &mut NullTracer);
        assert_eq!(kp_structure.passed_pawns(White), Bitboard::from(F5));
        assert_eq!(kp_structure.passed_pawns(Black), Bitboard::from(A5));
    }

    #[test]
    fn passers2() {
        let board: Board = "r1bq1bnr/p1pp1kpp/p7/8/1n2P3/8/PPP2PPP/RNBQK1NR w KQ - 0 7".parse().unwrap();
        let kp_structure = KingPawnStructure::new(&board, &mut NullTracer);
        assert_eq!(kp_structure.passed_pawns(White), Bitboard::EMPTY);
        assert_eq!(kp_structure.passed_pawns(Black), Bitboard::EMPTY);
    }
}
