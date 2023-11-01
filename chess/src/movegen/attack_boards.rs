use crate::bitboard::Bitboard;

type BBTable = [Bitboard; 64];

enum File { A, B, C, D, E, F, G, H }

impl File {
    pub const A_FILE: u64 = 0x0101010101010101;
    pub const H_FILE: u64 = 0x8080808080808080;
}

const fn gen_up_rays() -> BBTable {
    let mut bbs: BBTable = [Bitboard(0); 64];
    let mut square: usize = 0;

    while square < 64 {
        bbs[square] = Bitboard(File::A_FILE << square);
        square += 1;
    }

    bbs
}

const fn gen_down_rays() -> BBTable {
    let mut bbs: BBTable = [Bitboard(0); 64];
    let mut square: usize = 0;

    while square < 64 {
        bbs[square] = Bitboard(File::H_FILE >> (63 - square));
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
            bb |= 1 << curr;
            curr -= 1;
        }

        bb |= 1 << curr;

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
            bb |= 1 << curr;
            curr += 1;
        }

        bb |= 1 << curr;

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

        while curr % 8 < 7 && curr < 55 {
            bb |= 1 << curr;
            curr += 9;
        }

        bb |= 1 << curr;

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
        let mut bb: u64 = 1 << square;

        while curr % 8 > 0  && curr < 57 {
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
        let mut bb: u64 = 1 << square;

        while curr % 8 < 7  && curr > 7 {
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
        let mut bb: u64 = 1 << square;

        while curr % 8 > 0  && curr > 7 {
            curr -= 9;
            bb |= 1 << curr;
        }

        bbs[square] = Bitboard(bb as u64);
        square += 1;
    }

    bbs
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Direction { Up, Down, Left, Right, UpLeft, UpRight, DownLeft, DownRight}
use Direction::*;

impl Direction {
    pub const ALL: [Direction; 8] = [
         Up, Down, Left, Right, UpLeft, UpRight, DownLeft, DownRight
    ];

    pub const BISHOP: [Direction; 4] = [
         UpLeft, UpRight, DownLeft, DownRight
    ];

    pub const ROOK: [Direction; 4] = [
        Up, Down, Left, Right,
    ];

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

pub const DIAG_RAYS: [BBTable; 4] = [
    UP_RIGHT_RAYS,
    UP_LEFT_RAYS,
    DOWN_RIGHT_RAYS,
    DOWN_LEFT_RAYS,
];

pub const HV_RAYS: [BBTable; 4] = [
    UP_RAYS,
    DOWN_RAYS,
    LEFT_RAYS,
    RIGHT_RAYS,
];

pub const BISHOP_RAYS: [BBTable; 4] = DIAG_RAYS;
pub const ROOK_RAYS: [BBTable; 4] = HV_RAYS;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_up_ray_a1() {
        assert_eq!(UP_RAYS[0], Bitboard(0x101010101010101), "Gets the a file for A1");
    }

    #[test]
    fn test_up_ray_a3() {
        assert_eq!(UP_RAYS[2], Bitboard(0x404040404040404), "Gets the correct up-ray for A3");
    }

    #[test]
    fn test_up_ray_c3() {
        assert_eq!(UP_RAYS[18], Bitboard(0x404040404040000), "Gets the correct up-ray for C3");
    }

    #[test]
    fn test_left_ray_c3() {
        assert_eq!(LEFT_RAYS[18], Bitboard(0x70000), "Gets the correct left-ray for C3");
    }

    #[test]
    fn test_right_ray_d4() {
        assert_eq!(RIGHT_RAYS[27], Bitboard(0xf8000000), "Gets the correct right-ray for D4");
    }

    #[test]
    fn test_up_right_ray_d4() {
        assert_eq!(UP_RIGHT_RAYS[27], Bitboard(0x8040201008000000), "Gets the correct up-right-ray for D4");
    }

    #[test]
    fn test_up_left_ray_d4() {
        assert_eq!(UP_LEFT_RAYS[27], Bitboard(0x1020408000000), "Gets the correct up-left-ray for D4");
    }

    #[test]
    fn test_down_right_ray_d4() {
        assert_eq!(DOWN_RIGHT_RAYS[27], Bitboard(0x8102040), "Gets the correct down-right-ray for D4");
    }

    #[test]
    fn test_down_left_ray_d4() {
        assert_eq!(DOWN_LEFT_RAYS[27], Bitboard(0x8040201), "Gets the correct down-left-ray for D4");
    }
}
