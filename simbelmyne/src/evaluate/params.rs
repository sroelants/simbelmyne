pub const MG_PIECE_VALUES: [i32; 6] = [81, 336, 365, 477, 1025, 0];

pub const MG_PAWN_PSQT: [i32; 64] = [0, 0, 0, 0, 0, 0, 0, 0, 97, 133, 60, 94, 67, 125, 33, -11, -6, 6, 25, 30, 64, 55, 25, -19, -14, 12, 5, 20, 23, 12, 17, -22, -26, -2, -5, 11, 16, 6, 10, -24, -26, -4, -3, -9, 2, 3, 32, -12, -34, -1, -19, -23, -14, 23, 37, -21, 0, 0, 0, 0, 0, 0, 0, 0];

pub const MG_KNIGHT_PSQT: [i32; 64] = [-166, -89, -34, -48, 60, -96, -15, -106, -72, -40, 71, 36, 23, 61, 7, -16, -46, 60, 37, 65, 84, 128, 72, 44, -8, 16, 19, 53, 37, 69, 18, 22, -12, 3, 16, 13, 27, 19, 21, -7, -23, -9, 12, 9, 19, 16, 25, -16, -29, -52, -11, -3, -1, 18, -14, -18, -105, -21, -57, -32, -17, -27, -18, -23];

pub const MG_BISHOP_PSQT: [i32; 64] = [-28, 4, -81, -37, -25, -41, 7, -7, -25, 16, -18, -12, 30, 58, 18, -47, -15, 36, 43, 40, 35, 49, 37, -1, -4, 5, 19, 50, 37, 36, 7, -1, -6, 12, 12, 26, 34, 12, 10, 4, 0, 14, 14, 15, 14, 26, 17, 10, 4, 14, 16, 0, 6, 21, 32, 0, -32, -2, -13, -20, -12, -11, -38, -20];

pub const MG_ROOK_PSQT: [i32; 64] = [31, 42, 32, 50, 62, 9, 30, 43, 26, 32, 57, 61, 79, 66, 26, 44, -4, 19, 25, 36, 17, 44, 61, 16, -23, -10, 6, 26, 23, 34, -7, -19, -35, -25, -11, -1, 9, -7, 6, -22, -44, -24, -16, -17, 2, 0, -4, -32, -43, -16, -20, -9, -1, 11, -5, -70, -18, -12, 0, 16, 15, 7, -36, -25];

pub const MG_QUEEN_PSQT: [i32; 64] = [-28, 0, 28, 12, 58, 44, 42, 44, -23, -38, -4, 1, -15, 56, 27, 54, -12, -16, 6, 8, 29, 56, 47, 57, -27, -26, -16, -15, -1, 17, -1, 1, -9, -25, -9, -10, -2, -3, 3, -2, -14, 1, -11, -2, -4, 2, 14, 5, -34, -7, 10, 2, 7, 14, -3, 0, 0, -17, -9, 9, -14, -24, -30, -49];

pub const MG_KING_PSQT: [i32; 64] = [-65, 23, 15, -15, -56, -34, 2, 13, 28, -1, -20, -6, -8, -3, -37, -29, -9, 23, 2, -15, -19, 6, 21, -21, -16, -20, -12, -27, -29, -24, -13, -36, -49, 0, -27, -38, -45, -43, -32, -50, -14, -14, -22, -45, -43, -30, -15, -26, 1, 7, -8, -63, -42, -16, 9, 8, -14, 36, 12, -54, 7, -28, 23, 14];

pub const MG_PASSED_PAWN_TABLE: [i32; 64] = [0, 0, 0, 0, 0, 0, 0, 0, 44, 51, 41, 42, 27, 33, 18, 8, 47, 42, 42, 29, 23, 30, 12, 2, 27, 16, 12, 9, 10, 19, 5, 1, 13, 0, -8, -6, -13, -6, 8, 15, 4, 2, -3, -13, -3, 10, 12, 18, 7, 8, 1, -8, -2, 7, 15, 9, 0, 0, 0, 0, 0, 0, 0, 0];

pub const MG_ISOLATED_PAWN_PENALTY: i32 = -16;

pub const MG_DOUBLED_PAWN_PENALTY: i32 = -9;

pub const MG_BISHOP_PAIR_BONUS: i32 = 0;

pub const MG_ROOK_OPEN_FILE_BONUS: i32 = 49;

pub const MG_ROOK_SEMIOPEN_FILE_BONUS: i32 = 0;

pub const EG_PIECE_VALUES: [i32; 6] = [93, 280, 297, 512, 936, 0];

pub const EG_PAWN_PSQT: [i32; 64] = [0, 0, 0, 0, 0, 0, 0, 0, 177, 172, 157, 133, 146, 131, 164, 186, 93, 99, 84, 66, 55, 52, 82, 84, 31, 23, 12, 4, -1, 4, 17, 17, 13, 8, -3, -7, -7, -7, 3, 0, 3, 6, -5, 1, 0, -4, -1, -8, 13, 7, 8, 9, 13, 0, 1, -6, 0, 0, 0, 0, 0, 0, 0, 0];

pub const EG_KNIGHT_PSQT: [i32; 64] = [-57, -38, -13, -27, -31, -26, -63, -98, -24, -7, -25, -1, -8, -25, -23, -51, -23, -19, 10, 9, 0, -9, -19, -40, -16, 2, 22, 22, 22, 11, 8, -17, -17, -6, 16, 25, 15, 17, 4, -17, -23, -3, 0, 14, 10, -3, -19, -22, -42, -19, -9, -5, -2, -19, -23, -43, -29, -51, -22, -14, -22, -17, -49, -64];

pub const EG_BISHOP_PSQT: [i32; 64] = [-13, -20, -10, -8, -7, -8, -16, -23, -7, -3, 6, -11, -2, -13, -3, -14, 2, -8, 0, 0, -1, 5, 0, 4, -3, 9, 12, 9, 14, 9, 3, 2, -6, 2, 12, 19, 7, 10, -2, -8, -11, -3, 7, 10, 13, 2, -7, -14, -13, -18, -6, -1, 3, -8, -15, -27, -22, -8, -22, -4, -8, -15, -4, -16];

pub const EG_ROOK_PSQT: [i32; 64] = [12, 10, 18, 14, 11, 12, 7, 5, 10, 13, 12, 10, -3, 2, 8, 3, 7, 7, 6, 5, 4, -3, -4, -2, 4, 3, 12, 1, 1, 0, 0, 2, 3, 5, 8, 3, -4, -6, -7, -10, -3, 0, -5, -1, -7, -12, -7, -15, -5, -6, 0, 1, -9, -8, -10, -2, -8, 2, 2, -1, -5, -12, 4, -19];

pub const EG_QUEEN_PSQT: [i32; 64] = [-9, 21, 21, 27, 26, 19, 9, 19, -16, 20, 32, 41, 58, 24, 29, 0, -19, 6, 8, 49, 47, 35, 19, 9, 2, 22, 23, 45, 56, 40, 57, 36, -18, 28, 18, 46, 30, 34, 39, 23, -16, -27, 14, 5, 9, 17, 10, 5, -21, -22, -30, -15, -16, -23, -36, -32, -32, -27, -22, -43, -4, -31, -19, -40];

pub const EG_KING_PSQT: [i32; 64] = [-74, -34, -18, -18, -11, 14, 4, -16, -12, 16, 13, 17, 16, 38, 23, 10, 9, 16, 23, 15, 20, 45, 43, 13, -7, 21, 23, 26, 26, 33, 26, 2, -18, -3, 20, 24, 27, 23, 9, -10, -19, -3, 10, 21, 23, 15, 6, -8, -26, -10, 3, 13, 14, 3, -4, -16, -52, -33, -20, -11, -28, -14, -24, -42];

pub const EG_PASSED_PAWN_TABLE: [i32; 64] = [0, 0, 0, 0, 0, 0, 0, 0, 76, 73, 62, 52, 58, 59, 71, 76, 90, 82, 65, 39, 29, 60, 67, 84, 54, 51, 41, 34, 30, 34, 55, 52, 28, 26, 21, 18, 16, 19, 33, 29, 7, 5, 4, 1, 0, 0, 13, 6, 1, 2, -4, 0, -1, -1, 6, 6, 0, 0, 0, 0, 0, 0, 0, 0];

pub const EG_ISOLATED_PAWN_PENALTY: i32 = -6;

pub const EG_DOUBLED_PAWN_PENALTY: i32 = -19;

pub const EG_BISHOP_PAIR_BONUS: i32 = 0;

pub const EG_ROOK_OPEN_FILE_BONUS: i32 = 0;

pub const EG_ROOK_SEMIOPEN_FILE_BONUS: i32 = 0;
