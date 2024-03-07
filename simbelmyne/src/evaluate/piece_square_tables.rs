use chess::square::Square;
use chess::piece::PieceType;
use crate::evaluate::Eval;

use super::new_eval::S;
use super::new_params::BISHOP_PSQT;
use super::new_params::KING_PSQT;
use super::new_params::KNIGHT_PSQT;
use super::new_params::PAWN_PSQT;
use super::new_params::QUEEN_PSQT;
use super::new_params::ROOK_PSQT;
use super::params::EG_KING_PSQT;
use super::params::EG_QUEEN_PSQT;
use super::params::EG_ROOK_PSQT;
use super::params::EG_BISHOP_PSQT;
use super::params::EG_KNIGHT_PSQT;
use super::params::EG_PAWN_PSQT;
use super::params::MG_KING_PSQT;
use super::params::MG_QUEEN_PSQT;
use super::params::MG_ROOK_PSQT;
use super::params::MG_BISHOP_PSQT;
use super::params::MG_KNIGHT_PSQT;
use super::params::MG_PAWN_PSQT;

pub const MG_TABLES: [[Eval; Square::COUNT]; PieceType::COUNT] = [
    MG_PAWN_PSQT, 
    MG_KNIGHT_PSQT, 
    MG_BISHOP_PSQT, 
    MG_ROOK_PSQT, 
    MG_QUEEN_PSQT, 
    MG_KING_PSQT,
];

pub const EG_TABLES: [[Eval; Square::COUNT]; PieceType::COUNT] = [
    EG_PAWN_PSQT, 
    EG_KNIGHT_PSQT, 
    EG_BISHOP_PSQT, 
    EG_ROOK_PSQT, 
    EG_QUEEN_PSQT, 
    EG_KING_PSQT,
];

pub const PIECE_SQUARE_TABLES: [[S; Square::COUNT]; PieceType::COUNT] = [
    PAWN_PSQT, 
    KNIGHT_PSQT, 
    BISHOP_PSQT, 
    ROOK_PSQT, 
    QUEEN_PSQT, 
    KING_PSQT,
];
