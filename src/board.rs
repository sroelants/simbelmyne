use std::{ops::Div, fmt::Display};
use anyhow::anyhow;

use crate::{fen::{FEN, FENAtom}, parse};

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
    Black,
    White
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Position(u64);

impl Position {
    pub fn new(rank: u64, file: u64) -> Self {
        Position((1 << 8*rank) << file)
    }

    pub fn rank(&self) -> u64 {
        self.0.trailing_zeros().div(8).into()
    }

    pub fn file(&self) -> u64 {
        (self.0.trailing_zeros() % 8).try_into().unwrap()
    }

    pub fn up(&self) -> Option<Self> {
        self.0.checked_shl(8).map(Position)
    }

    pub fn down(&self) -> Option<Self> {
        self.0.checked_shr(8).map(Position)
    }

    pub fn left(&self) -> Option<Self> {
        if self.file() > 0 { Some(Position(self.0 << 1)) } else { None }
    }

    pub fn right(&self) -> Option<Self> {
        if self.file() < 7 { Some(Position(self.0 << 1)) } else { None }
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let rank = (self.rank() + 1).to_string();

        let file = match self.file() {
            0 => "a",
            1 => "b",
            2 => "c",
            3 => "d",
            4 => "e",
            5 => "f",
            6 => "g",
            7 => "h",
            _ => panic!("unreachable")
        }.to_string();

        write!(f, "{}", vec![file, rank].join(""))
    }
}

impl Bitboard for Position {
    fn bits(&self) -> u64 {
        self.0
    }

    fn set(&mut self, bits: u64) {
        self.0 = bits;
    }

}

pub struct PositionSet(pub u64);

impl Bitboard for PositionSet {
    fn bits(&self) -> u64 {
        self.0
    }

    fn set(&mut self, bits: u64) {
        self.0 = bits;
    }
}

/// This trait probably holds all of the bitboard specific logic 
/// (all your set operations)
pub trait Bitboard {
    fn bits(&self) -> u64;

    fn set(&mut self, bits: u64);

    fn add<B: Bitboard>(&mut self, positions: B) {
        self.set(self.bits() | positions.bits());
    }

    fn remove<B: Bitboard>(&mut self, positions: B) {
        self.set(self.bits() & !positions.bits());
    }
    
}

impl TryFrom<&str> for Position {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let (_, (file, rank)) = parse::algebraic_square(value)
            .map_err(|_| anyhow!("Failed to parse"))?;
        Ok(Position::new(rank, file))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Piece {
    pub color: Color,
    pub piece_type: PieceType,
    pub position: Position,
    pub has_moved: bool,
}

impl Piece {
    fn algebraic(&self) -> &str {
        match (self.color, self.piece_type) {
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
        }
    }

    pub fn is_white(&self) -> bool {
        self.color == Color::White
    }

    pub fn is_black(&self) -> bool {
        self.color == Color::Black
    }
}


#[derive(Debug)]
pub struct Board {
    pub pieces: Vec<Piece>
}

impl Board {
    pub fn at_coords(&self, rank: u64, file: u64) -> Option<&Piece> {
        self.pieces
            .iter()
            .find(|&piece| 
                piece.position.rank() == rank 
                && piece.position.file() == file
            )
    }

    pub fn get(&self, position: Position) -> Option<&Piece> {
        self.pieces
            .iter()
            .find(|&piece| piece.position == position)
    }

    pub fn get_mut(&mut self, position: Position) -> Option<&mut Piece> {
        self.pieces
            .iter_mut()
            .find(|piece| piece.position == position)
    }

    pub fn remove_at(&mut self, pos: Position) -> Option<Piece>{
        if let Some(idx) = self.pieces.iter().position(|p| p.position == pos) {
            Some(self.pieces.swap_remove(idx))
        } else {
            None
        }
    }
}

impl TryFrom<&str> for Board {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let fen = FEN::try_from(value)?;
        let mut board: Board = Board { pieces: vec![] };

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
                            position: Position::new(rank as u64, file),
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
        lines.push("  a b c d e f g h ".to_owned());

        for rank in (0..8).rev() {
            let rank_label = (rank + 1).to_string();
            let mut line: Vec<&str> = vec![];

            line.push(&rank_label);
            line.push(" ");

            for file in 0..8 {
                let square = match self.at_coords(rank, file) {
                    Some(piece) => piece.algebraic(),
                    None => "."
                };

                line.push(square);
                line.push(" ");
            }
            line.push(&rank_label);
            let line = line.join("");

            lines.push(line);
        }
        lines.push("  a b c d e f g h ".to_owned());

        write!(f, "{}", lines.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use crate::board::Position;

    #[test]
    fn position_new_00() {
        assert_eq!(Position::new(0,0).0, 1);
    }

    #[test]
    fn position_new_10() {
        assert_eq!(Position::new(1,0).0.trailing_zeros(), 8 );
    }

    #[test]
    fn position_new_05() {
        assert_eq!(Position::new(0,5).0.trailing_zeros(), 5 );
    }

    #[test]
    fn position_new_25() {
        assert_eq!(Position::new(2,5).0.trailing_zeros(), 21 );
    }

    #[test]
    fn position_rank() {
        assert_eq!(Position::new(2,5).rank(), 2 );
        assert_eq!(Position::new(7,7).rank(), 7 );
        assert_eq!(Position::new(4,2).rank(), 4 );
    }

    #[test]
    fn position_file() {
        assert_eq!(Position::new(2,5).file(), 5 );
        assert_eq!(Position::new(7,7).file(), 7 );
        assert_eq!(Position::new(4,2).file(), 2 );
    }
}
