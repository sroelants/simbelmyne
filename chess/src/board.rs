//! The board is the main data structure this libarry revolves around.
//! 
//! It holds the complete state for a game at one instant in time. (This means
//! it doesn't keep track of history-related things, such as repetitions and the
//! like)

use crate::constants::{DARK_SQUARES, LIGHT_SQUARES, RANKS};
use crate::square::Square;
use crate::bitboard::Bitboard;
use crate::movegen::lookups::BETWEEN;
use crate::movegen::castling::CastlingRights;
use crate::piece::{PieceType, Piece, Color};
use colored::Colorize;
use std::fmt::Display;
use std::str::FromStr;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Board {
    /// The color of the current player
    pub current: Color,

    /// Squares occupied by a given piece type
    pub piece_bbs: [Bitboard; PieceType::COUNT],

    /// Squares occupied _by_ a given side
    pub occupied_squares: [Bitboard; Color::COUNT],

    /// List of pieces, indexable by a Square, more efficient for lookups than `pieces`
    pub piece_list: [Option<Piece>; Square::COUNT],

    /// Keeps track of what types of castling are still allowed
    pub castling_rights: CastlingRights,

    /// The last half-turn's en-passant square, if there was a double push
    pub en_passant: Option<Square>,

    /// The number of plys since the last capture or pawn advance
    /// Useful for enforcing the 50-move draw rule
    pub half_moves: u8,

    /// The number of full turns
    /// Starts at one, and is incremented after every Black move
    pub full_moves: u16,

    // Mask of all pinrays, indexed by the color whose pieces are pinned
    pub pinrays: [Bitboard; Color::COUNT],

    // Mask of all checkers
    pub checkers: Bitboard,

    /// Mask of all squares attacked by the opponent
    pub threats: Bitboard,
}

impl Board {
    pub fn new(
        piece_list: [Option<Piece>; 64],
        piece_bbs: [Bitboard; 6],
        occupied_squares: [Bitboard; 2],
        current: Color,
        castling_rights: CastlingRights,
        en_passant: Option<Square>,
        half_moves: u8,
        full_moves: u16,
    ) -> Self {
        let mut board = Self {
            piece_list,
            piece_bbs,
            occupied_squares,
            current,
            castling_rights,
            en_passant,
            half_moves,
            full_moves,
            pinrays: [Bitboard::EMPTY; 2],
            checkers: Bitboard::EMPTY,
            threats: Bitboard::EMPTY,
        };

        board.pinrays = [
            board.compute_pinrays(Color::White),
            board.compute_pinrays(Color::Black),
        ];

        board.checkers = board.compute_checkers(current);

        board.threats = board.king_threats();

        board
    }

    /// Get the occupation bitboard for a given side.
    pub fn occupied_by(&self, side: Color) -> Bitboard {
        self.occupied_squares[side]
    }

    /// Get the total occupation of the board
    pub fn all_occupied(&self) -> Bitboard {
        self.occupied_squares.into_iter().collect()
    }
    /// Get the bitboard for given piece type and side
    pub fn get_bb(&self, ptype: PieceType, color: Color) -> Bitboard {
        self.piece_bbs[ptype] & self.occupied_by(color)
    }

    /// Return the piece on a given square, if any
    pub fn get_at(&self, square: Square) -> Option<Piece> {
        self.piece_list[square]
    }

    /// Add a piece on a given square.
    /// Panics if there is already a piece on the square!
    pub fn add_at(&mut self, square: Square, piece: Piece) {
        // Add to piece list
        self.piece_list[square] = Some(piece);

        // Update bitboards
        let bb: Bitboard = square.into();
        self.occupied_squares[piece.color()] |= bb;
        self.piece_bbs[piece.piece_type()] |= bb;
    }

    /// Remove a piece on a given square
    /// Panics if there is no piece on the square
    pub fn remove_at(&mut self, square: Square) -> Option<Piece> {
        let piece = self.piece_list[square]?;
        self.piece_list[square] = None;

        let bb: Bitboard = square.into();
        self.occupied_squares[piece.color()] &= !bb;
        self.piece_bbs[piece.piece_type()] &= !bb;

        Some(piece)
    }

    pub fn pawns(&self, side: Color) -> Bitboard {
        self.piece_bbs[PieceType::Pawn] & self.occupied_by(side)
    }

    pub fn knights(&self, side: Color) -> Bitboard {
        self.piece_bbs[PieceType::Knight] & self.occupied_by(side)
    }

    pub fn bishops(&self, side: Color) -> Bitboard {
        self.piece_bbs[PieceType::Bishop] & self.occupied_by(side)
    }

    pub fn rooks(&self, side: Color) -> Bitboard {
        self.piece_bbs[PieceType::Rook] & self.occupied_by(side)
    }

    pub fn queens(&self, side: Color) -> Bitboard {
        self.piece_bbs[PieceType::Queen] & self.occupied_by(side)
    }

    pub fn kings(&self, side: Color) -> Bitboard {
        self.piece_bbs[PieceType::King] & self.occupied_by(side)
    }

    pub fn diag_sliders(&self, side: Color) -> Bitboard {
        self.bishops(side) | self.queens(side)
    }

    pub fn hv_sliders(&self, side: Color) -> Bitboard {
        self.rooks(side) | self.queens(side)
    }

    pub fn pieces(&self, side: Color) -> Bitboard {
        self.knights(side) 
        | self.bishops(side)
        | self.rooks(side)
        | self.queens(side)
    }

    pub fn get_pinrays(&self, us: Color) -> Bitboard {
        self.pinrays[us]
    }

    pub fn get_checkers(&self) -> Bitboard {
        self.checkers
    }

    pub fn get_threats(&self) -> Bitboard {
        self.threats
    }

    pub fn get_promo_rank(&self) -> Bitboard {
        if self.current.is_white() {
            RANKS[7]
        } else {
            RANKS[0]
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
//
// Move generation logic
//
////////////////////////////////////////////////////////////////////////////////

impl Board {
    /// Calculate a map of king threats
    ///
    /// King threats are all the squares that are unsafe for the king to move to.
    /// These are basically all the squares that are under attack if the king
    /// were taken off the board. (Since the king might be obstructing some
    /// attacks, but it would still be unsafe for the king to move there).
    pub fn king_threats(&self) -> Bitboard {
        let mut attacked = Bitboard(0);
        let us = self.current;
        let them = !us;
        let ours = self.occupied_by(us) & !self.kings(us);
        let theirs = self.occupied_by(them);
        let blockers = ours | theirs;

        attacked |= self.pawn_attacks(them);

        for square in self.knights(them) {
            attacked |= square.knight_squares();
        }

        for square in self.bishops(them) {
            attacked |= square.bishop_squares(blockers);
        }

        for square in self.rooks(them) {
            attacked |= square.rook_squares(blockers);
        }

        for square in self.queens(them) {
            attacked |= square.queen_squares(blockers);
        }

        for square in self.kings(them) {
            attacked |= square.king_squares();
        }

        attacked
    }

    /// Return a bitboard of all squares attacked by pawns of the requested
    /// color
    pub fn pawn_attacks(&self, us: Color) -> Bitboard {
        let pawns = self.pawns(us);

        if us.is_white() {
            pawns.forward::<true>().left() | pawns.forward::<true>().right()
        } else {
            pawns.forward::<false>().left() | pawns.forward::<false>().right()
        }
    }

    /// Compute a bitboard of all the pieces putting the current player's king 
    /// in check.
    ///
    /// Defer to the more general `Board::xray_checkers` that allows one to mask
    /// out a subset of the blockers before computing the checkers.
    pub fn compute_checkers(&self, us: Color) -> Bitboard {
        self.xray_checkers(us, Bitboard::EMPTY)
    }

    /// Return the bitboard of pieces checking the current player's king if a 
    /// subset of blockers were removed.
    pub fn xray_checkers(&self, us: Color, invisible: Bitboard) -> Bitboard {
        let them = !us;
        let ours_visible = self.occupied_by(us) & !invisible;
        let theirs_visible = self.occupied_by(them) & !invisible;
        let blockers = ours_visible | theirs_visible;
        let our_king = self.kings(us).first();

        let checkers = 
            (self.pawns(them)     & our_king.pawn_attacks(us) & theirs_visible)
            | (self.knights(them) & our_king.knight_squares())
            | (self.bishops(them) & our_king.bishop_squares(blockers))
            | (self.rooks(them)   & our_king.rook_squares(blockers))
            | (self.queens(them)  & our_king.queen_squares(blockers));

        checkers
    }

    /// Find all attackers, black or white, attacking a given square.
    pub fn attackers(&self, square: Square, blockers: Bitboard) -> Bitboard {
        use PieceType::*;
        use Color::*;

        let attackers = 
              square.pawn_attacks(Black)      & self.pawns(White)
            | square.pawn_attacks(White)      & self.pawns(Black)
            | square.knight_squares()         & self.piece_bbs[Knight]
            | square.bishop_squares(blockers) & self.piece_bbs[Bishop]
            | square.rook_squares(blockers)   & self.piece_bbs[Rook]
            | square.queen_squares(blockers)  & self.piece_bbs[Queen];

        attackers
    }

    /// Compute the pin rays that are pinning the current player's pieces.
    pub fn compute_pinrays(&self, us: Color) -> Bitboard {
        // Idea: 
        // See how many of the opponent's sliders are checking our king if all
        // our pieces weren't there. Then check whether those rays contain a 
        // single piece. If so, it's pinned. (Note that it would be, by 
        // necessity, one of our pieces, since otherwise the king couldn't have 
        // been in check)
        let them = !us;
        let king_sq = self.kings(us).first();

        let ours = self.occupied_by(us);
        let theirs = self.occupied_by(them);
        let diag_sliders = self.diag_sliders(them);
        let hv_sliders = self.hv_sliders(them);

        let mut pinrays = Bitboard::EMPTY;

        let potential_pinners = king_sq.rook_squares(theirs) & hv_sliders
            | king_sq.bishop_squares(theirs) & diag_sliders;

        for pinner in potential_pinners {
            let mut ray = BETWEEN[pinner][king_sq];
            ray |= Bitboard::from(pinner);

            if (ray & ours).count() == 1 {
                pinrays |= ray;
            }
        }

        pinrays
    }
}

////////////////////////////////////////////////////////////////////////////////
//
// Game state: Checks, Mates, Draws
//
////////////////////////////////////////////////////////////////////////////////

impl Board {
    /// Check whether the current player is in check
    pub fn in_check(&self) -> bool {
        !self.get_checkers().is_empty()
    }

    /// Check for rule_based draws
    ///
    /// For now, this includes 50-move rule and insufficient material.
    /// Does not include stalemate, since we don't want to have to recompute all
    /// the legal moves whenever we do this check
    pub fn is_rule_draw(&self) -> bool {
        self.half_moves >= 100 || self.insufficient_material()
    }

    /// Check whether the board has insufficient material for either player to
    /// mate.
    pub fn insufficient_material(&self) -> bool {
        use PieceType::*;

        // As long as there's pawns on the board, there's hope
        let pawns = self.piece_bbs[Pawn];

        if !pawns.is_empty() {
            return false;
        }

        let occupied = self.all_occupied();
        let num_pieces = self.all_occupied().count();
        let kings = self.piece_bbs[King];

        // More than 4 pieces is sufficient
        if num_pieces > 4 {
            return false;
        }

        // Two kings is insufficient
        if num_pieces == 2 {
            return true;
        }

        // King + B/N vs King is insufficient
        let knights = self.piece_bbs[Knight];
        let bishops = self.piece_bbs[Bishop];
        let minors = knights | bishops;

        if occupied == kings | minors && minors.count() == 1 {
            return true
        }

        // Same colored bishops is insufficient
        if occupied == kings | (bishops & LIGHT_SQUARES)
            || occupied == kings | (bishops & DARK_SQUARES) {
            return true;
        }

        false
    }

    /// Check whether Zugzwang is unlikely
    ///
    /// Very rudimentary check: if we have pieces on the board besides pawns 
    /// and a king, it's probably not zugzwang.
    pub fn zugzwang_unlikely(&self) -> bool {
        let us = self.current;
        let ours = self.occupied_by(us);
        let pawns = self.pawns(us);
        let king = self.kings(us);

        ours != pawns | king
    }
}

////////////////////////////////////////////////////////////////////////////////
//
// Utility traits
//
////////////////////////////////////////////////////////////////////////////////

impl FromStr for Board {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> anyhow::Result<Self> {
        Board::from_fen(value)
    }
}

fn blank_line(rank: usize) -> String {
        let mut line: Vec<String> = Vec::new();
        line.push("  ║".to_string());
    if rank % 2 == 0 {
        line.push("     ".on_white().to_string());
        line.push("     ".on_black().to_string());
        line.push("     ".on_white().to_string());
        line.push("     ".on_black().to_string());
        line.push("     ".on_white().to_string());
        line.push("     ".on_black().to_string());
        line.push("     ".on_white().to_string());
        line.push("     ".on_black().to_string());
    } else {
        line.push("     ".on_black().to_string());
        line.push("     ".on_white().to_string());
        line.push("     ".on_black().to_string());
        line.push("     ".on_white().to_string());
        line.push("     ".on_black().to_string());
        line.push("     ".on_white().to_string());
        line.push("     ".on_black().to_string());
        line.push("     ".on_white().to_string());
    }

    line.push("║ ".to_string());
    line.join("")
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut lines: Vec<String> = vec![];
        lines.push("     a    b    c    d    e    f    g    h   ".to_string());
        lines.push("  ╔════════════════════════════════════════╗".to_string());

        for (rank, squares) in Square::RANKS.into_iter().enumerate() {
            lines.push(blank_line(rank));

            let mut line: Vec<String> = vec![];
            line.push((8 - rank).to_string());
            line.push(" ║".to_string());
            for (file, sq) in squares.into_iter().enumerate() {
                let square = if (rank + file) % 2 == 0 {
                    match self.get_at(sq) {
                        Some(piece) => format!("  {}  ", piece).black().on_white(),
                        None => "     ".to_string().on_white(),
                    }
                } else {
                    match self.get_at(sq) {
                        Some(piece) => format!("  {}  ", piece).white().on_black(),
                        None => "     ".to_string().on_black(),
                    }
                };

                line.push(square.to_string());
            }
            line.push("║ ".to_string());
            line.push((8 - rank).to_string());
            let line = line.join("");
            lines.push(line);

            lines.push(blank_line(rank));
        }
        lines.push("  ╚════════════════════════════════════════╝".to_string());
        lines.push("     a    b    c    d    e    f    g    h   ".to_string());

        write!(f, "{}", lines.join("\n"))
    }
}

impl Board {
    /// Return a mirrored version of the board, with all pieces flipped sides 
    /// and color.
    pub fn mirror(&self) -> Self {
        let mut piece_bbs = [Bitboard::EMPTY; PieceType::COUNT];
        let mut occupied_squares = [Bitboard::EMPTY; Color::COUNT];
        let mut piece_list = [None; Square::COUNT];

        // Flip all the pieces and their colors
        for (idx, &piece) in self.piece_list.iter().enumerate() {
            if let Some(piece) = piece {
                let square = Square::from(idx).flip();
                let bb = Bitboard::from(Square::from(square));
                let mirrored = piece.mirror();

                piece_list[square] = Some(mirrored);
                piece_bbs[mirrored.piece_type()] |= bb;
                occupied_squares[mirrored.color()] |= bb;
            }
        }

        let castling_rights = self.castling_rights.mirror();
        let en_passant = self.en_passant.map(|ep| ep.flip());
        let current = self.current.opp();

        let mirrored = Self::new(
            piece_list,
            piece_bbs,
            occupied_squares,
            current,
            castling_rights,
            en_passant,
            self.half_moves,
            self.full_moves,
        );

        mirrored
    }
}


////////////////////////////////////////////////////////////////////////////////
//
// Game phase
//
////////////////////////////////////////////////////////////////////////////////

impl Board {
    /// Values assignd to each piece type to calculate the approximate stage 
    /// of the game
    const GAME_PHASE_VALUES: [u8; PieceType::COUNT] = [0, 1, 1, 2, 4, 0];

    /// Return a number from 0 to 24, roughly indicating the game's progress 
    /// based on remaining material. 24 means midgame, 0 means endgame.
    pub fn phase(&self) -> u8 {
        self.piece_list
            .iter()
            .flatten()
            .map(|&piece| Self::GAME_PHASE_VALUES[piece.piece_type()])
            .sum()
    }
}

impl Default for Board {
    fn default() -> Self {
        Board::from_str("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap()
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
    fn test_attackers() {
        use Square::*;
        // kiwipete
        let board: Board = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1".parse().unwrap();

        let attackers = board.attackers(G4, board.all_occupied());

        let expected = [F3, E5, F6]
            .into_iter()
            .map(|sq| Bitboard::from(sq))
            .collect();

        assert_eq!(attackers, expected);

        let attackers = board.attackers(D5, board.all_occupied());

        let expected = [C3, E4, B6, E6, F6]
            .into_iter()
            .map(|sq| Bitboard::from(sq))
            .collect();

        assert_eq!(attackers, expected);
    }
}
