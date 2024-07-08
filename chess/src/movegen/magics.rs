use crate::{square::Square, bitboard::Bitboard};
use super::lookups::{gen_bishop_attacks, gen_rook_attacks};

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

////////////////////////////////////////////////////////////////////////////////
//
// Square method impls
//
////////////////////////////////////////////////////////////////////////////////

impl Square {
    /// Get a bitboard for all the squares visible to a bishop on this square.
    pub fn bishop_squares(self, blockers: Bitboard) -> Bitboard {
        let magic = BISHOP_MAGICS[self];
        let idx = magic.index(blockers);

        BISHOP_ATTACKS[idx]
    }

    /// Get a bitboard for all the squares visible to a rook on this square.
    pub fn rook_squares(self, blockers: Bitboard) -> Bitboard {
        let magic = ROOK_MAGICS[self];
        let idx = magic.index(blockers);

        ROOK_ATTACKS[idx]
    }
}

////////////////////////////////////////////////////////////////////////////////
//
// Attack table generation
//
////////////////////////////////////////////////////////////////////////////////


pub const BISHOP_ATTACKS: [Bitboard; 5248] = gen_bishop_attacks_table();
pub const ROOK_ATTACKS: [Bitboard; 102400] = gen_rook_attacks_table();

const fn gen_bishop_attacks_table() -> [Bitboard; 5248]  {
    let mut table = [Bitboard::EMPTY; 5248];
    let mut sq: usize = 0;

    while sq < 64 {
        let entry = BISHOP_MAGICS[sq];
        let mut subset: u64 = 0;

        // First treat the empty subset 
        let attacks = gen_bishop_attacks(Square::ALL[sq], Bitboard(subset));
        let blockers = Bitboard(subset);
        let idx = entry.index(blockers);
        table[idx] = attacks;
        subset = subset.wrapping_sub(entry.mask.0) & entry.mask.0;

        // For every subset of possible blockers, get the attacked squares and
        // store them in the table.
        while subset != 0 {
            let attacks = gen_bishop_attacks(Square::ALL[sq], Bitboard(subset));
            let blockers = Bitboard(subset);
            let idx = entry.index(blockers);
            table[idx] = attacks;

            subset = subset.wrapping_sub(entry.mask.0) & entry.mask.0;
        }

        sq += 1;
    }

    table
}


const fn gen_rook_attacks_table() -> [Bitboard; 102400] {
    let mut table = [Bitboard::EMPTY; 102400];
    let mut sq: usize = 0;

    while sq < 64 {
        let entry = ROOK_MAGICS[sq];
        let mut subset: u64 = 0;

        // First treat the empty subset 
        let attacks = gen_rook_attacks(Square::ALL[sq], Bitboard(subset));
        let blockers = Bitboard(subset);
        let idx = entry.index(blockers);
        table[idx] = attacks;
        subset = subset.wrapping_sub(entry.mask.0) & entry.mask.0;

        // For every subset of possible blockers, get the attacked squares and
        // store them in the table.
        while subset != 0 {
            let attacks = gen_rook_attacks(Square::ALL[sq], Bitboard(subset));
            let blockers = Bitboard(subset);
            let idx = entry.index(blockers);
            table[idx] = attacks;

            subset = subset.wrapping_sub(entry.mask.0) & entry.mask.0;
        }

        sq += 1;
    }

    table
}


////////////////////////////////////////////////////////////////////////////////
//
// Magic tables
//
////////////////////////////////////////////////////////////////////////////////

pub const BISHOP_MAGICS: [MagicEntry; Square::COUNT] = [
 MagicEntry { mask: Bitboard(18049651735527936), magic: 1143543703831040, shift: 58, offset: 0 },
 MagicEntry { mask: Bitboard(70506452091904), magic: 4616207506731991056, shift: 59, offset: 64 },
 MagicEntry { mask: Bitboard(275415828992), magic: 41134946502311936, shift: 59, offset: 96 },
 MagicEntry { mask: Bitboard(1075975168), magic: 9237041792476577800, shift: 59, offset: 128 },
 MagicEntry { mask: Bitboard(38021120), magic: 2324156749898121216, shift: 59, offset: 160 },
 MagicEntry { mask: Bitboard(8657588224), magic: 571763293431064, shift: 59, offset: 192 },
 MagicEntry { mask: Bitboard(2216338399232), magic: 9241675675694268936, shift: 59, offset: 224 },
 MagicEntry { mask: Bitboard(567382630219776), magic: 5764627348637487104, shift: 58, offset: 256 },
 MagicEntry { mask: Bitboard(9024825867763712), magic: 290490981856847112, shift: 59, offset: 320 },
 MagicEntry { mask: Bitboard(18049651735527424), magic: 9949612811531387468, shift: 59, offset: 352 },
 MagicEntry { mask: Bitboard(70506452221952), magic: 14411527775495137280, shift: 59, offset: 384 },
 MagicEntry { mask: Bitboard(275449643008), magic: 8968983333576, shift: 59, offset: 416 },
 MagicEntry { mask: Bitboard(9733406720), magic: 2810248375360815104, shift: 59, offset: 448 },
 MagicEntry { mask: Bitboard(2216342585344), magic: 4899956570242188290, shift: 59, offset: 480 },
 MagicEntry { mask: Bitboard(567382630203392), magic: 27870460605105152, shift: 59, offset: 512 },
 MagicEntry { mask: Bitboard(1134765260406784), magic: 7391071850088456, shift: 59, offset: 544 },
 MagicEntry { mask: Bitboard(4512412933816832), magic: 4538825104173056, shift: 59, offset: 576 },
 MagicEntry { mask: Bitboard(9024825867633664), magic: 9293415943536772, shift: 59, offset: 608 },
 MagicEntry { mask: Bitboard(18049651768822272), magic: 150871699947004418, shift: 57, offset: 640 },
 MagicEntry { mask: Bitboard(70515108615168), magic: 19140332795469824, shift: 57, offset: 768 },
 MagicEntry { mask: Bitboard(2491752130560), magic: 9226750425928040466, shift: 57, offset: 896 },
 MagicEntry { mask: Bitboard(567383701868544), magic: 578783101256343562, shift: 57, offset: 1024 },
 MagicEntry { mask: Bitboard(1134765256220672), magic: 2339769367535685, shift: 59, offset: 1152 },
 MagicEntry { mask: Bitboard(2269530512441344), magic: 634437543397376, shift: 59, offset: 1184 },
 MagicEntry { mask: Bitboard(2256206450263040), magic: 9809970291735545856, shift: 59, offset: 1216 },
 MagicEntry { mask: Bitboard(4512412900526080), magic: 149749085925214208, shift: 59, offset: 1248 },
 MagicEntry { mask: Bitboard(9024834391117824), magic: 144203149157335168, shift: 57, offset: 1280 },
 MagicEntry { mask: Bitboard(18051867805491712), magic: 1226113794711822470, shift: 55, offset: 1408 },
 MagicEntry { mask: Bitboard(637888545440768), magic: 2319635351808786434, shift: 55, offset: 1920 },
 MagicEntry { mask: Bitboard(1135039602493440), magic: 4504149803683858, shift: 57, offset: 2432 },
 MagicEntry { mask: Bitboard(2269529440784384), magic: 144256510283892736, shift: 59, offset: 2560 },
 MagicEntry { mask: Bitboard(4539058881568768), magic: 9295720039971849224, shift: 59, offset: 2592 },
 MagicEntry { mask: Bitboard(1128098963916800), magic: 65311183963955200, shift: 59, offset: 2624 },
 MagicEntry { mask: Bitboard(2256197927833600), magic: 11604879072873940992, shift: 59, offset: 2656 },
 MagicEntry { mask: Bitboard(4514594912477184), magic: 9241404336952575106, shift: 57, offset: 2688 },
 MagicEntry { mask: Bitboard(9592139778506752), magic: 2306476637149233664, shift: 55, offset: 2816 },
 MagicEntry { mask: Bitboard(19184279556981248), magic: 1161092880670848, shift: 55, offset: 3328 },
 MagicEntry { mask: Bitboard(2339762086609920), magic: 159437776683649, shift: 57, offset: 3840 },
 MagicEntry { mask: Bitboard(4538784537380864), magic: 2310426911845777600, shift: 59, offset: 3968 },
 MagicEntry { mask: Bitboard(9077569074761728), magic: 4611831158265692448, shift: 59, offset: 4000 },
 MagicEntry { mask: Bitboard(562958610993152), magic: 6548700399157312, shift: 59, offset: 4032 },
 MagicEntry { mask: Bitboard(1125917221986304), magic: 594651708866433056, shift: 59, offset: 4064 },
 MagicEntry { mask: Bitboard(2814792987328512), magic: 4900479464858722816, shift: 57, offset: 4096 },
 MagicEntry { mask: Bitboard(5629586008178688), magic: 4611722053211527169, shift: 57, offset: 4224 },
 MagicEntry { mask: Bitboard(11259172008099840), magic: 70373055922705, shift: 57, offset: 4352 },
 MagicEntry { mask: Bitboard(22518341868716544), magic: 147105895422108544, shift: 57, offset: 4480 },
 MagicEntry { mask: Bitboard(9007336962655232), magic: 565183353192960, shift: 59, offset: 4608 },
 MagicEntry { mask: Bitboard(18014673925310464), magic: 4522018603270400, shift: 59, offset: 4640 },
 MagicEntry { mask: Bitboard(2216338399232), magic: 216736352758153476, shift: 59, offset: 4672 },
 MagicEntry { mask: Bitboard(4432676798464), magic: 4611769590043772928, shift: 59, offset: 4704 },
 MagicEntry { mask: Bitboard(11064376819712), magic: 603492796636610688, shift: 59, offset: 4736 },
 MagicEntry { mask: Bitboard(22137335185408), magic: 6790602613589024, shift: 59, offset: 4768 },
 MagicEntry { mask: Bitboard(44272556441600), magic: 1261029954650243328, shift: 59, offset: 4800 },
 MagicEntry { mask: Bitboard(87995357200384), magic: 8865368375296, shift: 59, offset: 4832 },
 MagicEntry { mask: Bitboard(35253226045952), magic: 1459249945244139533, shift: 59, offset: 4864 },
 MagicEntry { mask: Bitboard(70506452091904), magic: 37172325652660256, shift: 59, offset: 4896 },
 MagicEntry { mask: Bitboard(567382630219776), magic: 4541005908936713, shift: 58, offset: 4928 },
 MagicEntry { mask: Bitboard(1134765260406784), magic: 1874625569703068161, shift: 59, offset: 4992 },
 MagicEntry { mask: Bitboard(2832480465846272), magic: 576466254165446914, shift: 59, offset: 5024 },
 MagicEntry { mask: Bitboard(5667157807464448), magic: 585468021360036865, shift: 59, offset: 5056 },
 MagicEntry { mask: Bitboard(11333774449049600), magic: 9228016941049914376, shift: 59, offset: 5088 },
 MagicEntry { mask: Bitboard(22526811443298304), magic: 5296241975597737220, shift: 59, offset: 5120 },
 MagicEntry { mask: Bitboard(9024825867763712), magic: 576469583033045504, shift: 59, offset: 5152 },
 MagicEntry { mask: Bitboard(18049651735527936), magic: 333846923255742504, shift: 58, offset: 5184 }
];

pub const ROOK_MAGICS: [MagicEntry; Square::COUNT] = [
MagicEntry { mask: Bitboard(282578800148862), magic: 396334507571101697, shift: 52, offset: 0 },
 MagicEntry { mask: Bitboard(565157600297596), magic: 18014673924829184, shift: 53, offset: 4096 },
 MagicEntry { mask: Bitboard(1130315200595066), magic: 72076294862422104, shift: 53, offset: 6144 },
 MagicEntry { mask: Bitboard(2260630401190006), magic: 324267970069010048, shift: 53, offset: 8192 },
 MagicEntry { mask: Bitboard(4521260802379886), magic: 2449962758546399249, shift: 53, offset: 10240 },
 MagicEntry { mask: Bitboard(9042521604759646), magic: 72060072234319880, shift: 53, offset: 12288 },
 MagicEntry { mask: Bitboard(18085043209519166), magic: 36064531364987392, shift: 53, offset: 14336 },
 MagicEntry { mask: Bitboard(36170086419038334), magic: 252206124290277632, shift: 52, offset: 16384 },
 MagicEntry { mask: Bitboard(282578800180736), magic: 4040432697074532352, shift: 53, offset: 20480 },
 MagicEntry { mask: Bitboard(565157600328704), magic: 9223723949339181122, shift: 54, offset: 22528 },
 MagicEntry { mask: Bitboard(1130315200625152), magic: 13853635543349461056, shift: 54, offset: 23552 },
 MagicEntry { mask: Bitboard(2260630401218048), magic: 2324420530558599200, shift: 54, offset: 24576 },
 MagicEntry { mask: Bitboard(4521260802403840), magic: 141046734390272, shift: 54, offset: 25600 },
 MagicEntry { mask: Bitboard(9042521604775424), magic: 422762254443520, shift: 54, offset: 26624 },
 MagicEntry { mask: Bitboard(18085043209518592), magic: 2387189294696506624, shift: 54, offset: 27648 },
 MagicEntry { mask: Bitboard(36170086419037696), magic: 4644343558193408, shift: 53, offset: 28672 },
 MagicEntry { mask: Bitboard(282578808340736), magic: 1170430131961992, shift: 53, offset: 30720 },
 MagicEntry { mask: Bitboard(565157608292864), magic: 2603714178464129024, shift: 54, offset: 32768 },
 MagicEntry { mask: Bitboard(1130315208328192), magic: 3518988306898944, shift: 54, offset: 33792 },
 MagicEntry { mask: Bitboard(2260630408398848), magic: 360297866348986624, shift: 54, offset: 34816 },
 MagicEntry { mask: Bitboard(4521260808540160), magic: 9262217782933407776, shift: 54, offset: 35840 },
 MagicEntry { mask: Bitboard(9042521608822784), magic: 9429961542730240, shift: 54, offset: 36864 },
 MagicEntry { mask: Bitboard(18085043209388032), magic: 1130298087641094, shift: 54, offset: 37888 },
 MagicEntry { mask: Bitboard(36170086418907136), magic: 10421171212289025, shift: 53, offset: 38912 },
 MagicEntry { mask: Bitboard(282580897300736), magic: 5800636596004855808, shift: 53, offset: 40960 },
 MagicEntry { mask: Bitboard(565159647117824), magic: 18331076111912960, shift: 54, offset: 43008 },
 MagicEntry { mask: Bitboard(1130317180306432), magic: 2450241050951819272, shift: 54, offset: 44032 },
 MagicEntry { mask: Bitboard(2260632246683648), magic: 2305959351820802, shift: 54, offset: 45056 },
 MagicEntry { mask: Bitboard(4521262379438080), magic: 326511524888183810, shift: 54, offset: 46080 },
 MagicEntry { mask: Bitboard(9042522644946944), magic: 1127000492343360, shift: 54, offset: 47104 },
 MagicEntry { mask: Bitboard(18085043175964672), magic: 72058710730475522, shift: 54, offset: 48128 },
 MagicEntry { mask: Bitboard(36170086385483776), magic: 1315192112148418820, shift: 53, offset: 49152 },
 MagicEntry { mask: Bitboard(283115671060736), magic: 141562147242016, shift: 53, offset: 51200 },
 MagicEntry { mask: Bitboard(565681586307584), magic: 54050343436165120, shift: 54, offset: 53248 },
 MagicEntry { mask: Bitboard(1130822006735872), magic: 290517362560471040, shift: 54, offset: 54272 },
 MagicEntry { mask: Bitboard(2261102847592448), magic: 6773010962843648, shift: 54, offset: 55296 },
 MagicEntry { mask: Bitboard(4521664529305600), magic: 422367092279296, shift: 54, offset: 56320 },
 MagicEntry { mask: Bitboard(9042787892731904), magic: 4920746711120353284, shift: 54, offset: 57344 },
 MagicEntry { mask: Bitboard(18085034619584512), magic: 2814767248965912, shift: 54, offset: 58368 },
 MagicEntry { mask: Bitboard(36170077829103616), magic: 9664725504745275969, shift: 53, offset: 59392 },
 MagicEntry { mask: Bitboard(420017753620736), magic: 108227403458838528, shift: 53, offset: 61440 },
 MagicEntry { mask: Bitboard(699298018886144), magic: 1161946296806948864, shift: 54, offset: 63488 },
 MagicEntry { mask: Bitboard(1260057572672512), magic: 72080684319047744, shift: 54, offset: 64512 },
 MagicEntry { mask: Bitboard(2381576680245248), magic: 563225169100808, shift: 54, offset: 65536 },
 MagicEntry { mask: Bitboard(4624614895390720), magic: 9374242796992528405, shift: 54, offset: 66560 },
 MagicEntry { mask: Bitboard(9110691325681664), magic: 9804900541598400516, shift: 54, offset: 67584 },
 MagicEntry { mask: Bitboard(18082844186263552), magic: 36899680323633153, shift: 54, offset: 68608 },
 MagicEntry { mask: Bitboard(36167887395782656), magic: 18160262709249, shift: 53, offset: 69632 },
 MagicEntry { mask: Bitboard(35466950888980736), magic: 441987183975465472, shift: 53, offset: 71680 },
 MagicEntry { mask: Bitboard(34905104758997504), magic: 170222060954452032, shift: 54, offset: 73728 },
 MagicEntry { mask: Bitboard(34344362452452352), magic: 9150136340976128, shift: 54, offset: 74752 },
 MagicEntry { mask: Bitboard(33222877839362048), magic: 35770907886080, shift: 54, offset: 75776 },
 MagicEntry { mask: Bitboard(30979908613181440), magic: 1125934275330176, shift: 54, offset: 76800 },
 MagicEntry { mask: Bitboard(26493970160820224), magic: 563019076338176, shift: 54, offset: 77824 },
 MagicEntry { mask: Bitboard(17522093256097792), magic: 563002852508160, shift: 54, offset: 78848 },
 MagicEntry { mask: Bitboard(35607136465616896), magic: 1162073852289769984, shift: 53, offset: 79872 },
 MagicEntry { mask: Bitboard(9079539427579068672), magic: 422230469181761, shift: 52, offset: 81920 },
 MagicEntry { mask: Bitboard(8935706818303361536), magic: 4683762314933977122, shift: 53, offset: 86016 },
 MagicEntry { mask: Bitboard(8792156787827803136), magic: 1441996340860027971, shift: 53, offset: 88064 },
 MagicEntry { mask: Bitboard(8505056726876686336), magic: 4503669488247041, shift: 53, offset: 90112 },
 MagicEntry { mask: Bitboard(7930856604974452736), magic: 649081365551645714, shift: 53, offset: 92160 },
 MagicEntry { mask: Bitboard(6782456361169985536), magic: 3659218049826819, shift: 53, offset: 94208 },
 MagicEntry { mask: Bitboard(4485655873561051136), magic: 90353538491745281, shift: 53, offset: 96256 },
 MagicEntry { mask: Bitboard(9115426935197958144), magic: 140896737722434, shift: 52, offset: 98304 }
];


////////////////////////////////////////////////////////////////////////////////
//
// Tests
//
////////////////////////////////////////////////////////////////////////////////

#[test]
fn test_subsets() {
    use Square::*;
    let mask: Bitboard = vec![A1, B1, C1, D1]
        .into_iter()
        .map(|sq| Bitboard::from(sq))
        .collect();

    assert_eq!(mask.subsets().count(), 16);

    assert!(mask.subsets()
        .find(|&subset| subset == Bitboard::from(A1) | Bitboard::from(B1)
    ).is_some());
}

#[test]
fn test_gen_bishop_mask() {
    use Square::*;
    assert_eq!(bishop_mask(E3), Bitboard(0x24428002800));
    assert_eq!(bishop_mask(H1), Bitboard(0x2040810204000));
    assert_eq!(bishop_mask(C8), Bitboard(0xa102040000000));
}

#[test]
fn test_gen_bishop_attacks() {
    use Square::*;
    let attacks = gen_bishop_attacks(D3, Bitboard(0xb0430800420423));
    assert_eq!(attacks, Bitboard(0x412214001420));
}

#[test]
fn test_rook_mask() {
    use Square::*;

    assert_eq!(rook_mask(E3), Bitboard(0x101010106e1000));
    assert_eq!(rook_mask(A1), Bitboard(0x0101010101017e));
}

#[test]
fn test_rook_attacks() {
    use Square::*;

    let attacks = rook_attacks(E3, Bitboard(0xb0430800420423));
    assert_eq!(attacks, Bitboard(0x101010106e1010));
}

