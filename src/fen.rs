use crate::board::{Color, PieceType};
use crate::parse;
use anyhow::anyhow;

pub struct FEN {
  pub ranks: Vec<Vec<FENAtom>>
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum FENAtom {
    Piece(Color, PieceType),
    Gap(usize)
}

impl Into<FENAtom> for (Color, PieceType) {
    fn into(self: Self) -> FENAtom {
        FENAtom::Piece(self.0, self.1)
    }
}

impl Into<FENAtom> for usize {
    fn into(self: Self) -> FENAtom {
        FENAtom::Gap(self)
    }
}

impl TryFrom<&str> for FEN {
    type Error = anyhow::Error;

    fn try_from(input: &str) -> anyhow::Result<FEN> {
        let result = parse::fen_board(input);

        match result {
            Ok((_, ranks)) => Ok(FEN { ranks }),
            Err(_) => Err(anyhow!("Failed to Parse"))
        }
    }
}
