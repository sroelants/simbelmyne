use crate::bitboard::Bitboard;

type BBTable = [Bitboard; 64];

#[allow(dead_code)]
enum File { A, B, C, D, E, F, G, H }

#[allow(dead_code)]
impl File {
    pub const A_FILE: u64 = 0x0101010101010101;
    pub const B_FILE: u64 = File::A_FILE << 1;
    pub const C_FILE: u64 = File::A_FILE << 2;
    pub const D_FILE: u64 = File::A_FILE << 3;
    pub const E_FILE: u64 = File::A_FILE << 4;
    pub const F_FILE: u64 = File::A_FILE << 5;
    pub const G_FILE: u64 = File::A_FILE << 6;
    pub const H_FILE: u64 = File::A_FILE << 7;
    pub const ALL: [Bitboard; 8] = [
        Bitboard(File::A_FILE),
        Bitboard(File::B_FILE),
        Bitboard(File::C_FILE),
        Bitboard(File::D_FILE),
        Bitboard(File::E_FILE),
        Bitboard(File::F_FILE),
        Bitboard(File::G_FILE),
        Bitboard(File::H_FILE),
    ];
}

pub enum Rank { First, Second, Third, Fourth, Fifth, Sixth, Seventh, Eighth }

impl Rank {
    pub const FIRST_RANK: u64 = 0xff;
    pub const SECOND_RANK: u64 = Rank::FIRST_RANK << 8;
    pub const THIRD_RANK: u64 = Rank::SECOND_RANK << 8;
    pub const FOURTH_RANK: u64 = Rank::THIRD_RANK << 8;
    pub const FIFTH_RANK: u64 = Rank::FOURTH_RANK << 8;
    pub const SIXTH_RANK: u64 = Rank::FIFTH_RANK << 8;
    pub const SEVENTH_RANK: u64 = Rank::SIXTH_RANK << 8;
    pub const EIGHTH_RANK: u64 = Rank::SEVENTH_RANK << 8;

    pub const ALL: [Bitboard; 8] = [
        Bitboard(Rank::FIRST_RANK),
        Bitboard(Rank::SECOND_RANK),
        Bitboard(Rank::THIRD_RANK),
        Bitboard(Rank::FOURTH_RANK),
        Bitboard(Rank::FIFTH_RANK),
        Bitboard(Rank::SIXTH_RANK),
        Bitboard(Rank::SEVENTH_RANK),
        Bitboard(Rank::EIGHTH_RANK),
    ];

    pub const W_PROMO_RANK: Bitboard = Bitboard(Rank::EIGHTH_RANK);
    pub const B_PROMO_RANK: Bitboard = Bitboard(Rank::FIRST_RANK);
}

////////////////////////////////////////////////////////////////////////////////
//
// Generate rays
//
////////////////////////////////////////////////////////////////////////////////

const fn gen_up_rays() -> BBTable {
    let mut bbs: BBTable = [Bitboard(0); 64];
    let mut square: usize = 0;

    while square < 64 {
        let rank = square / 8;

        if rank < 7 {
            bbs[square] = Bitboard(File::A_FILE << square + 8);
        }

        square += 1;
    }

    bbs
}

const fn gen_down_rays() -> BBTable {
    let mut bbs: BBTable = [Bitboard(0); 64];
    let mut square: usize = 0;

    while square < 64 {
        let rank = square / 8;

        if rank > 0 {
            bbs[square] = Bitboard(File::H_FILE >> (63 - (square - 8)));
        }

        square += 1;
    }

    bbs
}

const fn gen_left_rays() -> BBTable {
    let mut bbs: BBTable = [Bitboard(0); 64];
    let mut square: usize = 0;

    while square < 64 {
        let mut curr = square;
        let mut bb: u64 = 0;

        while curr % 8 > 0 {
            curr -= 1;
            bb |= 1 << curr;
        }

        bbs[square] = Bitboard(bb as u64);
        square += 1;
    }

    bbs
}

const fn gen_right_rays() -> BBTable {
    let mut bbs: BBTable = [Bitboard(0); 64];
    let mut square: usize = 0;

    while square < 64 {
        let mut bb: u64 = 0;
        let mut curr = square;

        while curr % 8 < 7 {
            curr += 1;
            bb |= 1 << curr;
        }

        bbs[square] = Bitboard(bb as u64);
        square += 1;
    }

    bbs
}

const fn gen_up_right_rays() -> BBTable {
    let mut bbs: BBTable = [Bitboard(0); 64];
    let mut square: usize = 0;

    while square < 64 {
        let mut curr = square;
        let mut bb: u64 = 0;

        while curr % 8 < 7 && curr / 8 < 7 {
            curr += 9;
            bb |= 1 << curr;
        }

        bbs[square] = Bitboard(bb as u64);
        square += 1;
    }

    bbs
}

const fn gen_up_left_rays() -> BBTable {
    let mut bbs: BBTable = [Bitboard(0); 64];
    let mut square: usize = 0;

    while square < 64 {
        let mut curr = square;
        let mut bb: u64 = 0;

        while curr % 8 > 0 && curr / 8 < 7 {
            curr += 7;
            bb |= 1 << curr;
        }

        bbs[square] = Bitboard(bb as u64);
        square += 1;
    }

    bbs
}

const fn gen_down_right_rays() -> BBTable {
    let mut bbs: BBTable = [Bitboard(0); 64];
    let mut square: usize = 0;

    while square < 64 {
        let mut curr = square;
        let mut bb: u64 = 0;

        while curr % 8 < 7 && curr > 7 {
            curr -= 7;
            bb |= 1 << curr;
        }

        bbs[square] = Bitboard(bb as u64);
        square += 1;
    }

    bbs
}

const fn gen_down_left_rays() -> BBTable {
    let mut bbs: BBTable = [Bitboard(0); 64];
    let mut square: usize = 0;

    while square < 64 {
        let mut curr = square;
        let mut bb: u64 = 0;

        while curr % 8 > 0 && curr > 7 {
            curr -= 9;
            bb |= 1 << curr;
        }

        bbs[square] = Bitboard(bb as u64);
        square += 1;
    }

    bbs
}

////////////////////////////////////////////////////////////////////////////////
//
// Generate attack boards
//
////////////////////////////////////////////////////////////////////////////////

const fn gen_bishop_attacks() -> BBTable {
    let mut bbs: BBTable = [Bitboard(0); 64];
    let mut square: usize = 0;

    while square < 64 {
        bbs[square] = Bitboard(
            UP_LEFT_RAYS[square].0
                | UP_RIGHT_RAYS[square].0
                | DOWN_LEFT_RAYS[square].0
                | DOWN_RIGHT_RAYS[square].0,
        );

        square += 1;
    }

    bbs
}

const fn gen_rook_attacks() -> BBTable {
    let mut bbs: BBTable = [Bitboard(0); 64];
    let mut square: usize = 0;

    while square < 64 {
        bbs[square] = Bitboard(
            UP_RAYS[square].0 | DOWN_RAYS[square].0 | LEFT_RAYS[square].0 | RIGHT_RAYS[square].0,
        );

        square += 1;
    }

    bbs
}

const fn gen_queen_attacks() -> BBTable {
    let mut bbs: BBTable = [Bitboard(0); 64];
    let mut square: usize = 0;

    while square < 64 {
        bbs[square] = Bitboard(BISHOP_ATTACKS[square].0 | ROOK_ATTACKS[square].0);

        square += 1;
    }

    bbs
}

const fn gen_pawn_pushes<const WHITE: bool, const DOUBLE_PUSH: bool>() -> BBTable {
    let mut bbs: BBTable = [Bitboard(0); 64];
    let mut square: usize = 0;

    while square < 64 {
        let rank = square / 8;
        let mut bitboard: u64 = 0;

        if WHITE {
            if rank < 7 {
                let up = square + 8;
                bitboard |= 1 << up
            }
            if DOUBLE_PUSH && rank < 6 {
                let upup = square + 16;
                bitboard |= 1 << upup
            }
        } else {
            if rank > 0 {
                let down = square - 8;
                bitboard |= 1 << down
            }

            if DOUBLE_PUSH && rank > 1 {
                let downdown = square - 16;
                bitboard |= 1 << downdown
            }
        }

        bbs[square] = Bitboard(bitboard);
        square += 1
    }

    bbs
}

const fn gen_pawn_attacks<const WHITE: bool>() -> BBTable {
    let mut bbs: BBTable = [Bitboard(0); 64];
    let mut square: usize = 0;

    while square < 64 {
        let file = square % 8;
        let rank = square / 8;
        let mut bitboard: u64 = 0;

        if WHITE {
            if file > 0 && rank < 7 {
                let up_left = square + 7;
                bitboard |= 1 << up_left
            }
            if file < 7 && rank < 7 {
                let up_right = square + 9;
                bitboard |= 1 << up_right;
            }
        } else {
            // BLACK
            if file > 0 && rank > 0 {
                let down_left = square - 9;
                bitboard |= 1 << down_left;
            }
            if file < 7 && rank > 0 {
                let down_right = square - 7;
                bitboard |= 1 << down_right
            }
        }
        bbs[square] = Bitboard(bitboard);
        square += 1
    }

    bbs
}

const fn gen_king_attacks() -> BBTable {
    let mut bbs: BBTable = [Bitboard(0); 64];
    let mut square: usize = 0;

    while square < 64 {
        let file = square % 8;
        let rank = square / 8;
        let mut bitboard: u64 = 0;

        if file > 0 {
            let left = square - 1;
            bitboard |= 1 << left;

            if rank > 0 {
                let downleft = square - 9;
                bitboard |= 1 << downleft;
            }

            if rank < 7 {
                let upleft = square + 7;
                bitboard |= 1 << upleft;
            }
        }

        if file < 7 {
            let right = square + 1;
            bitboard |= 1 << right;

            if rank < 7 {
                let upright = square + 9;
                bitboard |= 1 << upright;
            }

            if rank > 0 {
                let downright = square - 7;
                bitboard |= 1 << downright;
            }
        }

        if rank > 0 {
            let down = square - 8;
            bitboard |= 1 << down;
        }

        if rank < 7 {
            let up = square + 8;
            bitboard |= 1 << up;
        }

        bbs[square] = Bitboard(bitboard);
        square += 1
    }

    bbs
}

const fn gen_knight_attacks() -> BBTable {
    let mut bbs: BBTable = [Bitboard(0); 64];
    let mut square: usize = 0;

    while square < 64 {
        let file = square % 8;
        let rank = square / 8;
        let mut bitboard: u64 = 0;

        if file > 1 && rank < 7 {
            let leftleftup = square + 6;
            bitboard |= 1 << leftleftup;
        }

        if file > 0 && rank < 6 {
            let upupleft = square + 15;
            bitboard |= 1 << upupleft;
        }

        if file > 1 && rank > 0 {
            let leftleftdown = square - 10;
            bitboard |= 1 << leftleftdown;
        }

        if file > 0 && rank > 1 {
            let downdownleft = square - 17;
            bitboard |= 1 << downdownleft;
        }

        if file < 6 && rank < 7 {
            let rightrightup = square + 10;
            bitboard |= 1 << rightrightup;
        }

        if file < 7 && rank < 6 {
            let upupright = square + 17;
            bitboard |= 1 << upupright;
        }

        if file < 6 && rank > 0 {
            let rightrightdown = square - 6;
            bitboard |= 1 << rightrightdown;
        }

        if file < 7 && rank > 1 {
            let downdownright = square - 15;
            bitboard |= 1 << downdownright;
        }

        bbs[square] = Bitboard(bitboard);
        square += 1
    }

    bbs
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
    UpLeft,
    UpRight,
    DownLeft,
    DownRight,
}
use Direction::*;

impl Direction {
    pub const ALL: [Direction; 8] = [Up, Down, Left, Right, UpLeft, UpRight, DownLeft, DownRight];

    pub const BISHOP: [Direction; 4] = [UpLeft, UpRight, DownLeft, DownRight];

    pub const ROOK: [Direction; 4] = [Up, Down, Left, Right];

    pub fn is_positive(&self) -> bool {
        match self {
            UpLeft | Up | UpRight | Right => true,
            _ => false,
        }
    }
}

pub const UP_RAYS: BBTable = gen_up_rays();
pub const DOWN_RAYS: BBTable = gen_down_rays();
pub const LEFT_RAYS: BBTable = gen_left_rays();
pub const RIGHT_RAYS: BBTable = gen_right_rays();
pub const UP_RIGHT_RAYS: BBTable = gen_up_right_rays();
pub const UP_LEFT_RAYS: BBTable = gen_up_left_rays();
pub const DOWN_RIGHT_RAYS: BBTable = gen_down_right_rays();
pub const DOWN_LEFT_RAYS: BBTable = gen_down_left_rays();

pub const ATTACK_RAYS: [BBTable; 8] = [
    UP_RAYS,
    DOWN_RAYS,
    LEFT_RAYS,
    RIGHT_RAYS,
    UP_LEFT_RAYS,
    UP_RIGHT_RAYS,
    DOWN_LEFT_RAYS,
    DOWN_RIGHT_RAYS,
];

pub const DIAG_RAYS: [BBTable; 4] = [UP_RIGHT_RAYS, UP_LEFT_RAYS, DOWN_RIGHT_RAYS, DOWN_LEFT_RAYS];

pub const HV_RAYS: [BBTable; 4] = [UP_RAYS, DOWN_RAYS, LEFT_RAYS, RIGHT_RAYS];

const WHITE: bool = true;
const BLACK: bool = false;
const DOUBLE_PUSH: bool = true;
const SINGLE_PUSH: bool = false;

pub const W_PAWN_PUSHES: BBTable = gen_pawn_pushes::<WHITE, SINGLE_PUSH>();
pub const B_PAWN_PUSHES: BBTable = gen_pawn_pushes::<BLACK, SINGLE_PUSH>();
pub const W_PAWN_DPUSHES: BBTable = gen_pawn_pushes::<WHITE, DOUBLE_PUSH>();
pub const B_PAWN_DPUSHES: BBTable = gen_pawn_pushes::<BLACK, DOUBLE_PUSH>();
pub const W_PAWN_ATTACKS: BBTable = gen_pawn_attacks::<WHITE>();
pub const B_PAWN_ATTACKS: BBTable = gen_pawn_attacks::<BLACK>();
pub const KNIGHT_ATTACKS: BBTable = gen_knight_attacks();
pub const KING_ATTACKS: BBTable = gen_king_attacks();

pub const BISHOP_ATTACKS: BBTable = gen_bishop_attacks();
pub const ROOK_ATTACKS: BBTable = gen_rook_attacks();
pub const QUEEN_ATTACKS: BBTable = gen_queen_attacks();

#[cfg(test)]
mod tests {
    use crate::board::Square;

    use super::*;

    #[test]
    fn test_up_ray_a1() {
        assert_eq!(
            UP_RAYS[Square::A1 as usize],
            Bitboard(0x101010101010100),
            "Gets the a file for A1"
        );
    }

    #[test]
    fn test_up_ray_a3() {
        assert_eq!(
            UP_RAYS[Square::A3 as usize],
            Bitboard(0x101010101000000),
            "Gets the correct up-ray for A3"
        );
    }

    #[test]
    fn test_up_ray_c3() {
        assert_eq!(
            UP_RAYS[Square::C3 as usize],
            Bitboard(0x404040404000000),
            "Gets the correct up-ray for C3"
        );
    }

    #[test]
    fn test_left_ray_c3() {
        assert_eq!(
            LEFT_RAYS[Square::C3 as usize],
            Bitboard(0x30000),
            "Gets the correct left-ray for C3"
        );
    }

    #[test]
    fn test_right_ray_d4() {
        assert_eq!(
            RIGHT_RAYS[Square::D4 as usize],
            Bitboard(0xf0000000),
            "Gets the correct right-ray for D4"
        );
    }

    #[test]
    fn test_up_right_ray_d4() {
        assert_eq!(
            UP_RIGHT_RAYS[Square::D4 as usize],
            Bitboard(0x8040201000000000),
            "Gets the correct up-right-ray for D4"
        );
    }

    #[test]
    fn test_up_left_ray_d4() {
        assert_eq!(
            UP_LEFT_RAYS[Square::D4 as usize],
            Bitboard(0x1020400000000),
            "Gets the correct up-left-ray for D4"
        );
    }

    #[test]
    fn test_down_right_ray_d4() {
        assert_eq!(
            DOWN_RIGHT_RAYS[Square::D4 as usize],
            Bitboard(0x102040),
            "Gets the correct down-right-ray for D4"
        );
    }

    #[test]
    fn test_down_left_ray_d4() {
        assert_eq!(
            DOWN_LEFT_RAYS[Square::D4 as usize],
            Bitboard(0x40201),
            "Gets the correct down-left-ray for D4"
        );
    }

    #[test]
    fn test_pawn_pushes() {
        assert_eq!(
            W_PAWN_PUSHES[Square::new(4, 4) as usize],
            Bitboard(0x100000000000)
        );
        assert_eq!(
            W_PAWN_PUSHES[Square::new(7, 4) as usize],
            Bitboard(0x000000000000)
        );

        assert_eq!(
            B_PAWN_PUSHES[Square::new(4, 4) as usize],
            Bitboard(0x10000000)
        );
        assert_eq!(
            B_PAWN_PUSHES[Square::new(0, 4) as usize],
            Bitboard(0x000000000000)
        );

        // Double pushes
        assert_eq!(
            W_PAWN_DPUSHES[Square::new(4, 4) as usize],
            Bitboard(0x10100000000000)
        );
        assert_eq!(
            W_PAWN_DPUSHES[Square::new(6, 4) as usize],
            Bitboard(0x1000000000000000)
        );

        assert_eq!(
            B_PAWN_DPUSHES[Square::new(4, 4) as usize],
            Bitboard(0x10100000)
        );
        assert_eq!(B_PAWN_DPUSHES[Square::new(1, 4) as usize], Bitboard(0x10));
    }

    #[test]
    fn test_pawn_attacks() {
        assert_eq!(
            W_PAWN_ATTACKS[Square::new(4, 4) as usize],
            Bitboard(0x280000000000)
        );
        assert_eq!(
            W_PAWN_ATTACKS[Square::new(4, 0) as usize],
            Bitboard(0x20000000000)
        );
        assert_eq!(
            W_PAWN_ATTACKS[Square::new(4, 7) as usize],
            Bitboard(0x400000000000)
        );
        assert_eq!(W_PAWN_ATTACKS[Square::new(7, 4) as usize], Bitboard(0x00));

        assert_eq!(
            B_PAWN_ATTACKS[Square::new(4, 4) as usize],
            Bitboard(0x28000000)
        );
        assert_eq!(
            B_PAWN_ATTACKS[Square::new(4, 0) as usize],
            Bitboard(0x2000000)
        );
        assert_eq!(
            B_PAWN_ATTACKS[Square::new(4, 7) as usize],
            Bitboard(0x40000000)
        );
        assert_eq!(B_PAWN_ATTACKS[Square::new(0, 4) as usize], Bitboard(0x00));
    }

    #[test]
    fn test_knight_attacks() {
        assert_eq!(
            KNIGHT_ATTACKS[Square::new(4, 4) as usize],
            Bitboard(0x28440044280000)
        );
        assert_eq!(
            KNIGHT_ATTACKS[Square::new(6, 1) as usize],
            Bitboard(0x800080500000000)
        );
        assert_eq!(
            KNIGHT_ATTACKS[Square::new(1, 6) as usize],
            Bitboard(0xa0100010)
        );
    }

    #[test]
    fn test_king_attacks() {
        assert_eq!(
            KING_ATTACKS[Square::new(4, 4) as usize],
            Bitboard(0x382838000000)
        );
        assert_eq!(
            KING_ATTACKS[Square::new(7, 0) as usize],
            Bitboard(0x203000000000000)
        );
    }

    #[test]
    fn test_bishop_attacks() {
        assert_eq!(
            BISHOP_ATTACKS[Square::E5 as usize],
            Bitboard(0x8244280028448201)
        );
    }

    #[test]
    fn test_rook_attacks() {
        assert_eq!(
            ROOK_ATTACKS[Square::E5 as usize],
            Bitboard(0x101010ef10101010)
        );
    }

    #[test]
    fn test_queen_attacks() {
        assert_eq!(
            QUEEN_ATTACKS[Square::E5 as usize],
            Bitboard(0x925438ef38549211)
        );
    }
}
