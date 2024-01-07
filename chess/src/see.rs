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

const SEE_VALUES: [i32; PieceType::COUNT] = [100, 300, 300, 500, 900, 10000];

impl Board {
    /// Check whether a move passes a given SEE threshold by trading off all the
    /// pieces attacking the target square.
    pub fn see(&self, mv: Move, threshold: Eval) -> bool {
        use PieceType::*;
        use Color::*;

        let src = mv.src();
        let tgt = mv.tgt();

        let mut balance = 0;

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

        // If the balance is already in the opponent's favor without them
        // even having to move, we've already lost
        if balance < 0 {
            return false;
        }

        // If we'd match the threshold even if we lost the capturing piece,
        // return early with a win.
        if balance - SEE_VALUES[current_victim as usize] >= threshold {
            return true;
        }

        // Since we're not going through the board for updates, we keep track
        // of the occupations ourselves.
        let mut remaining = self.all_occupied();
        remaining ^= Bitboard::from(src);
        remaining |= Bitboard::from(tgt);

        if mv.is_en_passant() {
            remaining ^= Bitboard::from(mv.get_capture_sq());
        }

        let diag_sliders = 
            (self.diag_sliders(White) | self.diag_sliders(Black)) & remaining;

        let hv_sliders = 
            (self.hv_sliders(White) | self.hv_sliders(Black)) & remaining;

        let mut attackers = self.attackers(tgt, remaining) & remaining;
        let mut side = self.current;

        // Start trading off pieces
        loop {
            // Express balance in terms of the current side to play
            side = !side;
            balance = -balance;

            // Find least valuable attacker, and break if no attackers are left
            let Some(attacker_sq) = self.lva(attackers, side) else { break };
            let attacker = self.get_at(attacker_sq).unwrap();

            // Remove the attacker from the boards
            remaining ^= Bitboard::from(attacker_sq);

            // Any discovered attackers?
            if attacker.is_pawn() || attacker.is_diag_slider() {
                attackers |= tgt.bishop_squares(remaining) & diag_sliders;
            }

            if attacker.is_hv_slider() {
                attackers |= tgt.rook_squares(remaining) & hv_sliders;
            }

            attackers &= remaining;

            // If the attacker is a king, but the opponent still has attackers,
            // cut the exchange short (because the capture isn't legal).
            if attacker.is_king() 
                && !(attackers & self.occupied_by(!side)).is_empty() {
                break;
            }

            // Update balance
            balance += SEE_VALUES[current_victim as usize];

            current_victim = attacker.piece_type();

            // Check again whether we can stand to lose this attacker and still
            // come out positive.
            if side == self.current {
                if balance - SEE_VALUES[current_victim as usize] >= threshold {
                    return true;
                }
            } else {
                if balance - SEE_VALUES[current_victim as usize] >= 0 {
                    return false;
                }
            }
        }

        // Make sure the balance is wrt the original player
        if side == !self.current {
            balance = -balance
        }

        // After all the exchanges are done, check whether the final balance
        // matches the threshold
        threshold <= balance
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
        const SEE_SUITE: [(&str, &str, Eval, bool); 13] = [
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
            ("5rk1/1pp2q1p/p1pb4/8/3P1NP1/2P5/1P1BQ1P1/5RK1 b - - 0 1", "d6f4", 0, false),
            ("5rk1/1pp2q1p/p1pb4/8/3P1NP1/2P5/1P1BQ1P1/5RK1 b - - 0 1", "d6f4", -100, true),
        ];

        for (fen, mv, threshold, result) in SEE_SUITE {
            let board: Board = fen.parse().unwrap();
            let mv = board.find_move(mv.parse().unwrap()).unwrap();

            println!("Move: {mv}\n{board}\n");
            assert_eq!(board.see(mv, threshold), result);
        }
    }
}
