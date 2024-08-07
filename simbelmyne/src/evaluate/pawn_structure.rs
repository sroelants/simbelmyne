use chess::bitboard::Bitboard;
use chess::board::Board;
use chess::piece::Color;
use chess::piece::Color::*;

use crate::evaluate::lookups::PASSED_PAWN_MASKS;

use super::lookups::{DOUBLED_PAWN_MASKS, FILES};
use super::params::{DOUBLED_PAWN_PENALTY, ISOLATED_PAWN_PENALTY, PASSED_PAWN_TABLE, PHALANX_PAWN_BONUS, PROTECTED_PAWN_BONUS};
use super::tuner::EvalTrace;
use super::{Score, S};

const WHITE: bool = true;
const BLACK: bool = false;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct PawnStructure {
    /// The score associated with the pawn structure
    score: S,

    /// Pawn bitboards for White and Black
    pawns: [Bitboard; Color::COUNT],

    /// Pawn attacks bitboards for White and Black
    pawn_attacks: [Bitboard; Color::COUNT],

    /// Passed pawn bitboards for White and Black
    passed_pawns: [Bitboard; Color::COUNT],

    /// Semi-open file bitboards for White and Black
    semi_open_files: [Bitboard; Color::COUNT],

    /// Blocked pawns bitboards for White and Black
    blocked_pawns: [Bitboard; Color::COUNT],

    /// Outpost squares
    /// Squares that can't be attacked (easily) by opponent pawns, and are
    /// defended by one of our pawns
    outposts: [Bitboard; Color::COUNT],
}

impl PawnStructure {
    pub fn new(board: &Board) -> Self {
        // Pawn bitboardds
        let white_pawns = board.pawns(White);
        let black_pawns = board.pawns(Black);

        // Pawns attacks
        let white_left_attacks = white_pawns.forward_left::<WHITE>();
        let white_right_attacks = white_pawns.forward_right::<WHITE>();
        let white_attacks = white_left_attacks | white_right_attacks;

        let black_left_attacks = black_pawns.forward_left::<BLACK>();
        let black_right_attacks = black_pawns.forward_right::<BLACK>();
        let black_attacks = black_left_attacks | black_right_attacks;

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
            pawns: [white_pawns, black_pawns],
            pawn_attacks: [white_attacks, black_attacks],
            passed_pawns: [white_passers, black_passers],
            semi_open_files: [white_semi_open_files, black_semi_open_files],
            blocked_pawns: [white_blocked_pawns, black_blocked_pawns],
            outposts: [white_outposts, black_outposts]
        };

        pawn_structure.score = pawn_structure.compute_score::<WHITE>(None) 
            - pawn_structure.compute_score::<BLACK>(None);

        pawn_structure
    }

    pub fn score(&self) -> S {
        self.score
    }

    pub fn pawns(&self, us: Color) -> Bitboard {
        self.pawns[us]
    }

    pub fn pawn_attacks(&self, us: Color) -> Bitboard {
        self.pawn_attacks[us]
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

    pub fn blocked_pawns(&self, us: Color) -> Bitboard {
        self.blocked_pawns[us]
    }

    pub fn outposts(&self, us: Color) -> Bitboard {
        self.outposts[us]
    }

    pub fn compute_score<const WHITE: bool>(&self, mut trace: Option<&mut EvalTrace>) -> S {
        let mut total = S::default();
        let us = if WHITE { White } else { Black };
        let perspective = if WHITE { 1 } else { -1 };
        let our_pawns = self.pawns(us);

        // Passed pawns
        for sq in self.passed_pawns(us) {
            let sq = if WHITE { sq.flip() } else { sq };
            total += PASSED_PAWN_TABLE[sq];

            #[cfg(feature = "texel")]
            if let Some(ref mut trace) = trace  {
                trace.passed_pawn[sq] += perspective;
            }
        }

        // Doubled pawns
        for mask in DOUBLED_PAWN_MASKS {
            let doubled = (our_pawns & mask).count().saturating_sub(1) as Score;
            total += DOUBLED_PAWN_PENALTY * doubled;

            #[cfg(feature = "texel")]
            if let Some(ref mut trace) = trace  {
                trace.doubled_pawn += perspective * doubled
            }
        }

        // Phalanx pawns
        let phalanx_pawns = our_pawns & (our_pawns.left() | our_pawns.right());
        let phalanx_count = phalanx_pawns.count() as i32;
        total += PHALANX_PAWN_BONUS * phalanx_count;

        // Connected pawns
        let protected_pawns = our_pawns & self.pawn_attacks(us);
        let protected_count = protected_pawns.count() as i32;
        total += PROTECTED_PAWN_BONUS * protected_count;

        #[cfg(feature = "texel")]
        if let Some(ref mut trace) = trace  {
        }

        // Isolated pawns
        let isolated = our_pawns 
            & (self.semi_open_files(us).left() | FILES[7])
            & (self.semi_open_files(us).right() | FILES[0]);
        let isolated_count = isolated.count() as i32;
        total += ISOLATED_PAWN_PENALTY * isolated_count;

        #[cfg(feature = "texel")]
        if let Some(ref mut trace) = trace  {
            trace.phalanx_pawn += perspective * phalanx_count;
            trace.protected_pawn += perspective * protected_count;
            trace.isolated_pawn += perspective * isolated_count
        }

        total
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chess::square::Square::*;

    #[test]
    fn passers() {
        let board: Board = "8/8/8/p3kPp1/6P1/4K3/8/8 w - - 0 1".parse().unwrap();
        let pawn_structure = PawnStructure::new(&board);
        assert_eq!(pawn_structure.passed_pawns(White), Bitboard::from(F5));
        assert_eq!(pawn_structure.passed_pawns(Black), Bitboard::from(A5));
    }

    #[test]
    fn passers2() {
        let board: Board = "r1bq1bnr/p1pp1kpp/p7/8/1n2P3/8/PPP2PPP/RNBQK1NR w KQ - 0 7".parse().unwrap();
        let pawn_structure = PawnStructure::new(&board);
        assert_eq!(pawn_structure.passed_pawns(White), Bitboard::EMPTY);
        assert_eq!(pawn_structure.passed_pawns(Black), Bitboard::EMPTY);
    }

    #[test]
    fn pawn_attacks() {
        let board: Board = "8/7p/8/p3kPp1/P5P1/4K3/7P/8 w - - 0 1".parse().unwrap();
        let pawn_structure = PawnStructure::new(&board);
        assert_eq!(pawn_structure.pawn_attacks(White), Bitboard(0x50a200400000));
        assert_eq!(pawn_structure.pawn_attacks(Black), Bitboard(0x4000a2000000));
    }


}
