//! Find all the legal moves for a given board state
//!
//! Starts off with all the pseudo-legal moves, and whittles them down until
//! we end up with only legal moves. This takes into account things like
//! - Double pushes
//! - En-passant captures
//! - Castling
//! - Checks
//! - Pins

use crate::{
    bitboard::Bitboard,
    piece::Color,
    movegen::moves::MoveType,
};
use crate::movegen::attack_boards::Rank;
use crate::board::Board;
use crate::movegen::attack_boards::BETWEEN;
use crate::movegen::moves::Move;
use crate::piece::PieceType;

use super::moves::BareMove;

const NO_KING: bool = false;

impl Board {
    /// Find all the legal moves for the current board state
    pub fn legal_moves(&self) -> Vec<Move> {
        use PieceType::*;
        let us = self.current;
        let them = !us;
        let king_sq = self.get_bb(King, us).first();
        let ours = self.occupied_by(us);
        let theirs = self.occupied_by(them);
        let blockers = ours | theirs;
        let checkers = self.checkers();
        let in_check = checkers.count() > 0;
        let in_double_check = checkers.count() > 1;

        let pinrays = self.pinrays();
        let pinned_pieces = ours & pinrays.iter().collect();
        let king_safe_squares = !self.attacked_by::<NO_KING>(them);

        let mut legal_moves: Vec<Move> = Vec::with_capacity(50);

        for source in ours {
            let piece = self.get_at(source).unwrap();
            let is_pinned = pinned_pieces.contains(source);

            // When there's more than one piece giving check, there's no other
            // option but for the king to move out of check.
            if in_double_check && !piece.is_king() {
                continue;
            }

            let visible = match piece.piece_type() {
                Pawn => source.pawn_squares(us, blockers) 
                    | source.pawn_attacks(us) & theirs,
                Knight => source.knight_squares(),
                Bishop => source.bishop_squares(blockers),
                Rook => source.rook_squares(blockers),
                Queen => source.queen_squares(blockers),
                King => source.king_squares()
            };

            // Get the available target squares for this piece
            let mut targets = Bitboard::EMPTY;

            ///////////////////////////////////////////////////////////////////
            //
            // Captures
            //
            ///////////////////////////////////////////////////////////////////
            
            let mut captures = visible & theirs;

            // When in check, only captures of the checker are allowed
            if in_check && !piece.is_king() {
                captures &= checkers;
            }
            
            if piece.is_king() {
                captures &= king_safe_squares;
            }

            // Add potential en-passant moves, after making sure they don't lead
            // to discovered checks
            if self.en_passant.is_some() && piece.is_pawn() {
                let ep_sq = self.en_passant.unwrap();
                let ep_bb: Bitboard = ep_sq.into();

                // See if we can capture in the first place
                let can_capture = source.pawn_attacks(us).contains(ep_sq);
                let captured_sq = ep_sq.forward(them).unwrap();

                let cleared_rank = Rank::ALL[source.rank()];
                let source = Bitboard::from(source);
                let captured = Bitboard::from(captured_sq);
                let invisible = source | captured;
                let xray_checkers = self.xray_checkers(invisible);
                let exposes_check = !xray_checkers
                    .overlap(cleared_rank)
                    .is_empty();


                if can_capture && !exposes_check {
                    if !in_check || checkers.contains(captured_sq) {
                        captures |= ep_bb;
                    }
                }
            }

            // If we're pinned, we can't move outside of our pin-ray
            if is_pinned {
                let &pinray = pinrays
                    .iter()
                    .find(|ray| ray.contains(source))
                    .expect("A pinned piece should lie on a pinray");

                captures &= pinray;
            }

            targets |= captures;

            ///////////////////////////////////////////////////////////////////
            //
            // Quiets
            //
            ///////////////////////////////////////////////////////////////////

            let mut quiets = visible & !blockers;

            // The king can't move into an attacked square
            if piece.is_king() {
                quiets &= king_safe_squares;
            }

            // If we're in check, blocking is the only valid option
            if in_check && !piece.is_king() {
                let checker_sq = checkers.first();
                quiets &= BETWEEN[checker_sq as usize][king_sq as usize];
            }

            // If we're pinned, we can only move within our pin ray
            if is_pinned {
                let &pinray = pinrays
                    .iter()
                    .find(|ray| ray.contains(source))
                    .expect("A pinned piece should lie on a pinray");

                quiets &= pinray;
            }

            targets |= quiets;

            ///////////////////////////////////////////////////////////////////
            //
            // Convert to Moves
            //
            ///////////////////////////////////////////////////////////////////

            // Add remaining pseudolegal moves to legal moves
            for target in targets {
                // TODO, make these function calls so I don't evaluate all of these at once?
                // Or will the compiler inline these anyway? I imagine it will,
                // actually...
                let is_capture = Bitboard::from(target) & theirs != Bitboard::EMPTY;
                let is_en_passant = piece.is_pawn() && self.en_passant.is_some_and(|ep_sq| ep_sq == target);
                let is_double_push = piece.is_pawn() && source.distance(target) == 2;

                let is_promotion = piece.is_pawn() 
                    && match piece.color() {
                        Color::White => target.rank() == 7,
                        Color::Black => target.rank() == 0
                    };

                if is_promotion {
                    if is_capture {
                        legal_moves.push(Move::new(source, target, MoveType::KnightPromoCapture));
                        legal_moves.push(Move::new(source, target, MoveType::BishopPromoCapture));
                        legal_moves.push(Move::new(source, target, MoveType::RookPromoCapture));
                        legal_moves.push(Move::new(source, target, MoveType::QueenPromoCapture));
                    } else {
                        legal_moves.push(Move::new(source, target, MoveType::KnightPromo));
                        legal_moves.push(Move::new(source, target, MoveType::BishopPromo));
                        legal_moves.push(Move::new(source, target, MoveType::RookPromo));
                        legal_moves.push(Move::new(source, target, MoveType::QueenPromo));
                    }
                } else if is_capture {
                    // Flag (simple) captures
                    legal_moves.push(Move::new(source, target, MoveType::Capture));

                } else if is_en_passant  {
                    // Check EP
                    legal_moves.push(Move::new(source, target, MoveType::EnPassant));

                } else if is_double_push {
                    // Flag pawn double pushes
                    legal_moves.push(Move::new(source, target, MoveType::DoublePush));
                } else {
                    legal_moves.push(Move::new(source, target, MoveType::Quiet));
                }
            }
        }

        // Add available castles at the end
        legal_moves.extend(
            self.castling_rights
                .get_available(self.current)
                .into_iter()
                .filter(|&ctype| self.castle_allowed(ctype))
                .map(|ctype| ctype.king_move()),
        );

        legal_moves
    }

    // Find a legal move corresponding to an un-annotated bare move, if any.
    pub fn find_move(&self, bare: BareMove) -> Option<Move> {
        let legals = self.legal_moves();
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
        let legal_moves = board.legal_moves();

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
        let legal_moves = board.legal_moves();

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
            .legal_moves()
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
        let legal_moves = board.legal_moves();
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
        let legal_moves = board.legal_moves();

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
        let legal_moves = board.legal_moves();

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
        let legal_moves = board.legal_moves();

        let pawn_moves: Vec<&Move> = legal_moves.iter().filter(|mv| mv.src() == E4).collect();

        let en_passant = pawn_moves.iter().find(|mv| mv.tgt() == D3);
        assert!(
            en_passant.is_none(),
            "En-passant not allowed if it reveals a check"
        );
    }
}
