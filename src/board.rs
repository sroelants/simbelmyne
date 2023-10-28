use std::fmt::Display;
use std::str::FromStr;
use crate::bitboard::Bitboard;
use crate::fen::{FEN, FENAtom};
use crate::movegen::castling::CastlingRights;


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

    pub fn new(rank: usize, file: usize) -> Square {
        Square::ALL[rank * 8 + file]
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

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PieceType {
    Pawn   = 0,
    Knight = 1,
    Bishop = 2,
    Rook   = 3,
    Queen  = 4,
    King   = 5,
}

impl PieceType {
    const COUNT: usize = 6;
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
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White
        }
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Color::White => write!(f, "White")?,
            Color::Black => write!(f, "Black")?
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Piece {
    pub color: Color,
    pub piece_type: PieceType,
    pub position: Bitboard,
    pub has_moved: bool,
}

impl Piece {
    pub fn color(&self) -> Color {
        self.color
    }

    pub fn piece_type(&self) -> PieceType {
        self.piece_type
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


#[derive(Debug)]
pub struct Board {
    /// Squares occupied by a given piece type
    pub piece_bbs: [Bitboard; PieceType::COUNT],

    /// Squares occupied _by_ a given side
    pub occupied_squares: [Bitboard; 2],

    /// Squares attacked _by_ a given side
    pub attacked_squares: [Bitboard; 2],

    /// Endangered squares that limit king movenment
    /// These are similar, but subtly different from the attacked_squares
    /// https://peterellisjones.com/posts/generating-legal-chess-moves-efficiently/#gotcha-king-moves-away-from-a-checking-slider
    pub king_danger_squares: [Bitboard; 2],

    /// List of pieces, indexable by a Square, more efficient for lookups than `pieces`
    pub piece_list: [Option<Piece>; Square::COUNT],

    /// Keeps track of what types of castling are still allowed
    pub castling_rights: CastlingRights,
}

impl Board {
    #[allow(dead_code)]
    pub fn new() -> Board {
        Board::from_str("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap()
    }

    pub fn get_at(&self, square: Square) -> Option<&Piece> {
        self.piece_list[square as usize].as_ref()
    }

    pub fn add_at(&mut self, square: Square, piece: Piece) {
        let bb: Bitboard = square.into();
        self.piece_list[square as usize] = Some(piece);

        self.occupied_squares[piece.color as usize] |= bb;
        self.piece_bbs[piece.piece_type() as usize] |= bb;

        self.refresh_attacked_squares();
        self.refresh_danger_squares();
    }

    pub fn remove_at(&mut self, square: Square) -> Option<Piece>{
        let bb: Bitboard = square.into();
        let piece = self.piece_list[square as usize]?;

        self.piece_list[square as usize] = None;

        self.occupied_squares[piece.color as usize] ^= bb;
        self.piece_bbs[piece.piece_type() as usize] ^= bb;

        Some(piece)
    }

    pub fn refresh_attacked_squares(&mut self) {
        let blockers = self.all_occupied();

        self.attacked_squares = [
            self.compute_attacked_by(Color::White, blockers),
            self.compute_attacked_by(Color::Black, blockers)
        ];
    }

    pub fn refresh_danger_squares(&mut self) {
        let blockers = self.all_occupied();
        let without_wk = blockers.remove(self.get_bb(PieceType::King, Color::White));
        let without_bk = blockers.remove(self.get_bb(PieceType::King, Color::Black));

        self.king_danger_squares = [
            self.compute_attacked_by(Color::White, without_bk),
            self.compute_attacked_by(Color::Black, without_wk)
        ];
    }

    pub fn attacked_by(&self, side: Color) -> Bitboard {
        self.attacked_squares[side as usize]
    }

    pub fn compute_attacked_by(&mut self, side: Color, blockers: Bitboard) -> Bitboard{
        self.piece_list
            .iter()
            .flatten()
            .filter(|piece| piece.color == side)
            .map(|piece| piece.visible_squares(blockers))
            .collect::<Bitboard>()
            .remove(self.occupied_by(side))
    }

    pub fn occupied_by(&self, side: Color) -> Bitboard{
        self.occupied_squares[side as usize]
    }

    pub fn all_occupied(&self) -> Bitboard {
        self.occupied_squares.into_iter().collect()
    }

    pub fn get_bb(&self, ptype: PieceType, color: Color) -> Bitboard {
        self.piece_bbs[ptype as usize] & self.occupied_by(color)
    }
}

impl FromStr for Board {
    type Err = anyhow::Error;

    //TODO: Actually parse the other fields, like next player, castling rights, etc...
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let fen = FEN::try_from(value)?;

        let mut pieces = Vec::new();

        // FEN starts with the 8th rank down, so we need to reverse the ranks
        // to go in ascending order
        for (rank, atoms) in fen.ranks.into_iter().rev().enumerate() {
            let mut file: usize = 0;
            for atom in atoms {
                match atom {
                    FENAtom::Gap(n) => {
                        file += n;
                    },

                    FENAtom::Piece(color, piece_type) => {
                        pieces.push(Piece { 
                            color, 
                            piece_type, 
                            position: Bitboard::new(rank, file),
                            has_moved: false
                        });

                        file += 1;
                    },
                }
            }
        }

        let mut piece_list = [None; Square::COUNT];
        let mut occupied_squares = [Bitboard::default(); Color::COUNT];
        let mut piece_bbs = [Bitboard::default(); PieceType::COUNT];

        for piece in pieces {
            occupied_squares[piece.color as usize] |= piece.position;
            piece_bbs[piece.color() as usize] |= piece.position;
            piece_list[Square::from(piece.position) as usize] = Some(piece)
        }

        let attacked_squares = [Bitboard::default(); Color::COUNT];
        let king_danger_squares = [Bitboard::default(); Color::COUNT];
        let castling_rights = CastlingRights::from_str(value)?;

        let mut board = Board {
            piece_bbs,
            piece_list,
            occupied_squares,
            attacked_squares,
            king_danger_squares,
            castling_rights,
        };

        board.refresh_attacked_squares();

        Ok(board)
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
                    None => ".".to_string()
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
