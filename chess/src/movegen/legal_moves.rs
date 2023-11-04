use super::moves::Move;
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
    board::{Board, PieceType, Square},
};

impl Board {
    /// Find all the legal moves for the current board state
    pub fn legal_moves(&self) -> Vec<Move> {
        use PieceType::*;
        let player = self.current;
        let opp = player.opp();
        let king_bb = self.get_bb(King, player);
        let our_pieces = self.occupied_by(player);
        let their_pieces = self.occupied_by(opp);
        let blockers = our_pieces | their_pieces;
        let checkers = self.compute_checkers(opp);
        let in_check = !checkers.is_empty();
        let in_double_check = in_check && checkers.count_ones() > 1;

        // TODO: Optimization: pinned pieces is easier to compute: compute that
        // first, and only if there's pinned pieces in the first place, should
        // we compute the actual pinrays
        let pinrays = self.compute_pinrays(player);
        let pinned_pieces = our_pieces & pinrays.iter().collect();

        let mut legal_moves: Vec<Move> = Vec::new();

        for source in our_pieces {
            let piece = self.get_at(source).expect("Source should hold a piece");

            // When there's more than one piece giving check, there's no other
            // option but for the king to move out of check.
            if in_double_check && !piece.is_king() {
                continue;
            }

            // Get the pseudo-legal moves for the piece
            let mut pseudos: Bitboard = piece
                .visible_squares(our_pieces, their_pieces)
                .remove(our_pieces);

            // The king can't move into an attacked square
            if piece.is_king() {
                pseudos &= !self.king_danger_squares(player)
            }

            // If we're in check, capturing or blocking is the only valid option
            if in_check && !piece.is_king() {
                let checker = self.piece_list[Square::from(checkers) as usize]
                    .expect("There is a checking piece on this square");

                // Mask of squares we're allowed to move to when in check
                let mut check_mask = checker.position.into();

                // If the checker is a slider, there might be a check-ray that 
                // we can block, so add it to the check-mask.
                if checker.is_slider() {
                    let check_ray = checker
                        .visible_rays(blockers)
                        .into_iter()
                        .find(|ray| ray.contains(king_bb))
                        .expect("The checking piece is a slider, so there must be a pin-ray");

                    check_mask |= check_ray;
                }

                pseudos &= check_mask;
            }

            // If we're pinned, we can only move within our pin ray
            if pinned_pieces.contains(piece.position) {
                let pinray = pinrays
                    .iter()
                    .find(|ray| ray.contains(piece.position))
                    .expect("A pinned piece should lie on a pinray");

                pseudos &= *pinray;
            }

            // Add remaining pseudolegal moves to legal moves
            for target in pseudos {
                let mut mv = Move::new(source, target);

                // Flag pawn double pushes
                if piece.is_pawn() && Square::is_double_push(source, target) {
                    mv.set_double_push()
                }

                legal_moves.push(mv);
            }

            // Add potential en-passant moves, after making sure they don't lead
            // to discovered checks
            if let Some(en_passant) = self.en_passant {
                let ep_bb = en_passant.into();
                if piece.piece_type != Pawn {
                    continue;
                }

                let can_capture = piece.visible_squares(our_pieces, ep_bb).contains(ep_bb);

                // If we can't capture en-passant in the first place, bail
                if !can_capture {
                    continue;
                }

                let captured_bb: Bitboard = en_passant
                    .forward(opp)
                    .expect("Double-push pawn can't be out-of-bounds")
                    .into();

                // If the EP would reveal a discovered check, bail.
                if self.is_xray_check(player, piece.position | captured_bb) {
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
            self.castling_rights
                .get_available(self.current)
                .into_iter()
                .filter(|ctype| ctype.is_allowed(self))
                .map(|ctype| ctype.king_move()),
        );

        legal_moves
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

        assert_eq!(pawn_moves.len(), 2);

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
