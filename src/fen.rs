use crate::board::{Color, PieceType};
use crate::parse;

pub struct FEN {
  pub ranks: Vec<Vec<FENAtom>>
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum FENAtom {
    Piece(Color, PieceType),
    Gap(u64)
}

impl Into<FENAtom> for (Color, PieceType) {
    fn into(self: Self) -> FENAtom {
        FENAtom::Piece(self.0, self.1)
    }
}

impl Into<FENAtom> for u64 {
    fn into(self: Self) -> FENAtom {
        FENAtom::Gap(self)
    }
}

impl TryFrom<&str> for FEN {
    type Error = &'static str;

    fn try_from(input: &str) -> Result<FEN, &'static str> {
        let result = parse::fen_board(input);

        match result {
            Ok((_, ranks)) => Ok(FEN { ranks }),
            Err(_) => Err("Failed to Parse")
        }
    }
}
