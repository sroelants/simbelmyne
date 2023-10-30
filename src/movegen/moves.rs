use std::fmt::Display;
use crate::{board::{Piece, Board, PieceType}, bitboard::Step};
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
    pub const DPUSH_MASK: u16      = 0b0010_000000_000000;
    pub const EP_MASK: u16         = 0b0100_000000_000000;


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

    pub fn is_double_push(self) -> bool {
        self.0 & Self::DPUSH_MASK != 0
    }

    pub fn set_double_push(&mut self) {
        self.0 |= Self::DPUSH_MASK;
    }

    pub fn is_en_passant(self) -> bool {
        self.0 & Self::EP_MASK != 0
    }

    pub fn set_en_passant(&mut self) {
        self.0 |= Self::EP_MASK;
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
            Pawn => {
                let hasnt_moved = self.position.on_pawn_rank(self.color());
                if hasnt_moved { 2 } else { 1 }
            }
            Knight | King => 1,
            _ => 7 // The entire board
        }
    }

    pub fn directions(&self) -> Vec<Step> {
        use PieceType::*;

        match self.piece_type() {
            Pawn => vec![Step::forward(self.color)],

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

    pub fn visible_squares(&self, ours: Bitboard, theirs: Bitboard) -> Bitboard {
        let mut visible = Bitboard::default();
        let blockers = ours | theirs;

        for step in self.directions() {
            visible |= successors(Some(self.position), |pos| pos.offset(step))
            .skip(1)
            .take(self.range())
            .take_while_inclusive(|&pos| !blockers.contains(pos))
            .collect()
        }

        if self.piece_type == PieceType::Pawn {
            // 1. Remove captures as result of pawn pushes
            visible &= !theirs;

            // 2. Add pawn diagonal attacks
            let forward = Step::forward(self.color());
            let captures = [forward + Step::LEFT, forward + Step::RIGHT]
                .into_iter()
                .map(|dir| self.position.offset(dir))
                .flatten()
                .collect::<Bitboard>() 
                & theirs;

            visible |= captures;
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
}

impl Board {
    pub fn legal_moves(&self) -> Vec<Move> {
        use PieceType::*;
        let player = self.current;
        let opp = player.opp();
        let king_bb = self.get_bb(King, player);
        let our_pieces = self.occupied_by(player);
        let their_pieces = self.occupied_by(opp);
        let blockers = our_pieces | their_pieces;
        let checkers = self.compute_checkers(player);
        let in_check = !checkers.is_empty();
        let in_double_check = in_check && checkers.count_ones() > 1;
        let pinrays = self.compute_pinrays(player);
        let pinned_pieces = our_pieces & pinrays.iter().collect();

        let mut legal_moves: Vec<Move> = Vec::new();

        for source in our_pieces {
            let piece = self.get_at(source).expect("Source should hold a piece");

            // When there's more than one piece giving check, there's no other 
            // option but for the king to move out of check.
            if in_double_check && piece.piece_type() != King {
                continue;
            }

            let mut pseudos: Bitboard = piece
                .visible_squares(our_pieces, their_pieces);

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
            // TODO: Can we do this more efficiently? If the piece is pinned, 
            // don't even bother computing _all_ the legal moves, just check 
            // whether they can move within the pin ray and return it if so.
            if pinned_pieces.contains(piece.position) {
                let pinray = pinrays.iter()
                    .find(|ray| ray.contains(piece.position))
                    .expect("A pinned piece should lie on a pinray")
                    .to_owned();

                pseudos &= pinray;
            }

            // Add remaining pseudolegal moves to legal moves
            for target in pseudos {
                let mut mv = Move::new(source, target);

                // Flag pawn double pushes
                if piece.piece_type() == Pawn && Square::is_double_push(source, target) {
                    mv.set_double_push()
                }

                legal_moves.push(mv);
            }

            // Add potential en-passant moves, after making sure they don't lead
            // to discovered checks
            if let Some(en_passant) = self.en_passant {
                let ep_bb = en_passant.into();
                if piece.piece_type != Pawn { continue; }

                let can_capture = piece
                    .visible_squares(our_pieces, ep_bb) 
                    .contains(ep_bb);

                // If we can't capture en-passant in the first place, bail
                if !can_capture { continue; }

                let captured_bb: Bitboard = en_passant.forward(opp)
                    .expect("Double-push pawn can't be out-of-bounds")
                    .into();

                // If the EP would reveal a discovered check, bail.
                if self.is_xray_check(player, piece.position | captured_bb ) {
                    continue;
                }

                // Finally, add the move
                let mut mv = Move::new(source, en_passant);
                mv.set_en_passant();
                legal_moves.push(mv);
            }
        }

        // Add available castles at the end
        legal_moves.extend(
            self.castling_rights.get_available(self.current)
                .into_iter()
                .filter(|ctype| ctype.is_allowed(self))
                .map(|ctype| ctype.king_move())
        );

        // Add potential en-passant moves at the end

        legal_moves
    }
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
