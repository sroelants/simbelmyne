use crate::board::Board;
use crate::constants::{FILES, RANKS};
use crate::movegen::castling::CastleType;
use crate::movegen::legal_moves::All;
use crate::movegen::moves::Move;
use crate::piece::PieceType;
use std::fmt::Write;

pub trait ToSan {
    fn to_san(self, board: &Board) -> String;
}

impl ToSan for Move {
    /// Render a move in Short Algebraic Notation
    fn to_san(self, board: &Board) -> String {
        use PieceType::*;
        let us = board.current;
        let piece_type = board.get_at(self.src()).unwrap().piece_type();
        let blockers = board.all_occupied();

        // Check modifier
        let check_str = CheckState::new(&board.play_move(self)).to_san(board);

        // If the move is a castling move, we simply return the appropriate
        // string.
        if self.is_castle() {
            let castle_type = CastleType::from_move(self).unwrap();
            let castle_str = castle_type.to_san(board);

            return format!("{castle_str}{check_str}");
        } 

        // Piece name
        let piece_str = board.get_at(self.src())
            .expect("Not a legal move: {self}")
            .piece_type()
            .to_san(board);

        // Target square
        let target_str = self.tgt().to_string();

        // Capture marker
        let capture_str = match board.get_at(self.get_capture_sq()) {
            Some(_) => "x",
            None => ""
        };

        // Disambiguation

        let sources = match piece_type {
            Pawn => self.tgt().pawn_squares(!us, blockers),
            Knight => self.tgt().knight_squares(),
            Bishop => self.tgt().bishop_squares(blockers),
            Rook => self.tgt().rook_squares(blockers),
            Queen => self.tgt().queen_squares(blockers),
            King => self.tgt().king_squares(),
        };

        let piece_bb = board.get_bb(piece_type, us);
        let ambiguous = (sources & piece_bb).count() > 1;

        let disambiguation_str = if piece_type == Pawn && self.is_capture() {
            let sq_str = self.src().to_string();
                sq_str[..1].to_string()

        } else if ambiguous {
            let sq_str = self.src().to_string();
            let file = FILES[self.src().file()];
            let rank = RANKS[self.src().rank()];
            let ambiguous_file = (file & piece_bb).count() > 1;
            let ambiguous_rank = (rank & piece_bb).count() > 1;

            if ambiguous_file && ambiguous_rank {
                sq_str
            } else if ambiguous_file {
                sq_str[1..].to_string()
            } else {
                sq_str[..1].to_string()
            }
        } else {
            "".to_string()
        };

        // Promotion flag
        let promo_str = if let Some(promo) = self.get_promo_type() {
            format!("={}", promo.to_san(board))
        } else {
            format!("")
        };

        format!("{piece_str}{disambiguation_str}{capture_str}{target_str}{promo_str}{check_str}")
    }
}

impl ToSan for CastleType {
    fn to_san(self, _board: &Board) -> String {
        match self {
            Self::WK | Self::BK => "O-O",
            Self::WQ | Self::BQ => "O-O-O",
        }.to_string()
    }
}

impl ToSan for PieceType {
    fn to_san(self, _board: &Board) -> String {
        match self {
            Self::Pawn   => "",
            Self::Knight => "N",
            Self::Bishop => "B",
            Self::Rook   => "R",
            Self::Queen  => "Q",
            Self::King   => "K"
        }.to_string()
    }
}

impl PieceType {
    pub fn from_san(s: &str) -> Self {
        match s {
            ""  => Self::Pawn,
            "N" => Self::Knight,
            "B" => Self::Bishop,
            "R" => Self::Rook,
            "Q" => Self::Queen,
            "K" => Self::King,
            _ => panic!("Not valid SAN piece type: {s}"),
        }
    }
}

#[derive(Copy, Clone)]
enum CheckState {
    Check,
    Checkmate,
    None
}

impl CheckState {
    pub fn new(board: &Board) -> Self {
        if !board.in_check() {
            return Self::None;
        } else if board.legal_moves::<All>().len() == 0 {
            return Self::Checkmate;
        } else {
            return Self::Check;
        }
    }
}
impl ToSan for CheckState {
    fn to_san(self, _board: &Board) -> String {
        match self {
            Self::Check => "+",
            Self::Checkmate => "#",
            Self::None => ""
        }.to_string()
    }
}

#[cfg(test)]
mod tests {
    use colored::Colorize;
    use std::str::FromStr;
    use crate::movegen::moves::BareMove;
    use super::*;

    const SAN_SUITE: [&str; 9] = [
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1; e2e4; e4",
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1; e2a6; Bxa6",
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R b KQkq - 0 1; f6d5; Nfxd5",
        "1k6/8/8/8/8/5Q1Q/8/K6Q w - - 0 1; h3f1; Qh3f1",
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1; e1c1; O-O-O",
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1; e1g1; O-O",
        "r3k2r/p1ppqpb1/b3pnp1/1N1PN3/1pn1P3/5Q1p/PPPBBPPP/R3K2R w KQkq - 2 2; b5d6; Nd6+",
        "r3k2r/p1ppqpb1/b3pnp1/1N1PN3/1pn1P3/5Q1p/PPPBBPPP/R3K2R w KQkq - 2 2; b5c7; Nxc7+",
        "1k6/4Q3/8/8/8/8/8/K6R w - - 0 1; h1h8; Rh8#"
    ];

    #[test]
    fn test_san() {
        for pos in SAN_SUITE {
            let mut parts = pos.split(";");
            let fen = parts.next().unwrap().trim();
            let uci = parts.next().unwrap().trim();
            let expected = parts.next().unwrap().trim();
            let board: Board = fen.parse().unwrap();
            let bare_move = BareMove::from_str(uci).expect("Invalid move: {uci}");
            let mv = Move::from_bare(bare_move, &board).expect("Invalid move: {uci}");
            let san = mv.to_san(&board);

            if san != expected {
                panic!("Expected {}, found {}", expected.blue(), san.red());
            }
        }
    }
}

impl<T: IntoIterator<Item = Move>> ToSan for T {
    fn to_san(self, board: &Board) -> String {
        let mut board = board.clone();
        let mut san = String::new();

        for mv in self {
            let _ = write!(san, "{} ", mv.to_san(&board));
            board = board.play_move(mv);
        }

        san.trim_end().to_string()
    }
}
