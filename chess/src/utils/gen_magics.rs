use simbelmyne_chess::{bitboard::Bitboard, movegen::lookups::{bishop_mask, rook_mask}, square::Square};

const BISHOP: bool = true;
const ROOK: bool = false;


fn main() {
    let bishop_magics = gen_magics::<BISHOP>();
    let rook_magics = gen_magics::<ROOK>();

    println!("Bishop magics:\n{bishop_magics:#?}");
    println!("Rook magics:\n{rook_magics:#?}");
}

////////////////////////////////////////////////////////////////////////////////
//
// Find magics
//
////////////////////////////////////////////////////////////////////////////////

/// The number of bits we expect the magic number for each square to hash into
const ROOK_KEY_WIDTH: [u32; Square::COUNT] = [
  12, 11, 11, 11, 11, 11, 11, 12,
  11, 10, 10, 10, 10, 10, 10, 11,
  11, 10, 10, 10, 10, 10, 10, 11,
  11, 10, 10, 10, 10, 10, 10, 11,
  11, 10, 10, 10, 10, 10, 10, 11,
  11, 10, 10, 10, 10, 10, 10, 11,
  11, 10, 10, 10, 10, 10, 10, 11,
  12, 11, 11, 11, 11, 11, 11, 12
];

/// The number of bits we expect the magic number for each square to hash into
const BISHOP_KEY_WIDTH: [u32; Square::COUNT] = [
  6, 5, 5, 5, 5, 5, 5, 6,
  5, 5, 5, 5, 5, 5, 5, 5,
  5, 5, 7, 7, 7, 7, 5, 5,
  5, 5, 7, 9, 9, 7, 5, 5,
  5, 5, 7, 9, 9, 7, 5, 5,
  5, 5, 7, 7, 7, 7, 5, 5,
  5, 5, 5, 5, 5, 5, 5, 5,
  6, 5, 5, 5, 5, 5, 5, 6
];


/// Generate a set of magic numbers for a slider type
pub fn gen_magics<const BISHOP: bool>() -> [MagicEntry; Square::COUNT] {
    let mut offset = 0;
    let mut magics: [MagicEntry; Square::COUNT] = [
        MagicEntry { magic: 0, mask: Bitboard::EMPTY, shift: 0, offset: 0 }; 
        Square::COUNT
    ];

    for sq in Square::ALL {
        let mask = if BISHOP { 
            bishop_mask(sq) 
        } else { 
            rook_mask(sq) 
        };


        let num_bits = if BISHOP { 
            BISHOP_KEY_WIDTH[sq] 
        } else { 
            ROOK_KEY_WIDTH[sq] 
        };

        let shift = 64 - num_bits as u8;
        let magic = find_magic(mask, num_bits);
        let entry = MagicEntry { magic, mask, shift, offset };

        magics[sq] = entry;

        offset += 1 << num_bits;
    }

    magics
}

/// Find a single magic number for the given movement mask that maps to a 
/// keyspace of `bits` bits wide.
fn find_magic(mask: Bitboard, bits: u32) -> u64 {
    let mut rng = Rng::new();

    loop  {
        let candidate = rng.rand() & rng.rand() & rng.rand();

        if is_magic(candidate, mask, bits) {
            return candidate;
        } 
    }
}

/// Check whether a supposed magic number manages to map every subset in the
/// mask to a unique index. If there's no collisions, the number's magic!
fn is_magic(candidate: u64, mask: Bitboard, bits: u32) -> bool {
    let shift = 64 - bits;
    let mut seen: [bool; 4096] = [false; 4096];
    
    if candidate.wrapping_mul(*mask) >> shift < 6 { return false };

    for subset in Subsets::from(mask) {
        let index = candidate.wrapping_mul(subset.0) >> shift;

        if seen[index as usize] {
            return false;
        } else {
            seen[index as usize] = true;
        }
    }

    true
}


////////////////////////////////////////////////////////////////////////////////
//
// Utilities
//
////////////////////////////////////////////////////////////////////////////////

/// Helper struct to iterate over all the subsets of a bitboard
pub struct Subsets { 
    subset: Bitboard,
    mask: Bitboard,
    done: bool
}

impl From<Bitboard> for Subsets {
    fn from(value: Bitboard) -> Self {
        Subsets {
            subset: Bitboard::EMPTY,
            mask: value,
            done: false
        }
    }
}

impl Iterator for Subsets {
    type Item = Bitboard;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done { return None; }

        self.subset = Bitboard(
            self.subset.0.wrapping_sub(self.mask.0) & self.mask.0
        );

        if self.subset.is_empty() {
            self.done = true;
        }

        Some(self.subset)
    }
}

/// Quick and dirty RNG
struct Rng {
    state: u128,
}

impl Rng {
    fn new() -> Self {
        use std::time;
        Self { 
            state: time::SystemTime::now()
                .duration_since(time::UNIX_EPOCH)
                .unwrap()
                .as_nanos() 
        }
    }

    fn rand(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        #[allow(clippy::cast_possible_truncation)]
        let r = x as u64; // truncation is the intended behavior here.
        r ^ (x >> 64) as u64 // add in the high bits.
    }
}

#[derive(Debug, Copy, Clone)]
pub struct MagicEntry {
    pub mask: Bitboard,
    pub magic: u64,
    pub shift: u8,
    pub offset: u32,
}


impl MagicEntry {
    pub const fn index(&self, blockers: Bitboard) -> usize {
        let blockers = blockers.0 & self.mask.0;
        let offset = self.offset as usize;
        offset + (self.magic.wrapping_mul(blockers) >> self.shift) as usize
    }
}

