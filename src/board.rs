use std::{ops::Div, fmt::Display};
use anyhow::anyhow;

use crate::{fen::{FEN, FENAtom}, parse, moves::CastlingRights};

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

impl Color {
    pub fn opp(&self) -> Self {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct Bitboard(u64);

impl Bitboard {
    pub fn new(rank: u64, file: u64) -> Self {
        Bitboard((1 << 8*rank) << file)
    }

    pub fn rank(&self) -> u64 {
        self.0.trailing_zeros().div(8).into()
    }

    pub fn file(&self) -> u64 {
        (self.0.trailing_zeros() % 8).try_into().unwrap()
    }

    pub fn up(&self) -> Option<Self> {
        if self.0.leading_zeros() > 8 {
            Some(Bitboard(self.0 << 8))
        } else {
            None
        }
    }

    pub fn down(&self) -> Option<Self> {
        if self.0.trailing_zeros() > 8 {
            Some(Bitboard(self.0 >> 8))
        } else {
            None
        }
    }

    pub fn left(&self) -> Option<Self> {
        if self.file() > 0 { Some(Bitboard(self.0 >> 1)) } else { None }
    }

    pub fn right(&self) -> Option<Self> {
        if self.file() < 7 { Some(Bitboard(self.0 << 1)) } else { None }
    }

    pub fn up_left(&self) -> Option<Self> {
        self.up().and_then(|pos| pos.left())
    }

    pub fn up_right(&self) -> Option<Self> {
        self.up().and_then(|pos| pos.right())
    }

    pub fn down_left(&self) -> Option<Self> {
        self.down().and_then(|pos| pos.left())
    }

    pub fn down_right(&self) -> Option<Self> {
        self.down().and_then(|pos| pos.right())
    }

    pub fn forward(&self, color: Color) -> Option<Self> {
        match color {
            Color::White => self.up(),
            Color::Black => self.down()
        }
    }

    pub fn scan_up(&self) -> Vec<Self> {
        std::iter::successors(self.up(), |current| current.up()).collect()
    }

    pub fn scan_right(&self) -> Vec<Self> {
        std::iter::successors(self.right(), |current| current.right()).collect()
    }

    pub fn scan_down(&self) -> Vec<Self> {
        std::iter::successors(self.down(), |current| current.down()).collect()
    }

    pub fn scan_left(&self) -> Vec<Self> {
        std::iter::successors(self.left(), |current| current.left()).collect()
    }

    pub fn scan_up_left(&self) -> Vec<Self> {
        std::iter::successors(self.up_left(), |current| current.up_left())
            .collect()
    }

    pub fn scan_up_right(&self) -> Vec<Self> {
        std::iter::successors(self.up_right(), |current| current.up_right())
            .collect()
    }

    pub fn scan_down_left(&self) -> Vec<Self> {
        std::iter::successors(self.down_left(), |current| current.down_left())
            .collect()
    }

    pub fn scan_down_right(&self) -> Vec<Self> {
        std::iter::successors(self.down_right(), |current| current.down_right())
            .collect()
    }

    pub fn scan<F: Fn(&Bitboard) -> Option<Bitboard>>(&self, next: F) -> Vec<Self> {
        std::iter::successors(next(self), |pos| next(pos)).collect()
    }

    pub fn add_in_place(&mut self, positions: Self) {
        self.0 = self.0 | positions.0;
    }

    pub fn add(&self, bitboard: Self) -> Bitboard {
        Bitboard(self.0 | bitboard.0)
    }


    pub fn remove(&mut self, positions: Self) {
        self.0 = self.0 & !positions.0;
    }

    pub fn contains(&self, positions: Self) -> bool {
        self.0 & positions.0 != 0
    }

    pub fn bits(&self) -> u64 {
        self.0
    }
}

impl Display for Bitboard {
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

impl FromIterator<Bitboard> for Bitboard {
    fn from_iter<T: IntoIterator<Item = Bitboard>>(iter: T) -> Self {
        let mut result = Bitboard::default();

        for positions in iter {
            result.add_in_place(positions);
        }

        result
    }
}

impl From<Vec<Bitboard>> for Bitboard {
    fn from(boards: Vec<Bitboard>) -> Bitboard {
        let mut result = Bitboard::default();

        for board in boards {
            result.add_in_place(board);
        }

        result
    }
}

impl TryFrom<&str> for Bitboard {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let (_, (file, rank)) = parse::algebraic_square(value)
            .map_err(|_| anyhow!("Failed to parse"))?;
        Ok(Bitboard::new(rank, file))
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
}

impl Board {
    pub fn new() -> Board {
        Board::try_from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap()
    }

    pub fn at_coords(&self, rank: u64, file: u64) -> Option<&Piece> {
        self.pieces
            .iter()
            .find(|&piece| 
                piece.position.rank() == rank 
                && piece.position.file() == file
            )
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


    pub fn get_mut(&mut self, position: Bitboard) -> Option<&mut Piece> {
        self.pieces
            .iter_mut()
            .find(|piece| piece.position == position)
    }

    pub fn remove_at(&mut self, pos: Bitboard) -> Option<Piece>{
        if let Some(idx) = self.pieces.iter().position(|p| p.position == pos) {
            Some(self.pieces.swap_remove(idx))
        } else {
            None
        }
    }

    pub fn up_while_empty(&self, position: &Bitboard) -> Vec<Bitboard> {
        position.scan_up()
            .into_iter()
            .take_while(|pos| self.get(pos).is_none())
            .collect()
    }

    pub fn left_while_empty(&self, position: &Bitboard) -> Vec<Bitboard> {
        position.scan_left()
            .into_iter()
            .take_while(|pos| self.get(pos).is_none())
            .collect()
    }

    pub fn right_while_empty(&self, position: &Bitboard) -> Vec<Bitboard> {
        position.scan_right()
            .into_iter()
            .take_while(|pos| self.get(pos).is_none())
            .collect()
    }

    pub fn down_while_empty(&self, position: &Bitboard) -> Vec<Bitboard> {
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

impl TryFrom<&str> for Board {
    type Error = String;

    //TODO: Actually parse the other fields, like next player, castling rights, etc...
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let fen = FEN::try_from(value)?;
        let mut board: Board = Board { 
            pieces: vec![],
            castling_rights: CastlingRights::new()
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

#[cfg(test)]
mod tests {
    use crate::board::Bitboard;

    #[test]
    fn position_new_00() {
        assert_eq!(Bitboard::new(0,0).0, 1);
    }

    #[test]
    fn position_new_10() {
        assert_eq!(Bitboard::new(1,0).0.trailing_zeros(), 8 );
    }

    #[test]
    fn position_new_05() {
        assert_eq!(Bitboard::new(0,5).0.trailing_zeros(), 5 );
    }

    #[test]
    fn position_new_25() {
        assert_eq!(Bitboard::new(2,5).0.trailing_zeros(), 21 );
    }

    #[test]
    fn position_rank() {
        assert_eq!(Bitboard::new(2,5).rank(), 2 );
        assert_eq!(Bitboard::new(7,7).rank(), 7 );
        assert_eq!(Bitboard::new(4,2).rank(), 4 );
    }

    #[test]
    fn position_file() {
        assert_eq!(Bitboard::new(2,5).file(), 5 );
        assert_eq!(Bitboard::new(7,7).file(), 7 );
        assert_eq!(Bitboard::new(4,2).file(), 2 );
    }

    #[test]
    fn position_up() {
        assert_eq!(Bitboard::new(3,7).up(), Some(Bitboard::new(4,7)));
        assert_eq!(Bitboard::new(7,7).up(), None);
    }
}
