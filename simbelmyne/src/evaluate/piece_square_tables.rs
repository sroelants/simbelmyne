use chess::square::Square;
use chess::piece::PieceType;

use super::S;
use super::params::BISHOP_PSQT;
use super::params::KING_PSQT;
use super::params::KNIGHT_PSQT;
use super::params::PAWN_PSQT;
use super::params::QUEEN_PSQT;
use super::params::ROOK_PSQT;

pub const PIECE_SQUARE_TABLES: [[S; Square::COUNT]; PieceType::COUNT] = [
    PAWN_PSQT, 
    KNIGHT_PSQT, 
    BISHOP_PSQT, 
    ROOK_PSQT, 
    QUEEN_PSQT, 
    KING_PSQT,
];
