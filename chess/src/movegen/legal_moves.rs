/// Find all the legal moves for a given board state
///
/// Starts off with all the pseudo-legal moves, and whittles them down until
/// we end up with only legal moves. This takes into account things like
/// - Double pushes
/// - En-passant captures
/// - Castling
/// - Checks
/// - Pins
use crate::{
    bitboard::Bitboard,
    piece::Color,
    square::Square,
    movegen::moves::MoveType,
};
use crate::movegen::attack_boards::Rank;
use crate::board::pawn_attacks;
use crate::board::Board;
use crate::movegen::attack_boards::BETWEEN;
use crate::movegen::moves::Move;
use crate::piece::PieceType;

use super::moves::BareMove;

impl Board {
    /// Find all the legal moves for the current board state
    pub fn legal_moves(&self) -> Vec<Move> {
        use PieceType::*;
        let player = self.current;
        let opp = player.opp();
        let king_bb = self.get_bb(King, player);
        let king_sq: Square = king_bb.first();
        let our_pieces = self.occupied_by(player);
        let their_pieces = self.occupied_by(opp);
        let checkers = self.compute_checkers();
        let in_check = !checkers.is_empty();
        let in_double_check = in_check && checkers.count_ones() > 1;

        // TODO: Optimization: pinned pieces is easier to compute: compute that
        // first, and only if there's pinned pieces in the first place, should
        // we compute the actual pinrays
        let pinrays = self.compute_pinrays(player);
        let pinned_pieces = our_pieces & pinrays.iter().collect();

        let mut legal_moves: Vec<Move> = Vec::with_capacity(50);

        for source in our_pieces {
            let piece = self.get_at(source).expect("Source should hold a piece");

            // When there's more than one piece giving check, there's no other
            // option but for the king to move out of check.
            if in_double_check && !piece.is_king() {
                continue;
            }

            // Get the pseudo-legal moves for the piece
            let mut pseudos: Bitboard = piece
                .visible_squares(source, our_pieces, their_pieces)
                .without(our_pieces);

            // The king can't move into an attacked square
            if piece.is_king() {
                pseudos &= !self.king_danger_squares(player)
            }

            // Add potential en-passant moves, after making sure they don't lead
            // to discovered checks
            if let (Some(ep_sq), true) = (self.en_passant, piece.is_pawn()) {
                let ep_bb: Bitboard = ep_sq.into();

                // See if we can capture in the first place
                let can_capture = pawn_attacks(source, piece.color())
                    .contains(ep_sq);

                // Look for any checkers on the rank that would get cleared by 
                // the ep-capture. If there's a discovered check, the EP is 
                // illegal and we bail
                let captured_bb: Bitboard = ep_sq
                    .forward(opp)
                    .expect("En-passant endangered pawn can't be out-of-bounds")
                    .into();

                let source_bb = Bitboard::from(source);
                let xray_checkers = self.compute_xray_checkers(player, source_bb | captured_bb);
                let cleared_rank = Rank::ALL[source.rank()];
                let exposes_check = (xray_checkers & cleared_rank) != Bitboard::EMPTY;

                // If we passed all the checks, add the EP square to our 
                // legal moves
                if can_capture && !exposes_check {
                    pseudos |= ep_bb;
                }
            }

            // If we're in check, capturing or blocking is the only valid option
            if in_check && !piece.is_king() {
                let checker_sq = checkers.first();
                let checker = self.piece_list[checker_sq as usize]
                    .expect("There is a checking piece on this square");

                // Mask of squares we're allowed to move to when in check
                let mut check_mask = checkers;

                // If 
                // 1. the checker is a pawn, 
                // 2. Is subject to EP
                // 3. The current piece is a pawn
                // then add the EP-square to the check mask
                if self.en_passant.is_some() && piece.is_pawn() {
                    let ep_sq = self.en_passant.unwrap();
                    // The square that might get captured by EP
                    let ep_attacked_square: Bitboard = ep_sq
                        .backward(piece.color())
                        .unwrap()
                        .into(); 

                    let is_ep_capturable = ep_attacked_square
                        & self.get_bb(Pawn, opp)
                        & check_mask != Bitboard::EMPTY;

                    if is_ep_capturable {
                        check_mask |= ep_sq.into()
                    }
                }

                // If the checker is a slider, there is a check-ray that we 
                // might be able to block, so add it to the check-mask.
                if checker.is_slider() {
                    check_mask |= BETWEEN[checker_sq as usize][king_sq as usize];
                }

                pseudos &= check_mask;
            }

            // If we're pinned, we can only move within our pin ray
            if pinned_pieces.contains(source) {
                let pinray = pinrays
                    .iter()
                    .find(|ray| ray.contains(source))
                    .expect("A pinned piece should lie on a pinray");

                pseudos &= *pinray;
            }

            // Add remaining pseudolegal moves to legal moves
            for target in pseudos {
                // TODO, make these function calls so I don't evaluate all of these at once?
                // Or will the compiler inline these anyway? I imagine it will,
                // actually...
                let is_capture = Bitboard::from(target) & their_pieces != Bitboard::EMPTY;
                let is_en_passant = piece.is_pawn() && self.en_passant.is_some_and(|ep_sq| ep_sq == target);
                let is_double_push = piece.is_pawn() && source.distance(target) == 2;

                let is_promotion = piece.is_pawn() && match piece.color() {
                    Color::White => Rank::W_PROMO_RANK.contains(target.into()),
                    Color::Black => Rank::B_PROMO_RANK.contains(target.into())
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

    pub fn find_move(&self, bare: BareMove) -> Option<Move> {
        let legals = self.legal_moves();
        legals.into_iter().find(|legal| legal.eq(&bare))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
