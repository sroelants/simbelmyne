use std::fmt::Display;
use crate::{board::{Piece, Board, PieceType, Color }, bitboard::Step};
use std::iter::successors;
use crate::bitboard::Bitboard;
use itertools::Itertools;
use crate::board::Square;

/// Pack all the metadata related to a Move in a u16
///
/// 6 bits (0 - 63) for the source square
/// 6 bits (0 - 63) for the target square
/// 4 bits (0 - 16) for additional metadata (castling, captures, promotions)
/// When we get to move sorting, to we also want to squeeze in the sorting rank
/// here? 
/// cf. Rustic https://github.com/mvanthoor/rustic/blob/17b15a34b68000dffb681277c3ef6fc98f935a0b/src/movegen/defs.rs
/// cf. Carp https://github.com/dede1751/carp/blob/main/chess/src/moves.rs
#[derive(Default, Debug, Copy, Clone)]
pub struct Move(u16);

impl Move {
    pub const SRC_MASK: u16        = 0b0000_000000_111111;
    pub const TGT_MASK: u16        = 0b0000_111111_000000;
    pub const CASTLE_MASK: u16     = 0b0001_000000_000000;

    pub fn new(src: Square, tgt: Square) -> Move {
        let mut value = 0u16;
        value |= src as u16;
        value |= (tgt as u16) << 6;

        Move(value)
    }

    pub fn src(self) -> Square {
        ((self.0 & Self::SRC_MASK) as usize).into()
    }

    pub fn tgt(self) -> Square {
        (((self.0 & Self::TGT_MASK) >> 6) as usize).into()
    }

    pub fn is_castle(self) -> bool {
        self.0 & Self::CASTLE_MASK != 0
    }

    pub fn set_castle(&mut self) {
        self.0 |= Self::CASTLE_MASK;
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.src().to_alg())?;
        write!(f, "{}", self.tgt().to_alg())?;

        if self.is_castle() {
            write!(f, " (Castle)")?;
        }

        Ok(())
    }
}

impl Piece {
    pub fn range(&self) -> usize {
        use PieceType::*;

        match self.piece_type() {
            Pawn | Knight | King => 1,
            _ => 7
        }
    }

    pub fn directions(&self) -> Vec<Step> {
        use PieceType::*;

        match self.piece_type() {
            Pawn => vec![
                Step::forward(self.color) + Step::LEFT, 
                Step::forward(self.color) + Step::RIGHT
            ],

            Rook => Step::ORTHO_DIRS.to_vec(),

            Knight => vec![
                Step::new( 1,  2),
                Step::new( 1, -2),
                Step::new(-1,  2),
                Step::new(-1, -2),
                Step::new( 2,  1),
                Step::new( 2, -1),
                Step::new(-2,  1),
                Step::new(-2, -1),
            ],

            Bishop => Step::DIAG_DIRS.to_vec(),

            King | Queen => vec![
                Step::ORTHO_DIRS,
                Step::DIAG_DIRS
            ].concat(),
        }
    }

    //TODO: Add dedicated pawn branch here
    pub fn visible_squares(&self, blockers: Bitboard) -> Bitboard {
        let mut visible = Bitboard::default();

        for step in self.directions() {
            visible |= successors(Some(self.position), |pos| pos.offset(step))
            .skip(1)
            .take(self.range())
            .take_while_inclusive(|&pos| !blockers.contains(pos))
            .collect()
        }

        visible
    }

    pub fn visible_rays(&self, blockers: Bitboard) -> Vec<Bitboard> {
        self.directions()
            .into_iter()
            .map(|step| successors(Some(self.position), |pos| pos.offset(step))
                .skip(1)
                .take(self.range())
                .take_while_inclusive(|&pos| !blockers.contains(pos))
                .collect()
        ).collect()
    }

    pub fn pseudolegal_moves(&self, board: &Board) -> Vec<Move> {
        use PieceType::*;
        let king_bb = board.piece_bbs[King as usize] 
            & board.occupied_by(self.color);
        let blockers = board.all_occupied();
        let opp = self.color().opp();
        let checkers = board.checkers[opp as usize];
        let in_check = !checkers.is_empty();
        let in_double_check = in_check && checkers.count_ones() > 1;

        // When there's more than one piece giving check, there's no other option
        // but for the king to move out of check.
        if in_double_check && self.piece_type != King {
            return Vec::new();
        }

        // Get all "visible" squares for the piece. That is, all squares a 
        // piece can see up until (and including) the first blocking piece.
        let mut targets: Bitboard = match self.piece_type() {
            Pawn => pawn_pushes(self.position, self.color, blockers),
            _ => self.visible_squares(blockers)
        };

        // The king can't move into an attacked square
        if self.piece_type() == King {
            targets &= !board.king_danger_squares[opp as usize]
        }

        // If we're in check, capturing or blocking is the only valid option
        if in_check && self.piece_type != King {
            let checker = board.piece_list[Square::from(checkers) as usize]
                .expect("There is a checking piece on this square");

            let check_ray = checker.visible_rays(blockers)
                .into_iter()
                .find(|ray| ray.contains(king_bb))
                .expect("Checker has at exactly one checking ray");

            targets &= checkers | check_ray;
        }

        //TODO:  Pins
        // Calculate pin rays at the start of every halfround (same as attacked, 
        // checkers, etc...)
        // Steps are as follows:
        // 1. Cast rays out from king, blocked only by _opponent_ pieces
        // 2. Are there any reciprocal pieces on the ray?
        //    If so, it's a potential pin ray
        // 3. For each potential pin ray, check whether it contains exactly one
        //    of my pieces.
        //
        // Then, checking whether or not I'm pinned comes down to searching the
        // list of pinrays for one that contains my piece (Should I also store
        // the accumulated pinrays for easier checks? Rather than having every
        // piece iterate through the entire vector?)

        let mut moves: Vec<Move> = targets.into_iter()
            .map(|tgt| Move::new(self.position.into(), tgt))
            .collect();
 
        // Add available castles
        if self.piece_type() == King {
            moves.extend(
                board.castling_rights.get_available(self.color)
                    .into_iter()
                    .filter(|ctype| ctype.is_allowed(board))
                    .map(|ctype| ctype.king_move())
            )
        }

        moves
    }
}

impl Board {
    pub fn legal_moves(&self) -> Vec<Move> {
        use PieceType::*;
        let king_bb = self.get_bb(King, self.current_player);
        let our_pieces = self.occupied_by(self.current_player);
        let blockers = self.all_occupied();
        let opp = self.current_player.opp();
        let checkers = self.compute_checkers(self.current_player);
        let in_check = !checkers.is_empty();
        let in_double_check = in_check && checkers.count_ones() > 1;
        let attacked_squares = self.compute_attacked_by(self.current_player, blockers);
        let pinrays = self.compute_pinrays(self.current_player);
        let pinned_pieces = our_pieces & pinrays.iter().collect();

        let mut legal_moves: Vec<Move> = Vec::new();

        for square in our_pieces {
            let piece = self.get_at(square).expect("Square should hold a piece");
            let src: Square = piece.position.into();

            // When there's more than one piece giving check, there's no other 
            // option but for the king to move out of check.
            if in_double_check && piece.piece_type() != King {
                continue;
            }

            // FIXME: Pawn attacks???
            let mut pseudos: Bitboard = match piece.piece_type() {
                Pawn => pawn_pushes(piece.position, piece.color, blockers),
                _ => piece.visible_squares(blockers)
            }.remove(our_pieces);

            // The king can't move into an attacked square
            if piece.piece_type() == King {
                pseudos &= !self.king_danger_squares(opp)
            }

            // If we're in check, capturing or blocking is the only valid option
            if in_check && piece.piece_type() != King {
                let checker = self.piece_list[Square::from(checkers) as usize]
                    .expect("There is a checking piece on this square");

                let check_ray = checker.visible_rays(blockers)
                    .into_iter()
                    .find(|ray| ray.contains(king_bb))
                    .expect("Checker has at exactly one checking ray");

                pseudos &= checkers | check_ray;
            }

            // If we're pinned, we can only move within our pin ray
            if pinned_pieces.contains(piece.position) {
                let pinray = pinrays.iter()
                    .find(|ray| ray.contains(piece.position))
                    .expect("A pinned piece should lie on a pinray")
                    .to_owned();

                pseudos &= pinray;
            }

            legal_moves.extend(
                pseudos.into_iter().map(|tgt| Move::new(src, tgt))
            )
        }

        // Add available castles at the end
        legal_moves.extend(
            self.castling_rights.get_available(self.current_player)
                .into_iter()
                .filter(|ctype| ctype.is_allowed(self))
                .map(|ctype| ctype.king_move())
        );

        legal_moves
    }
}

fn pawn_pushes(position: Bitboard, side: Color, blockers: Bitboard) -> Bitboard {
    let forward = successors(
        Some(position), 
        |pos| pos.offset(Step::forward(side))
    );

    forward
        .skip(1)
        .take(if position.on_pawn_rank(side) { 2 } else { 1 })
        .take_while_inclusive(|&pos| !blockers.contains(pos))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn src_works() {
        let src = Square::new(3,4);
        let tgt = Square::new(4,5);

        let mv = Move::new(src,tgt);
        assert_eq!(mv.src(), src.into(), "mv.src() should return the source position, as a bitboard");
    }

    #[test]
    fn tgt_works() {
        let src = Square::new(3,4);
        let tgt = Square::new(4,5);

        let mv = Move::new(src,tgt);
        assert_eq!(mv.tgt(), tgt.into(), "mv.tgt() should return the source target, as a bitboard");
    }

    #[test]
    fn castling_bit() {
        let src = Square::new(3,4);
        let tgt = Square::new(4,5);

        let mut mv = Move::new(src,tgt);
        assert!(!mv.is_castle(), "is_castle returns false for a normal move");

        mv.set_castle();
        assert!(mv.is_castle(), "is_castle returns true after setting the castle bit");
    }
}
