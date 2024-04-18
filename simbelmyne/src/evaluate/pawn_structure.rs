use chess::bitboard::Bitboard;
use chess::board::Board;
use chess::piece::Color;
use chess::piece::Color::*;

use crate::evaluate::lookups::PASSED_PAWN_MASKS;

use super::lookups::{DOUBLED_PAWN_MASKS, FILES, ISOLATED_PAWN_MASKS};
use super::params::{CONNECTED_PAWN_BONUS, DOUBLED_PAWN_PENALTY, ISOLATED_PAWN_PENALTY, PASSED_PAWN_TABLE, PHALANX_PAWN_BONUS};
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
}

impl PawnStructure {
    pub fn new(board: &Board) -> Self {
        let white_pawns = board.pawns(White);
        let black_pawns = board.pawns(Black);

        let white_attacks = ((white_pawns & !FILES[0]) << 7) 
            | ((white_pawns & !FILES[7]) << 9);

        let black_attacks = ((black_pawns & !FILES[0]) >> 7) 
            | ((white_pawns & !FILES[7]) >> 9);

        let white_passers = white_pawns
            .filter(|&pawn| {
                let mask = PASSED_PAWN_MASKS[White as usize][pawn as usize];
                (mask & black_pawns).is_empty()
            })
            .collect::<Bitboard>();

        let black_passers = black_pawns
            .filter(|&pawn| {
                let mask = PASSED_PAWN_MASKS[Black as usize][pawn as usize];
                (mask & white_pawns).is_empty()
            })
            .collect::<Bitboard>();

        let white_semi_open_files = FILES[0..7].iter()
            .filter(|&&file| {
                (file & white_pawns).is_empty()
            })
            .collect::<Bitboard>();

        let black_semi_open_files = FILES[0..7].iter()
            .filter(|&&file| {
                (file & black_pawns).is_empty()
            })
            .collect::<Bitboard>();

        let white_blocked_pawns = white_pawns & black_pawns >> 8;
        let black_blocked_pawns = black_pawns & white_pawns << 8;

        let mut pawn_structure = Self {
            score: S::default(),
            pawns: [ white_pawns, black_pawns],
            pawn_attacks: [white_attacks, black_attacks],
            passed_pawns: [white_passers, black_passers],
            semi_open_files: [white_semi_open_files, black_semi_open_files],
            blocked_pawns: [white_blocked_pawns, black_blocked_pawns],
        };

        pawn_structure.score = pawn_structure.compute_score::<WHITE>() - pawn_structure.compute_score::<BLACK>();

        pawn_structure
    }

    pub fn score(&self) -> S {
        self.score
    }

    pub fn pawns(&self, us: Color) -> Bitboard {
        self.pawns[us as usize]
    }

    pub fn pawn_attacks(&self, us: Color) -> Bitboard {
        self.pawn_attacks[us as usize]
    }

    pub fn passed_pawns(&self, us: Color) -> Bitboard {
        self.passed_pawns[us as usize]
    }

    pub fn semi_open_files(&self, us: Color) -> Bitboard {
        self.semi_open_files[us as usize]
    }

    pub fn open_files(&self) -> Bitboard {
        self.semi_open_files(White) & self.semi_open_files(Black)
    }

    pub fn blocked_pawns(&self, us: Color) -> Bitboard {
        self.blocked_pawns[us as usize]
    }

    pub fn compute_score<const WHITE: bool>(&self) -> S {
        let mut total = S::default();

        let us = if WHITE { White } else { Black };
        let our_pawns = self.pawns(us);
        let their_pawns = self.pawns(!us);

        for sq in our_pawns {
            // Passed pawns
            let passed_mask = PASSED_PAWN_MASKS[us as usize][sq as usize];
            if their_pawns & passed_mask == Bitboard::EMPTY {
                let sq = if us.is_white() { sq.flip() } else { sq };
                total += PASSED_PAWN_TABLE[sq as usize];
            }

            // Connected pawns
            let connected = (our_pawns & sq.pawn_attacks(us)).count();
            total += CONNECTED_PAWN_BONUS[connected as usize];

            // Phalanx pawns
            let neighbors = Bitboard::from(sq.left()) | Bitboard::from(sq.right());
            let phalanx_pawns = our_pawns & neighbors;
            let phalanx_count = phalanx_pawns.count();
            total += PHALANX_PAWN_BONUS[phalanx_count as usize];

            // Isolated pawns
            let isolated_mask = ISOLATED_PAWN_MASKS[sq as usize];
            if our_pawns & isolated_mask == Bitboard::EMPTY {
                total += ISOLATED_PAWN_PENALTY;
            }

            // Doubled pawns
            // FIXME: Doesn't seem to be correct?
            // let is_doubled = (our_pawns & FILES[sq as usize]).count() > 1;
            // if is_doubled {
            //     total += DOUBLED_PAWN_PENALTY;
            // }
        }

        // Doubled pawns
        for mask in DOUBLED_PAWN_MASKS {
            let doubled = (our_pawns & mask).count().saturating_sub(1) as Score;
            total += DOUBLED_PAWN_PENALTY * doubled;
        }

        total
    }
}
