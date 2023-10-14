use std::{ops::Div, fmt::Display};

use crate::fen::{FEN, FENAtom};

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

#[derive(Debug)]
pub struct Piece {
    color: Color,
    piece_type: PieceType,
    position: Position,
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
}


#[derive(Debug)]
pub struct Board {
    pieces: Vec<Piece>
}

impl Board {
    pub fn at(&self, rank: u64, file: u64) -> Option<&Piece> {
        self.pieces
            .iter()
            .find(|&piece| 
                piece.position.rank() == rank 
                && piece.position.file() == file
            )
    }

    pub fn pretty_print(&self) -> String {
        let mut lines: Vec<String> = vec![];
        lines.push("  a b c d e f g h ".to_owned());

        for rank in 0..8 {
            let rank_label = rank.to_string();
            let mut line: Vec<&str> = vec![];

            line.push(&rank_label);
            line.push(" ");

            for file in 0..8 {
                let square = match self.at(rank, file) {
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

        lines.join("\n")
    }
}

impl TryFrom<&str> for Board {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let fen = FEN::try_from(value)?;
        let mut board: Board = Board { pieces: vec![] };

        for (rank, atoms) in fen.ranks.into_iter().enumerate() {
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
                            position: Position::new(rank as u64, file)
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

        for rank in 0..8 {
            let rank_label = rank.to_string();
            let mut line: Vec<&str> = vec![];

            line.push(&rank_label);
            line.push(" ");

            for file in 0..8 {
                let square = match self.at(rank, file) {
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
