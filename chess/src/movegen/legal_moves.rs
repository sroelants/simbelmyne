//! Find all the legal moves for a given board state
//!
//! Moves are generated by piece type, so we can shortcut, e.g. when only the
//! king is allowed to move.
//!
//! All the move generating functions are parametrized by a `QUIETS` const
//! generic that decides whether or not to include quiet moves.

use crate::constants::RANKS;
use crate::square::Square;
use crate::{
    bitboard::Bitboard,
    movegen::moves::MoveType,
};
use crate::board::Board;
use crate::movegen::lookups::{BETWEEN, RAYS};
use crate::movegen::moves::Move;
use crate::piece::PieceType;

use super::moves::BareMove;

const NO_KING: bool = false;
const QUIETS: bool = true;

impl Board {
    /// Find all the legal moves for the current board state
    pub fn legal_moves<const QUIETS: bool>(&self) -> Vec<Move> {
        let us = self.current;
        let checkers = self.checkers();
        let pinrays = self.pinrays[us as usize];
        let mut moves: Vec<Move> = Vec::with_capacity(50);

        // Add the king moves to the list of legal moves
        for square in self.kings(us) {
            self.king_moves::<QUIETS>(square, &mut moves);
        }

        // If we're in double check, only king moves are valid, so we exit 
        // early.
        if checkers.count() > 1 {
            return moves;
        }

        // Add the pawn moves to the list of legal moves
        for square in self.pawns(us) {
            self.pawn_moves::<QUIETS>(square, &mut moves, checkers, pinrays);
        }

        // Add the remaining piece moves to the list of legal moves
        for square in self.pieces(us) {
            self.piece_moves::<QUIETS>(square, &mut moves, checkers, pinrays);
        }

        moves
    }

    /// Find all the legal pawn moves for a given square
    ///
    /// Add the legal moves to the provided `moves` buffer.
    /// This is by far the most complex of the move helpers, since it needs to
    /// account for en-passant and promotions, on top of the usual check and
    /// pinning logic
    fn pawn_moves<const QUIETS: bool>(
        &self, 
        square: Square, 
        moves: &mut Vec<Move>, 
        checkers: Bitboard, 
        pinrays: Bitboard,
    ) {
        use MoveType::*;
        let us = self.current;
        let ours = self.occupied_by(us);
        let theirs = self.occupied_by(!us);
        let blockers = ours | theirs;
        let king_sq = self.kings(us).first();
        let in_check = checkers.count() > 0;
        let pinned_pieces = ours & pinrays;
        let is_pinned = pinned_pieces.contains(square);

        let mut visible = square.pawn_squares(us, blockers) 
            | square.pawn_attacks(us) & theirs;

        // If we're pinned, we can't move outside of our pin-ray
        if is_pinned {
            let pinray = pinrays & RAYS[king_sq as usize][square as usize];
            visible &= pinray;
        }


        ////////////////////////////////////////////////////////////////////////
        //
        // Captures (including promotions & en-passant)
        //
        ////////////////////////////////////////////////////////////////////////

        let mut captures = visible & theirs;

        if in_check {
            captures &= checkers;
        }

        // Push a move for every target square
        for target in captures {
            if target.is_promo_rank(us) {
                moves.push(Move::new(square, target, KnightPromoCapture));
                moves.push(Move::new(square, target, BishopPromoCapture));
                moves.push(Move::new(square, target, RookPromoCapture));
                moves.push(Move::new(square, target, QueenPromoCapture));
            } else {
                moves.push(Move::new(square, target, Capture));
            }
        }

        // Add potential en-passant moves, after making sure they don't lead
        // to discovered checks
        if self.en_passant.is_some() && !is_pinned {
            self.en_passant_move(square, moves, checkers);
        }

        // If we only want tacticals, we're done here.
        if !QUIETS {
            return;
        }

        ////////////////////////////////////////////////////////////////////////
        //
        // Quiets (pushes & promotions)
        //
        ////////////////////////////////////////////////////////////////////////
        
        let mut quiets = visible & !blockers;

        // If we're in check, blocking is the only valid option
        if in_check {
            let checker_sq = checkers.first();
            quiets &= BETWEEN[checker_sq as usize][king_sq as usize];
        }

        // Push a move for every target square
        for target in quiets {
            if target.is_promo_rank(us) {
                moves.push(Move::new(square, target, KnightPromo));
                moves.push(Move::new(square, target, BishopPromo));
                moves.push(Move::new(square, target, RookPromo));
                moves.push(Move::new(square, target, QueenPromo));
            } else if square.distance(target) == 2 {
                moves.push(Move::new(square, target, DoublePush));
            } else {
                moves.push(Move::new(square, target, Quiet));
            }
        }
    }

    /// Find all the legal king moves
    ///
    /// Add the legal moves to the provided `moves` buffer.
    /// Probably the easiest among the move generating helpers, since kings
    /// don't need to deal with checks or pins.
    fn king_moves<const QUIETS: bool>(
        &self,
        square: Square,
        moves: &mut Vec<Move>,
    ) {
        use MoveType::*;
        let us = self.current;
        let them = !us;
        let ours = self.occupied_by(us);
        let theirs = self.occupied_by(them);
        let blockers = ours | theirs;

        let mut visible = square.king_squares();

        // King can only move to squares that aren't attacked
        // The NO_KING parameter removes the king itself before calculating the
        // attacked squares, to make sure the king's not blocking any attacks.
        visible &= !self.attacked_by::<NO_KING>(them);

        ////////////////////////////////////////////////////////////////////////
        //
        // Captures
        //
        ////////////////////////////////////////////////////////////////////////

        let captures = visible & theirs;

        for target in captures {
            moves.push(Move::new(square, target, Capture));
        }

        // If we only want tacticals, we're done here.
        if !QUIETS {
            return;
        }

        ////////////////////////////////////////////////////////////////////////
        //
        // Quiets
        //
        ////////////////////////////////////////////////////////////////////////

        let quiets = visible & !blockers;

        for target in quiets {
            moves.push(Move::new(square, target, Quiet));
        }

        // Add castling moves
        for ctype in self.castling_rights.get_available(us) {
            if self.castle_allowed(ctype) {
                moves.push(ctype.king_move());
            }
        }
    }

    /// Find all the legal moves for pieces other than pawns or kings
    /// 
    /// Add the legal moves to the provided `moves` buffer.
    fn piece_moves<const QUIETS: bool>(
        &self, 
        square: Square,
        moves: &mut Vec<Move>, 
        checkers: Bitboard, 
        pinrays: Bitboard
    ) {
        use PieceType::*;
        use MoveType::*;
        let us = self.current;
        let king_sq = self.kings(us).first();
        let ours = self.occupied_by(us);
        let theirs = self.occupied_by(!us);
        let blockers = ours | theirs;
        let in_check = checkers.count() > 0;
        let pinned_pieces = ours & pinrays;
        let is_pinned = pinned_pieces.contains(square);
        let piece = self.get_at(square).unwrap();

        let mut visible = match piece.piece_type() {
            Knight => square.knight_squares(),
            Bishop => square.bishop_squares(blockers),
            Rook => square.rook_squares(blockers),
            Queen => square.queen_squares(blockers),
            _ => unreachable!()
        };

        // If we're pinned, we can't move outside of our pin-ray
        if is_pinned {
            let pinray = pinrays & RAYS[king_sq as usize][square as usize];
            visible &= pinray;
        }

        ////////////////////////////////////////////////////////////////////////
        //
        // Captures
        //
        ////////////////////////////////////////////////////////////////////////

        let mut captures = visible & theirs;

        if in_check {
            captures &= checkers;
        }

        for target in captures {
            moves.push(Move::new(square, target, Capture));
        }

        if !QUIETS {
            return;
        }

        ////////////////////////////////////////////////////////////////////////
        //
        // Quiets
        //
        ////////////////////////////////////////////////////////////////////////
        
        let mut quiets = visible & !blockers;

        // If we're in check, blocking is the only valid option
        if in_check {
            let checker_sq = checkers.first();
            quiets &= BETWEEN[checker_sq as usize][king_sq as usize];
        }

        for target in quiets {
            moves.push(Move::new(square, target, Quiet));
        }
    }

    /// If there's a valid EP move, add it to the moves buffer
    fn en_passant_move(
        &self, 
        square: Square, 
        moves: &mut Vec<Move>, 
        checkers: Bitboard
    ) {
        let us = self.current;
        let ep_sq = self.en_passant.unwrap();
        let in_check = checkers.count() > 0;
        let attacked_sq = ep_sq.backward(us).unwrap();

        // See if we can capture in the first place
        let can_capture = square.pawn_attacks(us).contains(ep_sq);

        if !can_capture {
            return;
        }

        // If we're in check, EP is only allowed if the pawn we're trying to 
        // capture happens to be the checker.
        if in_check && !checkers.contains(attacked_sq) {
            return;
        }

        // Make sure the capture doesn't lead to a discovered check.
        let cleared_rank = RANKS[square.rank()];
        let source = Bitboard::from(square);
        let captured = Bitboard::from(attacked_sq);
        let invisible = source | captured;
        let xray_checkers = self.xray_checkers(invisible);
        let exposes_check = !xray_checkers
            .overlap(cleared_rank)
            .is_empty();

        if exposes_check {
            return;
        }

        moves.push(Move::new(square, ep_sq, MoveType::EnPassant));
    }

    // Find a legal move corresponding to an un-annotated bare move, if any.
    pub fn find_move(&self, bare: BareMove) -> Option<Move> {
        let legals = self.legal_moves::<QUIETS>();
        legals.into_iter().find(|legal| legal.eq(&bare))
    }
}

////////////////////////////////////////////////////////////////////////////////
//
// Tests
//
////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;
    use crate::square::Square;
    use Square::*;
    

    #[test]
    fn double_pushes() {
        let board: Board = "rnbqkbnr/ppp1pppp/3p4/8/8/3P4/PPP1PPPP/RNBQKBNR w KQkq - 0 2"
            .parse()
            .unwrap();
        let legal_moves = board.legal_moves::<QUIETS>();

        // e2 can double-push
        assert!(legal_moves
            .iter()
            .find(|mv| mv.src() == Square::E2 && mv.tgt() == Square::E4 && mv.is_double_push())
            .is_some());

        // d3 can't double-push
        assert!(legal_moves
            .iter()
            .find(|mv| mv.src() == Square::D3 && mv.tgt() == Square::D5)
            .is_none());
    }

    #[test]
    fn pieces_must_block_to_counter_checks() {
        let board: Board = "1k6/8/8/5q2/8/8/4R3/1K6 w - - 0 1".parse().unwrap();
        let legal_moves = board.legal_moves::<QUIETS>();

        let rook_moves: Vec<Move> = legal_moves
            .into_iter()
            .filter(|mv| mv.src() == Square::E2)
            .collect();

        // Only two legal moves: block on c2 or e4
        assert_eq!(rook_moves.len(), 2);
        assert!(rook_moves
            .iter()
            .find(|mv| mv.tgt() == Square::C2)
            .is_some());
        assert!(rook_moves
            .iter()
            .find(|mv| mv.tgt() == Square::E4)
            .is_some())
    }

    #[test]
    fn king_must_move_out_of_check() {
        let board: Board = "1k6/8/8/5q2/8/3K4/8/8 w - - 0 1".parse().unwrap();
        let king_moves: Vec<Move> = board
            .legal_moves::<QUIETS>()
            .into_iter()
            .filter(|mv| mv.src() == Square::D3)
            .collect();

        // Only king moves are getting out of check
        assert_eq!(king_moves.len(), 6);
        assert!(king_moves
            .iter()
            .find(|mv| mv.tgt() == Square::E4)
            .is_none());
        assert!(king_moves
            .iter()
            .find(|mv| mv.tgt() == Square::C2)
            .is_none());
    }

    #[test]
    fn check_blocks_and_king_moves_combined() {
        let board: Board = "1k6/8/8/5q2/8/4P3/PP5r/RK6 w - - 0 1".parse().unwrap();
        let legal_moves = board.legal_moves::<QUIETS>();
        let king_moves: Vec<&Move> = legal_moves
            .iter()
            .filter(|mv| mv.src() == Square::B1)
            .collect();

        let pawn_moves: Vec<&Move> = legal_moves
            .iter()
            .filter(|mv| mv.src() == Square::E3)
            .collect();

        // Only legal moves are Kc1 and e4
        assert_eq!(legal_moves.len(), 2);

        // King's only move is c1
        assert_eq!(king_moves.len(), 1);
        assert_eq!(king_moves.first().unwrap().tgt(), Square::C1);

        // Pawn's only move is e4
        assert_eq!(pawn_moves.len(), 1);
        assert_eq!(pawn_moves.first().unwrap().tgt(), Square::E4);
    }

    #[test]
    fn pins() {
        let board: Board = "1k6/2q5/8/1n6/5B2/1R6/8/1K6 b - - 0 1".parse().unwrap();
        let legal_moves = board.legal_moves::<QUIETS>();

        let knight_moves: Vec<&Move> = legal_moves.iter().filter(|mv| mv.src() == B5).collect();

        let queen_moves: Vec<&Move> = legal_moves.iter().filter(|mv| mv.src() == C7).collect();

        // Knight is completely pinned
        assert_eq!(knight_moves.len(), 0);

        // Queen can move within the pin ray
        assert_eq!(queen_moves.len(), 3);
    }

    #[test]
    fn en_passant() {
        let board: Board = "1k6/8/8/8/3Pp3/8/8/1K6 b - d3 0 1".parse().unwrap();
        let legal_moves = board.legal_moves::<QUIETS>();

        let pawn_moves: Vec<&Move> = legal_moves.iter().filter(|mv| mv.src() == E4).collect();

        assert_eq!(pawn_moves.len(), 2, "there are two legal pawn moves from e4");

        let en_passant = pawn_moves.iter().find(|mv| mv.tgt() == D3);
        assert!(en_passant.is_some(), "We can capture en-passant");
        assert!(
            en_passant.unwrap().is_en_passant(),
            "The en-passant flag is set"
        );
    }

    #[test]
    fn en_passant_revealed_check() {
        let board: Board = "8/8/8/8/k2Pp2R/8/8/K7 b - d3 0 1".parse().unwrap();
        let legal_moves = board.legal_moves::<QUIETS>();

        let pawn_moves: Vec<&Move> = legal_moves.iter().filter(|mv| mv.src() == E4).collect();

        let en_passant = pawn_moves.iter().find(|mv| mv.tgt() == D3);
        assert!(
            en_passant.is_none(),
            "En-passant not allowed if it reveals a check"
        );
    }
}
