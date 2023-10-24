use std::fmt::Display;
use std::str::FromStr;
use crate::bitboard::Bitboard;
use crate::fen::{FEN, FENAtom};
use crate::movegen::castling::CastlingRights;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Color {
    White = 0,
    Black = 1,
}

impl Color {
    pub fn opp(&self) -> Self {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White
        }
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
    pub fn is_white(&self) -> bool {
        self.color == Color::White
    }

    pub fn is_black(&self) -> bool {
        self.color == Color::Black
    }

    pub fn forward(&self) -> Option<Piece> {
        let mut piece = self.clone();

        let forward_pos = if piece.is_white() {
            piece.position.up()

        } else {
            self.position.down()
        }?;

        piece.position = forward_pos;

        Some(piece)
    }
}

impl Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let piece = match (self.color, self.piece_type) {
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


// TODO: Should Board store its pieces in a HashMap<Position, Piece>?
#[derive(Debug)]
pub struct Board {
    pub pieces: Vec<Piece>,
    pub castling_rights: CastlingRights,
    pub occupied_squares: [Bitboard; 2],
    pub attacked_squares: [Bitboard; 2],
}

impl Board {
    pub fn new() -> Board {
        Board::from_str("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap()
    }

    pub fn at_coords(&self, rank: u64, file: u64) -> Option<&Piece> {
        self.pieces
            .iter()
            .find(|&piece| 
                piece.position.rank() == rank 
                && piece.position.file() == file
            )
    }

    pub fn add(&mut self, piece: Piece) {
        self.pieces.push(piece);
        self.occupied_squares[piece.color as usize].add_in_place(piece.position);
        self.attacked_squares[piece.color as usize]
            .add_in_place(piece.attacked_squares(self));
    }

    pub fn remove_at(&mut self, position: &Bitboard) -> Option<Piece>{
        let idx = self.pieces.iter().position(|p| p.position == *position)?;
        let piece = self.pieces.swap_remove(idx);
        self.occupied_squares[piece.color as usize].remove_in_place(piece.position);
        self.attacked_squares[piece.color as usize]
            .remove_in_place(piece.attacked_squares(self));

        Some(piece)
    }

    pub fn get(&self, position: &Bitboard) -> Option<&Piece> {
        self.pieces
            .iter()
            .find(|piece| piece.position == *position)
    }

    pub fn is_empty(&self, position: &Bitboard) -> bool {
        self.get(position).is_none()
    }

    pub fn has_piece(&self, position: &Bitboard) -> bool {
        self.get(position).is_some()
    }

    pub fn has_colored_piece(&self, position: &Bitboard, color: Color) -> bool {
        self.get(position).filter(|piece| piece.color == color).is_some()
    }


    pub fn get_mut(&mut self, position: &Bitboard) -> Option<&mut Piece> {
        self.pieces
            .iter_mut()
            .find(|piece| &piece.position == position)
    }

    pub fn up_while_empty(&self, position: &Bitboard) -> Bitboard {
        position.scan_up()
            .into_iter()
            .take_while(|pos| self.get(pos).is_none())
            .collect()
    }

    pub fn left_while_empty(&self, position: &Bitboard) -> Bitboard {
        position.scan_left()
            .into_iter()
            .take_while(|pos| self.get(pos).is_none())
            .collect()
    }

    pub fn right_while_empty(&self, position: &Bitboard) -> Bitboard {
        position.scan_right()
            .into_iter()
            .take_while(|pos| self.get(pos).is_none())
            .collect()
    }

    pub fn down_while_empty(&self, position: &Bitboard) -> Bitboard {
        position.scan_down()
            .into_iter()
            .take_while(|pos| self.get(pos).is_none())
            .collect()
    }

    pub fn first_piece_up(&self, position: &Bitboard) -> Option<&Piece> {
        position.scan_up().iter().find_map(|pos| self.get(pos))
    }

    pub fn first_piece_down(&self, position: &Bitboard) -> Option<&Piece> {
        position.scan_down().iter().find_map(|pos| self.get(pos))
    }

    pub fn first_piece_left(&self, position: &Bitboard) -> Option<&Piece> {
        position.scan_left().iter().find_map(|pos| self.get(pos))
    }

    pub fn first_piece_right(&self, position: &Bitboard) -> Option<&Piece> {
        position.scan_right().iter().find_map(|pos| self.get(pos))
    }

    pub fn scan_empty<F: Fn(&Bitboard) -> Option<Bitboard>>(
        &self, 
        position: &Bitboard, 
        next: F
    ) -> Vec<Bitboard> {
        position.scan(next)
            .into_iter()
            .take_while(|pos| self.is_empty(pos))
            .collect()
    }

    pub fn first_piece<F: Fn(&Bitboard) -> Option<Bitboard>>(
        &self, 
        position: &Bitboard, 
        next: F
    ) -> Option<&Piece> {
        position.scan(next)
            .iter()
            .find_map(|pos| self.get(pos))
    }
}

impl FromStr for Board {
    type Err = anyhow::Error;

    //TODO: Actually parse the other fields, like next player, castling rights, etc...
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let fen = FEN::try_from(value)?;
        let mut board: Board = Board { 
            pieces: vec![],
            castling_rights: CastlingRights::from_str(value)?,
            attacked_squares: [Bitboard::default(), Bitboard::default()],
            occupied_squares: [Bitboard::default(), Bitboard::default()]
        };

        // FEN starts with the 8th rank down, so we need to reverse the ranks
        // to go in ascending order
        for (rank, atoms) in fen.ranks.into_iter().rev().enumerate() {
            let mut file: u64 = 0;
            for atom in atoms {
                match atom {
                    FENAtom::Gap(n) => {
                        file += n;
                    },

                    FENAtom::Piece(color, piece_type) => {
                        board.pieces.push(Piece { 
                            color, 
                            piece_type, 
                            position: Bitboard::new(rank as u64, file),
                            has_moved: false
                        });
                        file += 1;

                    },
                }
            }
        }

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
                let square = match self.at_coords(rank, file) {
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
