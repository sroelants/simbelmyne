//! Static exchange evaluation
//!
//! The goal of static exchange evaluation is to give a static estimate 
//! (static meaning: without doing an actual search) whether or not a move leads
//! to material gain or losses. We do this by playing out all possible captures
//! as a result of the provided move, starting with the least valuable attacker.
//!
//! This allows us to do more granular move ordering (by pushing "bad captures"
//! to the back of the list) and more effective pruning (by pruning bad 
//! captures as well as quiets).

use crate::bitboard::Bitboard;
use crate::board::Board;
use crate::movegen::moves::Move;
use crate::piece::PieceType;
use crate::piece::Color;
use crate::square::Square;

type Eval = i32;

pub const SEE_VALUES: [i32; PieceType::COUNT] = [100, 300, 300, 500, 900, 10000];

impl Board {
    /// Check whether a move passes a given SEE threshold by trading off all the
    /// pieces attacking the target square.
    pub fn see(&self, mv: Move, threshold: Eval) -> bool {
        use PieceType::*;
        use Color::*;

        let src = mv.src();
        let tgt = mv.tgt();

        let mut balance = -threshold;

        // Castling moves are always zero SEE, since they by definition can't be
        // captured.
        if mv.is_castle() {
            return threshold <= balance; 
        }

        if mv.is_promotion() {
            balance -= SEE_VALUES[Pawn as usize];
            balance += SEE_VALUES[mv.get_promo_type().unwrap() as usize];
        }

        if mv.is_capture() {
            let captured_piece = self.get_at(mv.get_capture_sq()).unwrap();
            balance += SEE_VALUES[captured_piece.piece_type() as usize];
        }

        let mut current_victim = if mv.is_promotion() {
            mv.get_promo_type().unwrap()
        } else {
            self.get_at(src).unwrap().piece_type()
        };

        // Since we're not going through the board for updates, we keep track
        // of the occupations ourselves.
        let mut remaining = self.all_occupied();
        remaining ^= Bitboard::from(src);
        remaining |= Bitboard::from(tgt);

        if mv.is_en_passant() {
            remaining ^= Bitboard::from(mv.get_capture_sq());
        }

        let mut diag_sliders = 
            (self.diag_sliders(White) | self.diag_sliders(Black)) & remaining;

        let mut hv_sliders = 
            (self.hv_sliders(White) | self.hv_sliders(Black)) & remaining;

        let mut attackers = self.attackers(tgt, remaining) & remaining;
        let mut side = self.current;

        // Start trading off pieces
        loop {
            // Express balance in terms of the current side to play
            side = !side;

            // Check whether or not we need to re-capture in the first place
            if side == self.current && balance >= 0 
                || side != self.current && balance <= 0 {
                break;
            }

            // Find least valuable attacker, and break if no attackers are left
            let Some(attacker_sq) = self.lva(attackers, side) else { break };
            let attacker = self.get_at(attacker_sq).unwrap();

            // If our last attacker is a king, but the opponent still has 
            // attackers, cut the exchange short (because the capture isn't 
            // legal).
            if attacker.is_king() 
                && !(attackers & self.occupied_by(!side)).is_empty() {
                break;
            }

            // Remove the attacker from the boards
            remaining    ^= Bitboard::from(attacker_sq);
            attackers    &= remaining;
            diag_sliders &= remaining;
            hv_sliders   &= remaining;

            // Any discovered attackers?
            if attacker.is_pawn() || attacker.is_diag_slider() {
                attackers |= tgt.bishop_squares(remaining) & diag_sliders;
            }

            if attacker.is_hv_slider() {
                attackers |= tgt.rook_squares(remaining) & hv_sliders;
            }

            // Update balance
            if side == self.current {
                balance += SEE_VALUES[current_victim as usize];
            } else {
                balance -= SEE_VALUES[current_victim as usize];
            }

            current_victim = attacker.piece_type();
        }

        // After all the exchanges are done, check whether the final balance
        // matches the threshold
        balance >= 0
    }

    /// Find the least valuable piece for a given side in a bitboard of 
    /// attackers.
    fn lva(&self, attackers: Bitboard, side: Color) -> Option<Square> {
        let mut lva = None;
        let mut lowest_score = Eval::MAX;

        for attacker_sq in attackers & self.occupied_by(side) {
            let attacker = self.get_at(attacker_sq).unwrap();
            let score = SEE_VALUES[attacker.piece_type() as usize];

            if  score < lowest_score {
                lva = Some(attacker_sq);
                lowest_score = score;
            }
        }

        lva
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

    #[test]
    fn test_lva() {
        use Square::*;
        use Color::*;
        // kiwipete
        let board: Board = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1".parse().unwrap();

        let attackers = board.attackers(D5, board.all_occupied());
        assert_eq!(board.lva(attackers, White), Some(E4));
        assert_eq!(board.lva(attackers, Black), Some(E6));
    }

    #[test]
    fn test_see() {
        // kiwipete
        let board: Board = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1".parse().unwrap();

        // Queen to h5 is negative SEE for white
        let mv = board.find_move("f3h5".parse().unwrap()).unwrap();
        assert!(!board.see(mv, 0), "Queen to H5 is negative SEE for white");

        // Bishop to b5 is zero SEE
        let mv = board.find_move("e2b5".parse().unwrap()).unwrap();
        assert!(board.see(mv, 0), "Bishop to B5 is zero SEE for white");
    } 

    // Test suite stolen from Carp
    #[test]
    fn test_see_carp() {
        #[rustfmt::skip]
        let suite: Vec<(&str, &str, Eval, bool)> = vec![
            ("1k1r4/1pp4p/p7/4p3/8/P5P1/1PP4P/2K1R3 w - - 0 1", "e1e5", 0, true),
            ("1k1r3q/1ppn3p/p4b2/4p3/8/P2N2P1/1PP1R1BP/2K1Q3 w - - 0 1", "d3e5", 0, false),
            ("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1", "g2h3", 0, true),
            ("k3r3/8/8/4p3/8/2B5/1B6/K7 w - - 0 1", "c3e5", 0, true),
            ("4kbnr/p1P4p/b1q5/5pP1/4n3/5Q2/PP1PPP1P/RNB1KBNR w KQk f6 0 1", "g5f6", 0, true),
            ("6k1/1pp4p/p1pb4/6q1/3P1pRr/2P4P/PP1Br1P1/5RKN w - - 0 1", "f1f4", 0, false),
            ("6RR/4bP2/8/8/5r2/3K4/5p2/4k3 w - - 0 1", "f7f8q", 0, true),
            ("r1bqk1nr/pppp1ppp/2n5/1B2p3/1b2P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 0 1", "e1g1", 0, true),
            ("4kbnr/p1P1pppp/b7/4q3/7n/8/PPQPPPPP/RNB1KBNR w KQk - 0 1", "c7c8q", 0, true),
            ("4kbnr/p1P1pppp/b7/4q3/7n/8/PP1PPPPP/RNBQKBNR w KQk - 0 1", "c7c8q", 0, false),
            ("3r3k/3r4/2n1n3/8/3p4/2PR4/1B1Q4/3R3K w - - 0 1", "d3d4", 0, false),
            ("5rk1/1pp2q1p/p1pb4/8/3P1NP1/2P5/1P1BQ1P1/5RK1 b - - 0 1", "d6f4", 0, true),
            ("5rk1/1pp2q1p/p1pb4/8/3P1NP1/2P5/1P1BQ1P1/5RK1 b - - 0 1", "d6f4", -100, true),
        ];

        for (fen, mv, threshold, result) in suite {
            let board: Board = fen.parse().unwrap();
            let mv = board.find_move(mv.parse().unwrap()).unwrap();

            println!("Move: {mv}\n{board}\n");
            assert_eq!(board.see(mv, threshold), result);
        }
    }

    // Test suite stolen from Starzix
    #[test]
    fn test_see_starzix() {
        #[rustfmt::skip]
        let suite: Vec<(&str, &str, Eval, bool)> = vec![
            ("6k1/1pp4p/p1pb4/6q1/3P1pRr/2P4P/PP1Br1P1/5RKN w - - 0 1", "f1f4", -100, true),
            ("5rk1/1pp2q1p/p1pb4/8/3P1NP1/2P5/1P1BQ1P1/5RK1 b - - 0 1", "d6f4", 0, true),
            ("4R3/2r3p1/5bk1/1p1r3p/p2PR1P1/P1BK1P2/1P6/8 b - - 0 1", "h5g4", 0, true),
            ("4R3/2r3p1/5bk1/1p1r1p1p/p2PR1P1/P1BK1P2/1P6/8 b - - 0 1", "h5g4", 0, true),
            ("4r1k1/5pp1/nbp4p/1p2p2q/1P2P1b1/1BP2N1P/1B2QPPK/3R4 b - - 0 1", "g4f3", 0, true),
            ("2r1r1k1/pp1bppbp/3p1np1/q3P3/2P2P2/1P2B3/P1N1B1PP/2RQ1RK1 b - - 0 1", "d6e5", 100, true),
            ("7r/5qpk/p1Qp1b1p/3r3n/BB3p2/5p2/P1P2P2/4RK1R w - - 0 1", "e1e8", 0, true),
            ("6rr/6pk/p1Qp1b1p/2n5/1B3p2/5p2/P1P2P2/4RK1R w - - 0 1", "e1e8", -500, true),
            ("7r/5qpk/2Qp1b1p/1N1r3n/BB3p2/5p2/P1P2P2/4RK1R w - - 0 1", "e1e8", -500,  true),
            ("6RR/4bP2/8/8/5r2/3K4/5p2/4k3 w - - 0 1", "f7f8q", 200, true),
            ("6RR/4bP2/8/8/5r2/3K4/5p2/4k3 w - - 0 1", "f7f8n", 200, true),
            ("7R/5P2/8/8/6r1/3K4/5p2/4k3 w - - 0 1", "f7f8q", 800, true),
            ("7R/5P2/8/8/6r1/3K4/5p2/4k3 w - - 0 1", "f7f8b", 200, true),
            ("7R/4bP2/8/8/1q6/3K4/5p2/4k3 w - - 0 1", "f7f8r", -100, true),
            ("8/4kp2/2npp3/1Nn5/1p2PQP1/7q/1PP1B3/4KR1r b - - 0 1", "h1f1", 0, true),
            ("8/4kp2/2npp3/1Nn5/1p2P1P1/7q/1PP1B3/4KR1r b - - 0 1", "h1f1", 0, true),
            ("2r2r1k/6bp/p7/2q2p1Q/3PpP2/1B6/P5PP/2RR3K b - - 0 1", "c5c1", 100, true),
            ("r2qk1nr/pp2ppbp/2b3p1/2p1p3/8/2N2N2/PPPP1PPP/R1BQR1K1 w kq - 0 1", "f3e5", 100, true),
            ("6r1/4kq2/b2p1p2/p1pPb3/p1P2B1Q/2P4P/2B1R1P1/6K1 w - - 0 1", "f4e5", 0, true),
            ("3q2nk/pb1r1p2/np6/3P2Pp/2p1P3/2R4B/PQ3P1P/3R2K1 w - h6 0 1", "g5h6", 0, true),
            ("3q2nk/pb1r1p2/np6/3P2Pp/2p1P3/2R1B2B/PQ3P1P/3R2K1 w - h6 0 1", "g5h6", 100, true),
            ("2r4r/1P4pk/p2p1b1p/7n/BB3p2/2R2p2/P1P2P2/4RK2 w - - 0 1", "c3c8", 500, true),
            ("2r5/1P4pk/p2p1b1p/5b1n/BB3p2/2R2p2/P1P2P2/4RK2 w - - 0 1", "c3c8", 500, true),
            ("2r4k/2r4p/p7/2b2p1b/4pP2/1BR5/P1R3PP/2Q4K w - - 0 1", "c3c5", 300, true),
            ("8/pp6/2pkp3/4bp2/2R3b1/2P5/PP4B1/1K6 w - - 0 1", "g2c6", -200, true),
            ("4q3/1p1pr1k1/1B2rp2/6p1/p3PP2/P3R1P1/1P2R1K1/4Q3 b - - 0 1", "e6e4", -400, true),
            ("4q3/1p1pr1kb/1B2rp2/6p1/p3PP2/P3R1P1/1P2R1K1/4Q3 b - - 0 1", "h7e4", 100, true),
            ("3r3k/3r4/2n1n3/8/3p4/2PR4/1B1Q4/3R3K w - - 0 1", "d3d4", -100, true),
            ("1k1r4/1ppn3p/p4b2/4n3/8/P2N2P1/1PP1R1BP/2K1Q3 w - - 0 1", "d3e5", 100, true),
            ("1k1r3q/1ppn3p/p4b2/4p3/8/P2N2P1/1PP1R1BP/2K1Q3 w - - 0 1", "d3e5", -200, true),
            ("rnb2b1r/ppp2kpp/5n2/4P3/q2P3B/5R2/PPP2PPP/RN1QKB2 w Q - 0 1", "h4f6", 100, true),
            ("r2q1rk1/2p1bppp/p2p1n2/1p2P3/4P1b1/1nP1BN2/PP3PPP/RN1QR1K1 b - - 0 1", "g4f3", 0, true),
            ("r1bqkb1r/2pp1ppp/p1n5/1p2p3/3Pn3/1B3N2/PPP2PPP/RNBQ1RK1 b kq - 0 1", "c6d4", 0, true),
            ("r1bq1r2/pp1ppkbp/4N1p1/n3P1B1/8/2N5/PPP2PPP/R2QK2R w KQ - 0 1", "e6g7", 0, true),
            ("r1bq1r2/pp1ppkbp/4N1pB/n3P3/8/2N5/PPP2PPP/R2QK2R w KQ - 0 1", "e6g7", 300, true),
            ("rnq1k2r/1b3ppp/p2bpn2/1p1p4/3N4/1BN1P3/PPP2PPP/R1BQR1K1 b kq - 0 1", "d6h2", -200, true),
            ("rn2k2r/1bq2ppp/p2bpn2/1p1p4/3N4/1BN1P3/PPP2PPP/R1BQR1K1 b kq - 0 1", "d6h2", 100, true),
            ("r2qkbn1/ppp1pp1p/3p1rp1/3Pn3/4P1b1/2N2N2/PPP2PPP/R1BQKB1R b KQq - 0 1", "g4f3", 100, true),
            ("rnbq1rk1/pppp1ppp/4pn2/8/1bPP4/P1N5/1PQ1PPPP/R1B1KBNR b KQ - 0 1", "b4c3", 0, true),
            ("r4rk1/3nppbp/bq1p1np1/2pP4/8/2N2NPP/PP2PPB1/R1BQR1K1 b - - 0 1", "b6b2", -800, true),
            ("r4rk1/1q1nppbp/b2p1np1/2pP4/8/2N2NPP/PP2PPB1/R1BQR1K1 b - - 0 1", "f6d5", -200, true),
            ("1r3r2/5p2/4p2p/2k1n1P1/2PN1nP1/1P3P2/8/2KR1B1R b - - 0 1", "b8b3", -400, true),
            ("1r3r2/5p2/4p2p/4n1P1/kPPN1nP1/5P2/8/2KR1B1R b - - 0 1", "b8b4", 100, true),
            ("2r2rk1/5pp1/pp5p/q2p4/P3n3/1Q3NP1/1P2PP1P/2RR2K1 b - - 0 1", "c8c1", 0, true),
            ("5rk1/5pp1/2r4p/5b2/2R5/6Q1/R1P1qPP1/5NK1 b - - 0 1", "f5c2", -100, true),
            ("1r3r1k/p4pp1/2p1p2p/qpQP3P/2P5/3R4/PP3PP1/1K1R4 b - - 0 1", "a5a2", -800, true),
            ("1r5k/p4pp1/2p1p2p/qpQP3P/2P2P2/1P1R4/P4rP1/1K1R4 b - - 0 1", "a5a2", 100, true),
            ("r2q1rk1/1b2bppp/p2p1n2/1ppNp3/3nP3/P2P1N1P/BPP2PP1/R1BQR1K1 w - - 0 1", "d5e7", 0, true),
            ("rnbqrbn1/pp3ppp/3p4/2p2k2/4p3/3B1K2/PPP2PPP/RNB1Q1NR w - - 0 1", "d3e4", 100, true),
            ("rnb1k2r/p3p1pp/1p3p1b/7n/1N2N3/3P1PB1/PPP1P1PP/R2QKB1R w KQkq - 0 1", "e4d6", -200, true),
            ("r1b1k2r/p4npp/1pp2p1b/7n/1N2N3/3P1PB1/PPP1P1PP/R2QKB1R w KQkq - 0 1", "e4d6", 0, true),
            ("2r1k2r/pb4pp/5p1b/2KB3n/4N3/2NP1PB1/PPP1P1PP/R2Q3R w k - 0 1", "d5c6", -300, true),
            ("2r1k2r/pb4pp/5p1b/2KB3n/1N2N3/3P1PB1/PPP1P1PP/R2Q3R w k - 0 1", "d5c6", 0, true),
            ("2r1k3/pbr3pp/5p1b/2KB3n/1N2N3/3P1PB1/PPP1P1PP/R2Q3R w - - 0 1", "d5c6", -300, true),
            ("5k2/p2P2pp/8/1pb5/1Nn1P1n1/6Q1/PPP4P/R3K1NR w KQ - 0 1", "d7d8q", 800, true),
            ("r4k2/p2P2pp/8/1pb5/1Nn1P1n1/6Q1/PPP4P/R3K1NR w KQ - 0 1", "d7d8q", -100, true),
            ("5k2/p2P2pp/1b6/1p6/1Nn1P1n1/8/PPP4P/R2QK1NR w KQ - 0 1", "d7d8q", 200, true),
            ("4kbnr/p1P1pppp/b7/4q3/7n/8/PP1PPPPP/RNBQKBNR w KQk - 0 1", "c7c8q", -100, true),
            ("4kbnr/p1P1pppp/b7/4q3/7n/8/PPQPPPPP/RNB1KBNR w KQk - 0 1", "c7c8q", 200, true),
            ("4kbnr/p1P1pppp/b7/4q3/7n/8/PPQPPPPP/RNB1KBNR w KQk - 0 1", "c7c8q", 200, true),
            ("4kbnr/p1P4p/b1q5/5pP1/4n3/5Q2/PP1PPP1P/RNB1KBNR w KQk f6 0 1", "g5f6", 0, true),
            ("4kbnr/p1P4p/b1q5/5pP1/4n3/5Q2/PP1PPP1P/RNB1KBNR w KQk f6 0 1", "g5f6", 0, true),
            ("4kbnr/p1P4p/b1q5/5pP1/4n2Q/8/PP1PPP1P/RNB1KBNR w KQk f6 0 1", "g5f6", 0, true),
            ("1n2kb1r/p1P4p/2qb4/5pP1/4n2Q/8/PP1PPP1P/RNB1KBNR w KQk - 0 1", "c7b8q", 200, true),
            ("rnbqk2r/pp3ppp/2p1pn2/3p4/3P4/N1P1BN2/PPB1PPPb/R2Q1RK1 w kq - 0 1", "g1h2", 300, true),
            ("3N4/2K5/2n5/1k6/8/8/8/8 b - - 0 1", "c6d8", 0, true),
            ("3n3r/2P5/8/1k6/8/8/3Q4/4K3 w - - 0 1", "c7d8q", 700, true),
            ("r2n3r/2P1P3/4N3/1k6/8/8/8/4K3 w - - 0 1", "e6d8", 300, true),
            ("8/8/8/1k6/6b1/4N3/2p3K1/3n4 w - - 0 1", "e3d1", 0, true),
            ("8/8/1k6/8/8/2N1N3/4p1K1/3n4 w - - 0 1", "c3d1", 100, true),
            ("r1bqk1nr/pppp1ppp/2n5/1B2p3/1b2P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 0 1", "e1g1", 0, true)
        ];

        for (fen, mv, threshold, result) in suite {
            let board: Board = fen.parse().unwrap();
            let mv = board.find_move(mv.parse().unwrap()).unwrap();

            println!("Move: {mv}\n{board}\n");
            assert_eq!(board.see(mv, threshold), result);
        }
    }
}
