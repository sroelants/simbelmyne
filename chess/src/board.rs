use crate::constants::{LIGHT_SQUARES, DARK_SQUARES};
use crate::square::Square;
use crate::bitboard::Bitboard;
use crate::movegen::attack_boards::{W_PAWN_ATTACKS, B_PAWN_ATTACKS, Direction};
use crate::movegen::castling::CastlingRights;
use crate::movegen::moves::{visible_squares, visible_ray};
use crate::piece::{PieceType, Piece, Color};
use crate::util::fen::{FENAtom, FEN};
use anyhow::anyhow;
use itertools::Itertools;
use std::fmt::Display;
use std::str::FromStr;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Board {
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

#[allow(dead_code)]
impl Board {
    pub const EMPTY: Self = Self {
        current: Color::White,
        piece_bbs: [Bitboard::EMPTY; PieceType::COUNT],
        occupied_squares: [Bitboard::EMPTY; Color::COUNT],
        piece_list: [None; Square::COUNT],
        castling_rights: CastlingRights::ALL,
        en_passant: None,
        half_moves: 0,
        full_moves: 0,
    };

    pub fn new() -> Board {
        Board::from_str("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap()
    }

    pub fn get_at(&self, square: Square) -> Option<&Piece> {
        self.piece_list.get(square as usize)?.as_ref()
    }

    pub fn add_at(&mut self, square: Square, piece: Piece) {
        let bb: Bitboard = square.into();
        self.piece_list[square as usize] = Some(piece);

        self.occupied_squares[piece.color() as usize] |= bb;
        self.piece_bbs[piece.piece_type() as usize] |= bb;
    }

    pub fn remove_at(&mut self, square: Square) -> Option<Piece> {
        let bb: Bitboard = square.into();
        let piece = self.piece_list[square as usize]?;

        self.piece_list[square as usize] = None;

        self.occupied_squares[piece.color() as usize] ^= bb;
        self.piece_bbs[piece.piece_type() as usize] ^= bb;

        Some(piece)
    }

    /// Compute the squares this side's king cannot move to
    ///
    /// Subtly different from the `attacked_by` squares, since the king itself
    /// could be blocking some attacked squares
    pub fn king_danger_squares(&self, side: Color) -> Bitboard {
        use PieceType::*;
        let mut attacked = Bitboard(0);

        let ours = self.occupied_by(side);
        let theirs = self.occupied_by(side.opp());
        let opp = side.opp();

        let ours_without_king = ours.remove(self.get_bb(PieceType::King, side));

        let pawns = theirs & self.piece_bbs[Pawn as usize];
        let rooks = theirs & self.piece_bbs[Rook as usize];
        let knights = theirs & self.piece_bbs[Knight as usize];
        let bishops = theirs & self.piece_bbs[Bishop as usize];
        let queens = theirs & self.piece_bbs[Queen as usize];
        let kings = theirs & self.piece_bbs[King as usize];

        for pawn in pawns {
            let square = Square::from(pawn);
            attacked |= pawn_attacks(square, opp);
        }

        for knight in knights {
            let square = Square::from(knight);
            attacked |= visible_squares(square, Knight, opp, theirs, ours);
        }

        for bishop in bishops {
            let square = Square::from(bishop);
            attacked |= visible_squares(square, Bishop, opp, theirs, ours_without_king);
        }

        for rook in rooks {
            let square = Square::from(rook);
            attacked |= visible_squares(square, Rook, opp, theirs, ours_without_king);
        }

        for queen in queens {
            let square = Square::from(queen);
            attacked |= visible_squares(square, Queen, opp, theirs, ours_without_king);
        }

        let square = Square::from(kings);
        attacked |= visible_squares(square, King, opp, theirs, ours);

        attacked
    }

    pub fn attacked_by(&self, side: Color) -> Bitboard {
        let ours = self.occupied_by(side);
        let theirs = self.occupied_by(side.opp());

        self.compute_attacked_by(side, ours, theirs)
    }

    /// Compute a bitboard of all the pieces putting the current side's king in
    /// check.
    pub fn compute_checkers(&self) -> Bitboard {
        use PieceType::*;

        let us = self.current;
        let them = self.current.opp();
        let ours = self.occupied_by(us);
        let theirs = self.occupied_by(them);

        let king_sq: Square = self.get_bb(King, us).into();

        let pawns = self.piece_bbs[Pawn as usize];
        let rooks = self.piece_bbs[Rook as usize];
        let knights = self.piece_bbs[Knight as usize];
        let bishops = self.piece_bbs[Bishop as usize];
        let queens = self.piece_bbs[Queen as usize];

        let attackers = theirs
            & ((pawns & pawn_attacks(king_sq, us))
                | (rooks & visible_squares(king_sq, Rook, us, ours, theirs))
                | (knights & visible_squares(king_sq, Knight, us, ours, theirs))
                | (bishops & visible_squares(king_sq, Bishop, us, ours, theirs))
                | (queens & visible_squares(king_sq, Queen, us, ours, theirs)));

        attackers
    }

    /// Compute the pin rays that are pinning this player's pieces.
    pub fn compute_pinrays(&self, side: Color) -> Vec<Bitboard> {
        /// See how many of the opponent's sliders are checking our king if all
        /// our pieces weren't there. Then check whether those rays contain a 
        /// single piece. If so, it's pinned. (Note that it would be, by necessity,
        /// one of our pieces, since otherwise the king couldn't have been in check)
        use PieceType::*;
        let king_bb = self.get_bb(King, side);
        let king_sq: Square = king_bb.into();
        let opp = side.opp();

        let ours = self.occupied_by(side);
        let theirs = self.occupied_by(opp);
        let diag_sliders = self.get_bb(Bishop, opp) | self.get_bb(Queen, opp);
        let hv_sliders = self.get_bb(Rook, opp) | self.get_bb(Queen, opp);

        let mut pinrays: Vec<Bitboard> = Vec::new();

        for dir in Direction::DIAG {
            let visible_ray = visible_ray(dir, king_sq, theirs);
            let has_diag_slider = visible_ray & diag_sliders != Bitboard::EMPTY;
            let has_single_piece = (visible_ray & ours).count() == 1;
            if has_diag_slider && has_single_piece {
                pinrays.push(visible_ray);
            }
        }

        for dir in Direction::HV {
            let visible_ray = visible_ray(dir, king_sq, theirs);
            let has_hv_slider = visible_ray & hv_sliders != Bitboard::EMPTY;
            let has_single_piece = (visible_ray & ours).count() == 1;
            if has_hv_slider && has_single_piece {
                pinrays.push(visible_ray);
            }
        }


        pinrays
    }

    /// Figure out if this side's king is in check if we were to remove a set of
    /// blockers.
    pub fn is_xray_check(&self, side: Color, invisible: Bitboard) -> bool {
        use PieceType::*;
        let king_sq: Square = self.get_bb(King, side).into();
        let opp = side.opp();

        let blockers = self.all_occupied().remove(invisible);
        let diag_sliders = self.get_bb(Bishop, opp) | self.get_bb(Queen, opp);
        let ortho_sliders = self.get_bb(Rook, opp) | self.get_bb(Queen, opp);

        let ortho_check = Direction::HV
            .into_iter()
            .map(|dir| king_sq.visible_ray(dir, blockers))
            .any(|ray| ray.has_overlap(ortho_sliders));

        if ortho_check {
            return true;
        };

        let diag_check = Direction::DIAG
            .into_iter()
            .map(|dir| king_sq.visible_ray(dir, blockers))
            .any(|ray| ray.has_overlap(diag_sliders));

        if diag_check {
            return true;
        };
        false
    }

    /// Return the bitboard of pieces checking this side's king if a subset of
    /// blockers were removed.
    /// TODO: Merge this with compute_checkers? 
    /// Can we provide our own bitboard of blockers?
    /// Should compute_checkers take `ours` and `theirs` bitboards? Or simply an
    /// optional mask?
    pub fn compute_xray_checkers(&self, side: Color, invisible: Bitboard) -> Bitboard {
        use PieceType::*;

        let ours = self.occupied_by(side) & !invisible;
        let theirs = self.occupied_by(side.opp()) & !invisible;

        let our_king: Square = self.get_bb(King, side).into();

        let pawns = self.piece_bbs[Pawn as usize];
        let rooks = self.piece_bbs[Rook as usize];
        let knights = self.piece_bbs[Knight as usize];
        let bishops = self.piece_bbs[Bishop as usize];
        let queens = self.piece_bbs[Queen as usize];

        let checkers = theirs
            & ((pawns & pawn_attacks(our_king, side))
                | (rooks & visible_squares(our_king, Rook, side, theirs, ours))
                | (knights & visible_squares(our_king, Knight, side, theirs, ours))
                | (bishops & visible_squares(our_king, Bishop, side, theirs, ours))
                | (queens & visible_squares(our_king, Queen, side, theirs, ours)));

        checkers
    }

    // Compute the map of all squares attacked by this side, given desired 
    // blocker bitboards
    pub fn compute_attacked_by(&self, side: Color, ours: Bitboard, theirs: Bitboard) -> Bitboard {
        use PieceType::*;
        let mut attacked = Bitboard(0);

        let pawns = ours & self.piece_bbs[Pawn as usize];
        let rooks = ours & self.piece_bbs[Rook as usize];
        let knights = ours & self.piece_bbs[Knight as usize];
        let bishops = ours & self.piece_bbs[Bishop as usize];
        let queens = ours & self.piece_bbs[Queen as usize];
        let kings = ours & self.piece_bbs[King as usize];

        for pawn in pawns {
            let square = Square::from(pawn);
            attacked |= pawn_attacks(square, side);
        }

        for knight in knights {
            let square = Square::from(knight);
            attacked |= visible_squares(square, Knight, side, ours, theirs);
        }

        for bishop in bishops {
            let square = Square::from(bishop);
            attacked |= visible_squares(square, Bishop, side, ours, theirs);
        }

        for rook in rooks {
            let square = Square::from(rook);
            attacked |= visible_squares(square, Rook, side, ours, theirs);
        }

        for queen in queens {
            let square = Square::from(queen);
            attacked |= visible_squares(square, Queen, side, ours, theirs);
        }

        let square = Square::from(kings);
        attacked |= visible_squares(square, King, side, ours, theirs);

        attacked
    }

    pub fn occupied_by(&self, side: Color) -> Bitboard {
        self.occupied_squares[side as usize]
    }

    pub fn all_occupied(&self) -> Bitboard {
        self.occupied_squares.into_iter().collect()
    }

    pub fn get_bb(&self, ptype: PieceType, color: Color) -> Bitboard {
        self.piece_bbs[ptype as usize] & self.occupied_by(color)
    }

    /// Check whether the current player is in check
    pub fn in_check(&self) -> bool {
        !self.compute_checkers().is_empty()
    }

    /// Check whether the current player is in checkmate
    /// NOTE: This is fairly intensive, avoid using in hot loops
    pub fn checkmate(&self) -> bool {
        self.in_check() && self.legal_moves().len() == 0 
    }

    /// Check for rule_based draws
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
    /// mate. (This is an automatic draw).
    pub fn insufficient_material(&self) -> bool {
        // NOTE: Since we're looking for efficiency here, we're better off identifying
        // positions where mate can't be _forced_, even if theoretically possible. 
        // The quicker we find a (likely) draw, the better (i.e., just like 
        // checkmate, we should break off the search and return a draw asap.
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

impl Board {
    pub fn from_fen(fen: &str) -> anyhow::Result<Board> {
        let mut parts = fen.split(' ');

        let piece_string = parts.next().ok_or(anyhow!("Invalid FEN string"))?;

        let fen = FEN::try_from(piece_string)?;

        let mut piece_bbs = [Bitboard::EMPTY; PieceType::COUNT];
        let mut occupied_squares = [Bitboard::EMPTY; Color::COUNT];
        let mut piece_list = [None; Square::COUNT];

        // FEN starts with the 8th rank down, so we need to reverse the ranks
        // to go in ascending order
        for (rank, atoms) in fen.ranks.into_iter().rev().enumerate() {
            let mut file: usize = 0;
            for atom in atoms {
                match atom {
                    FENAtom::Gap(n) => {
                        file += n;
                    }

                    FENAtom::Piece(color, piece_type) => {
                        let position = Bitboard::new(rank, file);
                        let sq = Square::from(position);
                        let piece = Piece::new(piece_type, color);

                        piece_list[sq as usize] = Some(piece);

                        piece_bbs[piece_type as usize] |= position;
                        occupied_squares[color as usize] |= position;

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

/// FEN
/// TODO:
/// - Parse piece list into /-delimited strings
/// All the other stuff is pretty much in place already.
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
        let next_player = self.current.to_fen();
        let castling = self.castling_rights.to_fen();
        let en_passant = self
            .en_passant
            .map(|sq| sq.to_string())
            .unwrap_or(String::from("-"));
        let half_moves = self.half_moves;
        let full_moves = self.full_moves;

        format!("{pieces} {next_player} {castling} {en_passant} {half_moves} {full_moves}")
    }
}

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
    if side.is_white() {
        W_PAWN_ATTACKS[square as usize]
    } else {
        B_PAWN_ATTACKS[square as usize]
    }
}
