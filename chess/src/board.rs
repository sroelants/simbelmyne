//! The board is the main data structure this libarry revolves around.
//! 
//! It holds the complete state for a game at one instant in time. (This means
//! it doesn't keep track of history-related things, such as repetitions and the
//! like)

use crate::constants::{LIGHT_SQUARES, DARK_SQUARES};
use crate::square::Square;
use crate::bitboard::Bitboard;
use crate::movegen::attack_boards::{PAWN_ATTACKS, Direction};
use crate::movegen::castling::CastlingRights;
use crate::piece::{PieceType, Piece, Color};
use crate::util::fen::{FENAtom, FEN};
use anyhow::anyhow;
use itertools::Itertools;
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
}

impl Board {
    pub fn new() -> Board {
        Board::from_str("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap()
    }

    /// Get the occupation bitboard for a given side.
    pub fn occupied_by(&self, side: Color) -> Bitboard {
        self.occupied_squares[side as usize]
    }

    /// Get the total occupation of the board
    pub fn all_occupied(&self) -> Bitboard {
        self.occupied_squares.into_iter().collect()
    }
    /// Get the bitboard for given piece type and side
    pub fn get_bb(&self, ptype: PieceType, color: Color) -> Bitboard {
        self.piece_bbs[ptype as usize] & self.occupied_by(color)
    }

    /// Return the piece on a given square, if any
    pub fn get_at(&self, square: Square) -> Option<Piece> {
        self.piece_list[square as usize]
    }

    /// Add a piece on a given square.
    /// Panics if there is already a piece on the square!
    pub fn add_at(&mut self, square: Square, piece: Piece) {
        // Add to piece list
        self.piece_list[square as usize] = Some(piece);

        // Update bitboards
        let bb: Bitboard = square.into();
        self.occupied_squares[piece.color() as usize] |= bb;
        self.piece_bbs[piece.piece_type() as usize] |= bb;
    }

    /// Remove a piece on a given square
    /// Panics if there is no piece on the square
    pub fn remove_at(&mut self, square: Square) -> Option<Piece> {
        let piece = self.piece_list[square as usize]?;
        self.piece_list[square as usize] = None;

        let bb: Bitboard = square.into();
        self.occupied_squares[piece.color() as usize] &= !bb;
        self.piece_bbs[piece.piece_type() as usize] &= !bb;

        Some(piece)
    }
}

////////////////////////////////////////////////////////////////////////////////
//
// Move generation logic
//
////////////////////////////////////////////////////////////////////////////////

impl Board {
    /// Compute the map of all squares attacked by the requested side, 
    ///
    /// Takes a `KING` flag to indicate whether or not the opponent's king 
    /// should be included in the blockers. This is important if we want to 
    /// know what squares are safe for the king to move to. This is because the 
    /// king itself could be blocking some attacked squares, leading it to 
    /// believe they are safe to move to.
    pub fn attacked_by<const KING: bool>(&self, us: Color) -> Bitboard {
        use PieceType::*;

        let mut attacked = Bitboard(0);
        let them = !us;
        let ours = self.occupied_by(us);
        let mut theirs = self.occupied_by(them);

        if !KING {
            theirs &= !self.piece_bbs[King as usize];
        }

        let pawns = ours & self.piece_bbs[Pawn as usize];
        let rooks = ours & self.piece_bbs[Rook as usize];
        let knights = ours & self.piece_bbs[Knight as usize];
        let bishops = ours & self.piece_bbs[Bishop as usize];
        let queens = ours & self.piece_bbs[Queen as usize];
        let kings = ours & self.piece_bbs[King as usize];

        let blockers = ours | theirs;

        for square in pawns {
            attacked |= pawn_attacks(square, us);
        }

        for square in knights {
            attacked |= square.knight_squares();
        }

        for square in bishops {
            attacked |= square.bishop_squares(blockers);
        }

        for square in rooks {
            attacked |= square.rook_squares(blockers);
        }

        for square in queens {
            attacked |= square.queen_squares(blockers);
        }

        for square in kings {
            attacked |= square.king_squares();
        }

        attacked
    }

    /// Compute a bitboard of all the pieces putting the current player's king 
    /// in check.
    ///
    /// Defer to the more general `Board::xray_checkers` that allows one to mask
    /// out a subset of the blockers before computing the checkers.
    pub fn checkers(&self) -> Bitboard {
        self.xray_checkers(Bitboard::EMPTY)
    }

    /// Return the bitboard of pieces checking the current player's king if a 
    /// subset of blockers were removed.
    pub fn xray_checkers(&self, invisible: Bitboard) -> Bitboard {
        use PieceType::*;

        let us = self.current;
        let them = !us;
        let ours = self.occupied_by(us) & !invisible;
        let theirs = self.occupied_by(them) & !invisible;
        let blockers = ours | theirs;
        let our_king: Square = self.get_bb(King, us).first();

        let pawns = self.piece_bbs[Pawn as usize];
        let rooks = self.piece_bbs[Rook as usize];
        let knights = self.piece_bbs[Knight as usize];
        let bishops = self.piece_bbs[Bishop as usize];
        let queens = self.piece_bbs[Queen as usize];

        let checkers = (pawns & our_king.pawn_attacks(us))
                | (rooks & our_king.rook_squares(blockers))
                | (knights & our_king.knight_squares())
                | (bishops & our_king.bishop_squares(blockers))
                | (queens & our_king.queen_squares(blockers));

        theirs & checkers
    }

    /// Compute the pin rays that are pinning the current player's pieces.
    pub fn compute_pinrays(&self) -> Vec<Bitboard> {
        // Idea: 
        // See how many of the opponent's sliders are checking our king if all
        // our pieces weren't there. Then check whether those rays contain a 
        // single piece. If so, it's pinned. (Note that it would be, by 
        // necessity, one of our pieces, since otherwise the king couldn't have 
        // been in check)
        use PieceType::*;
        let us = self.current;
        let them = !us;
        let king_sq = self.get_bb(King, us).first();

        let ours = self.occupied_by(us);
        let theirs = self.occupied_by(them);
        let diag_sliders = self.get_bb(Bishop, them) | self.get_bb(Queen, them);
        let hv_sliders = self.get_bb(Rook, them) | self.get_bb(Queen, them);

        let mut pinrays: Vec<Bitboard> = Vec::new();

        for dir in Direction::DIAG {
            let visible_ray = king_sq.visible_ray(dir, theirs);
            let has_diag_slider = !visible_ray.overlap(diag_sliders).is_empty();
            let has_single_piece = (visible_ray & ours).count() == 1;
            if has_diag_slider && has_single_piece {
                pinrays.push(visible_ray);
            }
        }

        for dir in Direction::HV {
            let visible_ray = king_sq.visible_ray(dir, theirs);
            let has_hv_slider = !visible_ray.overlap(hv_sliders).is_empty();
            let has_single_piece = (visible_ray & ours).count() == 1;
            if has_hv_slider && has_single_piece {
                pinrays.push(visible_ray);
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
        !self.checkers().is_empty()
    }

    /// Check whether the current player is in checkmate
    /// NOTE: This is fairly intensive, avoid using in hot loops
    pub fn checkmate(&self) -> bool {
        self.in_check() && self.legal_moves().len() == 0 
    }

    /// Check for rule_based draws
    ///
    /// For now, this includes 50-move rule and insufficient material.
    /// Does not include stalemate, since we don't want to have to recompute all
    /// the legal moves whenever we do this check
    pub fn is_rule_draw(&self) -> bool {
        let is_fifty_moves = self.half_moves >= 100;
        let is_insufficient_material = self.insufficient_material();

        is_fifty_moves || is_insufficient_material
    }

    /// Check for draws
    /// NOTE: This is fairly intensive, avoid using in hot loops
    pub fn is_draw(&self) -> bool {
        let is_stalemate = self.legal_moves().len() == 0 && !self.in_check();

        is_stalemate || self.is_rule_draw()
    }

    /// Check whether the board has insufficient material for either player to
    /// mate.
    ///
    // Since we're looking for efficiency here, we're better off identifying
    // positions where mate can't be _forced_, even if theoretically possible. 
    // The quicker we find a (likely) draw, the better (i.e., just like 
    // checkmate, we should break off the search and return a draw asap.
    pub fn insufficient_material(&self) -> bool {
        let occupied = self.all_occupied();
        let knights = self.piece_bbs[PieceType::Knight as usize];
        let bishops = self.piece_bbs[PieceType::Bishop as usize];
        let same_color_bishops = (bishops & LIGHT_SQUARES).count() > 0
            || (bishops & DARK_SQUARES).count() > 0;

        // Two kings is insufficient
        if occupied.count() == 2 {
            return true;
        }

        // King + B/N vs King is insufficient
        if occupied.count() == 3 && (knights | bishops).count() > 0 {
            return true;
        }

        // Same colored bishops is insufficient
        if occupied.count() == 4 && same_color_bishops {
            return true;
        }

        false
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

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut lines: Vec<String> = vec![];
        lines.push("  a b c d e f g h ".to_string());

        for (rank, squares) in Square::RANKS.into_iter().enumerate() {
            let mut line: Vec<String> = vec![];

            line.push((rank + 1).to_string());
            line.push(" ".to_string());

            for sq in squares {
                let square = match self.get_at(sq) {
                    Some(piece) => format!("{}", piece),
                    None => ".".to_string(),
                };

                line.push(square);
                line.push(" ".to_string());
            }
            line.push((rank + 1).to_string());
            let line = line.join("");

            lines.push(line);
        }
        lines.push("  a b c d e f g h ".to_owned());

        write!(f, "{}", lines.join("\n"))
    }
}

////////////////////////////////////////////////////////////////////////////////
//
// FEN utilities
//
////////////////////////////////////////////////////////////////////////////////

impl Board {
    pub fn to_fen(&self) -> String {
        let ranks = self.piece_list.into_iter().chunks(8);
        let ranks = ranks.into_iter().collect_vec();
        let mut rank_strs: Vec<String> = Vec::new();

        for rank in ranks.into_iter().rev() {
            let mut elements: Vec<String> = Vec::new();
            let piece_runs = rank.into_iter().group_by(|p| p.is_some());

            for run in &piece_runs {
                match run {
                    (true, pieces) => {
                        for piece in pieces {
                            elements.push(piece.unwrap().to_string())
                        }
                    }
                    (false, gaps) => elements.push(gaps.count().to_string()),
                }
            }

            rank_strs.push(elements.join(""));
        }

        let pieces = rank_strs.into_iter().join("/");
        let next_player = self.current.to_string();
        let castling = self.castling_rights.to_string();
        let en_passant = self
            .en_passant
            .map(|sq| sq.to_string())
            .unwrap_or(String::from("-"));
        let half_moves = self.half_moves;
        let full_moves = self.full_moves;

        format!("{pieces} {next_player} {castling} {en_passant} {half_moves} {full_moves}")
    }

    pub fn from_fen(fen: &str) -> anyhow::Result<Board> {
        let mut parts = fen.split(' ');

        let piece_string = parts.next().ok_or(anyhow!("Invalid FEN string"))?;

        let fen = FEN::try_from(piece_string)?;

        let mut piece_bbs = [Bitboard::EMPTY; PieceType::COUNT];
        let mut occupied_squares = [Bitboard::EMPTY; Color::COUNT];
        let mut piece_list = [None; Square::COUNT];

        // FEN starts with the 8th rank down, so we need to reverse the ranks
        // to go in ascending order
        for (rank, atoms) in fen.ranks.into_iter().enumerate() {
            let mut file: usize = 0;
            for atom in atoms {
                match atom {
                    FENAtom::Gap(n) => {
                        file += n;
                    }

                    FENAtom::Piece(color, piece_type) => {
                        let sq = Square::RANKS[rank][file];
                        let bb = Bitboard::from(sq);
                        let piece = Piece::new(piece_type, color);

                        piece_list[sq as usize] = Some(piece);

                        piece_bbs[piece_type as usize] |= bb;
                        occupied_squares[color as usize] |= bb;

                        file += 1;
                    }
                }
            }
        }

        let current: Color = parts.next().ok_or(anyhow!("Invalid FEN string"))?.parse()?;

        let castling_rights: CastlingRights =
            parts.next().ok_or(anyhow!("Invalid FEN string"))?.parse()?;

        let en_passant: Option<Square> = parts
            .next()
            .ok_or(anyhow!("Invalid FEN string"))?
            .parse()
            .ok();

        let half_moves = parts.next().ok_or(anyhow!("Invalid FEN string"))?.parse()?;

        let full_moves = parts.next().ok_or(anyhow!("Invalid FEN string"))?.parse()?;

        Ok(Board {
            piece_list,
            piece_bbs,
            occupied_squares,
            current,
            castling_rights,
            en_passant,
            half_moves,
            full_moves,
        })
    }
}

////////////////////////////////////////////////////////////////////////////////
//
// Tests
//
////////////////////////////////////////////////////////////////////////////////

#[test]
fn test_to_fen() {
    let initial_fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let board = Board::from_str(initial_fen).unwrap();
    let fen = board.to_fen();
    assert_eq!(initial_fen, fen);
}

/// Return the squares that are attacked by a pawn of a given color, placed on
/// a given square. This only regards whether the square is _under attack_, not
/// whether there is an actual piece there that the pawn might capture on this 
/// turn
pub fn pawn_attacks(square: Square, side: Color) -> Bitboard {
    PAWN_ATTACKS[side as usize][square as usize]
}
