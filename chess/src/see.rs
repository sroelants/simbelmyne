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

const SEE_VALUES: [i32; PieceType::COUNT] = [100, 300, 300, 500, 900, 0];

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

        if mv.is_capture() && !mv.is_en_passant() {
            let captured_piece = self.get_at(tgt).unwrap();
            balance += SEE_VALUES[captured_piece.piece_type() as usize];
        }

        let mut current_victim = if mv.is_promotion() {
            mv.get_promo_type().unwrap()
        } else {
            self.get_at(src).unwrap().piece_type()
        };

        // If we'd match the threshold even if we lost the capturing piece,
        // return early with a win.
        if balance - SEE_VALUES[current_victim as usize] >= threshold {
            return true;
        }

        // Since we're not going through the board for updates, we keep track
        // of the occupations ourselves.
        let mut blockers = self.all_occupied();
        blockers ^= Bitboard::from(src);
        blockers |= Bitboard::from(tgt);

        if mv.is_en_passant() {
            blockers ^= Bitboard::from(mv.get_capture_sq());
        }

        let diag_sliders = 
            (self.diag_sliders(White) | self.diag_sliders(Black)) & blockers;

        let hv_sliders = 
            (self.hv_sliders(White) | self.hv_sliders(Black)) & blockers;

        let mut attackers = self.attackers(tgt, blockers) & blockers;
        let mut side = self.current;

        // Start trading off pieces
        loop {
            // Express balance in terms of the current side to play
            side = !side;
            balance = -balance;

            // Find least valuable attacker
            let Some(attacker_sq) = self.lva(attackers, side) else { break };
            let attacker_bb = Bitboard::from(attacker_sq);
            let attacker = self.get_at(attacker_sq).unwrap();

            // Remove the attacker from the boards
            blockers ^= attacker_bb;
            attackers &= blockers;
            // diag_sliders &= blockers;
            // hv_sliders &= blockers;

            // Any discovered attackers?
            if attacker.is_pawn() || attacker.is_diag_slider() {
                attackers |= tgt.bishop_squares(blockers) & diag_sliders;
            }

            if attacker.is_hv_slider() {
                attackers |= tgt.rook_squares(blockers) & hv_sliders;
            }

            // If the attacker is a king, but the opponent still has attackers,
            // cut the exchange short.
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
}
