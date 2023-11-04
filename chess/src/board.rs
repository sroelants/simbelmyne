use crate::bitboard::{Bitboard, Step};
use crate::movegen::attack_boards::{W_PAWN_ATTACKS, B_PAWN_ATTACKS, Direction, ATTACK_RAYS};
use crate::movegen::castling::CastlingRights;
use crate::movegen::moves::{visible_squares, visible_ray};
use crate::util::fen::{FENAtom, FEN};
use anyhow::anyhow;
use std::fmt::Display;
use std::ops::Not;
use std::str::FromStr;

#[rustfmt::skip]
const SQUARE_NAMES: [&str; Square::COUNT] = [
    "a1", "b1", "c1", "d1", "e1", "f1", "g1", "h1", 
    "a2", "b2", "c2", "d2", "e2", "f2", "g2", "h2",
    "a3", "b3", "c3", "d3", "e3", "f3", "g3", "h3", 
    "a4", "b4", "c4", "d4", "e4", "f4", "g4", "h4",
    "a5", "b5", "c5", "d5", "e5", "f5", "g5", "h5", 
    "a6", "b6", "c6", "d6", "e6", "f6", "g6", "h6",
    "a7", "b7", "c7", "d7", "e7", "f7", "g7", "h7", 
    "a8", "b8", "c8", "d8", "e8", "f8", "g8", "h8",
];

#[rustfmt::skip]
#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Square {
    A1, B1, C1, D1, E1, F1, G1, H1,
    A2, B2, C2, D2, E2, F2, G2, H2,
    A3, B3, C3, D3, E3, F3, G3, H3, 
    A4, B4, C4, D4, E4, F4, G4, H4,
    A5, B5, C5, D5, E5, F5, G5, H5,
    A6, B6, C6, D6, E6, F6, G6, H6,
    A7, B7, C7, D7, E7, F7, G7, H7,
    A8, B8, C8, D8, E8, F8, G8, H8,
}

use itertools::Itertools;
use Square::*;
impl Square {
    pub const ALL: [Square; Square::COUNT] = [
        A1, B1, C1, D1, E1, F1, G1, H1, 
        A2, B2, C2, D2, E2, F2, G2, H2, 
        A3, B3, C3, D3, E3, F3, G3, H3, 
        A4, B4, C4, D4, E4, F4, G4, H4, 
        A5, B5, C5, D5, E5, F5, G5, H5, 
        A6, B6, C6, D6, E6, F6, G6, H6, 
        A7, B7, C7, D7, E7, F7, G7, H7, 
        A8, B8, C8, D8, E8, F8, G8, H8,
    ];

    pub const COUNT: usize = 64;
    pub const W_PAWN_RANK: usize = 1;
    pub const B_PAWN_RANK: usize = 6;
    pub const W_DPUSH_RANK: usize = 3;
    pub const B_DPUSH_RANK: usize = 4;

    pub fn new(rank: usize, file: usize) -> Square {
        Square::ALL[rank * 8 + file]
    }

    pub fn try_new(rank: usize, file: usize) -> Option<Square> {
        if rank <= 7 && file <= 7 {
            Some(Square::new(rank, file))
        } else {
            None
        }
    }

    pub fn try_from_usize(value: usize) -> Option<Square> {
        if value < 64 {
            Some(Square::ALL[value])
        } else {
            None
        }
    }

    pub fn to_alg(&self) -> &'static str {
        SQUARE_NAMES[*self as usize]
    }

    pub fn rank(&self) -> usize {
        (*self as usize) / 8
    }

    pub fn file(&self) -> usize {
        (*self as usize) % 8
    }

    pub fn forward(&self, side: Color) -> Option<Square> {
        if side.is_white() {
            Square::try_new(self.rank() + 1, self.file())
        } else {
            Square::try_new(self.rank() - 1, self.file())
        }
    }

    pub fn backward(&self, side: Color) -> Option<Square> {
        self.forward(side.opp())
    }

    pub fn is_double_push(source: Square, target: Square) -> bool {
        (source.rank() == Self::W_PAWN_RANK && target.rank() == Self::W_DPUSH_RANK
            || source.rank() == Self::B_PAWN_RANK && target.rank() == Self::B_DPUSH_RANK)
            && source.file() == target.file()
    }
}

impl From<usize> for Square {
    fn from(value: usize) -> Self {
        Square::ALL[value]
    }
}

impl Display for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", SQUARE_NAMES[*self as usize])?;
        Ok(())
    }
}

impl FromStr for Square {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self> {
        let idx = SQUARE_NAMES
            .iter()
            .position(|&name| name == s.to_lowercase())
            .ok_or(anyhow!("Not a valid square identifier"))?;

        Ok(Square::ALL[idx].to_owned())
    }
}

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PieceType {
    Pawn = 0,
    Knight = 1,
    Bishop = 2,
    Rook = 3,
    Queen = 4,
    King = 5,
}

impl PieceType {
    const COUNT: usize = 6;

    pub fn is_pawn(&self) -> bool {
        *self == PieceType::Pawn
    }

    pub fn is_rook(&self) -> bool {
        *self == PieceType::Rook
    }

    pub fn is_knight(&self) -> bool {
        *self == PieceType::Knight
    }

    pub fn is_bishop(&self) -> bool {
        *self == PieceType::Bishop
    }

    pub fn is_queen(&self) -> bool {
        *self == PieceType::Queen
    }

    pub fn is_king(&self) -> bool {
        *self == PieceType::King
    }
}

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Color {
    White = 0,
    Black = 1,
}

impl Color {
    const COUNT: usize = 2;

    pub fn opp(&self) -> Self {
        !*self
    }

    pub fn is_white(&self) -> bool {
        *self == Color::White
    }

    pub fn is_black(&self) -> bool {
        *self == Color::Black
    }

    pub fn to_fen(&self) -> String {
        if self.is_white() {
            String::from("w")
        } else {
            String::from("b")
        }
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Color::White => write!(f, "White")?,
            Color::Black => write!(f, "Black")?,
        }
        Ok(())
    }
}

impl FromStr for Color {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self> {
        match s {
            "w" | "W" | "white" | "White" => Ok(Color::White),
            "b" | "B" | "black" | "Black" => Ok(Color::Black),
            _ => Err(anyhow!("Not a valid color string"))?,
        }
    }
}
impl Not for Color {
    type Output = Color;

    fn not(self) -> Self::Output {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Piece {
    pub color: Color,
    pub piece_type: PieceType,
    pub position: Bitboard,
}

#[allow(dead_code)]
impl Piece {
    pub fn color(&self) -> Color {
        self.color
    }

    pub fn piece_type(&self) -> PieceType {
        self.piece_type
    }

    pub fn is_pawn(&self) -> bool {
        self.piece_type() == PieceType::Pawn
    }

    pub fn is_rook(&self) -> bool {
        self.piece_type() == PieceType::Rook
    }

    pub fn is_knight(&self) -> bool {
        self.piece_type() == PieceType::Knight
    }

    pub fn is_bishop(&self) -> bool {
        self.piece_type() == PieceType::Bishop
    }

    pub fn is_queen(&self) -> bool {
        self.piece_type() == PieceType::Queen
    }

    pub fn is_king(&self) -> bool {
        self.piece_type() == PieceType::King
    }

    pub fn is_slider(&self) -> bool {
        self.is_rook() || self.is_bishop() || self.is_queen()
    }
}

impl Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let piece = match (self.color(), self.piece_type()) {
            (Color::White, PieceType::Pawn) => "P",
            (Color::White, PieceType::Rook) => "R",
            (Color::White, PieceType::Knight) => "N",
            (Color::White, PieceType::Bishop) => "B",
            (Color::White, PieceType::Queen) => "Q",
            (Color::White, PieceType::King) => "K",

            (Color::Black, PieceType::Pawn) => "p",
            (Color::Black, PieceType::Rook) => "r",
            (Color::Black, PieceType::Knight) => "n",
            (Color::Black, PieceType::Bishop) => "b",
            (Color::Black, PieceType::Queen) => "q",
            (Color::Black, PieceType::King) => "k",
        };

        write!(f, "{piece}")
    }
}

#[derive(Debug, Copy, Clone)]
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
    pub full_moves: u8,
}

#[allow(dead_code)]
impl Board {
    pub fn new() -> Board {
        Board::from_str("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap()
    }

    pub fn get_at(&self, square: Square) -> Option<&Piece> {
        self.piece_list.get(square as usize)?.as_ref()
    }

    pub fn add_at(&mut self, square: Square, piece: Piece) {
        let bb: Bitboard = square.into();
        self.piece_list[square as usize] = Some(piece);

        self.occupied_squares[piece.color as usize] |= bb;
        self.piece_bbs[piece.piece_type() as usize] |= bb;
    }

    pub fn remove_at(&mut self, square: Square) -> Option<Piece> {
        let bb: Bitboard = square.into();
        let piece = self.piece_list[square as usize]?;

        self.piece_list[square as usize] = None;

        self.occupied_squares[piece.color as usize] ^= bb;
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

    /// Compute a bitboard of the requested side's pieces that are putting the
    /// opponent king in check
    pub fn compute_checkers(&self, side: Color) -> Bitboard {
        use PieceType::*;

        let ours = self.occupied_by(side);
        let theirs = self.occupied_by(side.opp());

        let opp_king = self.get_bb(King, side.opp());
        let opp_king: Square = opp_king.into();

        let pawns = self.piece_bbs[Pawn as usize];
        let rooks = self.piece_bbs[Rook as usize];
        let knights = self.piece_bbs[Knight as usize];
        let bishops = self.piece_bbs[Bishop as usize];
        let queens = self.piece_bbs[Queen as usize];

        let attackers = ours
            & ((pawns & visible_squares(opp_king, Pawn, side.opp(), theirs, ours))
                | (rooks & visible_squares(opp_king, Rook, side.opp(), theirs, ours))
                | (knights & visible_squares(opp_king, Knight, side.opp(), theirs, ours))
                | (bishops & visible_squares(opp_king, Bishop, side.opp(), theirs, ours))
                | (queens & visible_squares(opp_king, Queen, side.opp(), theirs, ours)));

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

        for dir in Direction::BISHOP {
            let visible_ray = visible_ray(dir, king_sq, theirs);
            let has_diag_slider = visible_ray & diag_sliders != Bitboard::EMPTY;
            let has_single_piece = (visible_ray & ours).is_single();
            if has_diag_slider && has_single_piece {
                pinrays.push(visible_ray);
            }
        }

        for dir in Direction::ROOK {
            let visible_ray = visible_ray(dir, king_sq, theirs);
            let has_hv_slider = visible_ray & hv_sliders != Bitboard::EMPTY;
            let has_single_piece = (visible_ray & ours).is_single();
            if has_hv_slider && has_single_piece {
                pinrays.push(visible_ray);
            }
        }


        pinrays
    }

    pub fn is_xray_check(&self, side: Color, invisible: Bitboard) -> bool {
        use PieceType::*;
        let king_bb = self.get_bb(King, side);
        let opp = side.opp();

        let blockers = self.all_occupied().remove(invisible);
        let diag_sliders = self.get_bb(Bishop, opp) | self.get_bb(Queen, opp);
        let ortho_sliders = self.get_bb(Rook, opp) | self.get_bb(Queen, opp);

        let ortho_check = Step::ORTHO_DIRS
            .into_iter()
            .map(|dir| king_bb.visible_ray(dir, blockers))
            .any(|ray| ray.has_overlap(ortho_sliders));

        if ortho_check {
            return true;
        };

        let diag_check = Step::DIAG_DIRS
            .into_iter()
            .map(|dir| king_bb.visible_ray(dir, blockers))
            .any(|ray| ray.has_overlap(diag_sliders));

        if diag_check {
            return true;
        };
        false
    }

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
                        let piece = Piece {
                            color,
                            piece_type,
                            position,
                        };

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

        let half_moves: u8 = parts.next().ok_or(anyhow!("Invalid FEN string"))?.parse()?;

        let full_moves: u8 = parts.next().ok_or(anyhow!("Invalid FEN string"))?.parse()?;

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

        for rank in (0..8).rev() {
            let mut line: Vec<String> = vec![];

            line.push((rank + 1).to_string());
            line.push(" ".to_string());

            for file in 0..8 {
                let square = match self.get_at(Square::new(rank, file)) {
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
fn pawn_attacks(square: Square, side: Color) -> Bitboard {
    if side.is_white() {
        W_PAWN_ATTACKS[square as usize]
    } else {
        B_PAWN_ATTACKS[square as usize]
    }
}
