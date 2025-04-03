use crate::evaluate::S;
use crate::s;
use super::tuner::EvalWeights;

pub const PARAMS: EvalWeights = EvalWeights {
    piece_values: [
        s!(35,59),
        s!(117,168),
        s!(85,122),
        s!(147,274),
        s!(346,428),
        s!(0,0),
    ],
    pawn_psqt: [
        s!(0,0),
        s!(0,0),
        s!(0,0),
        s!(0,0),
        s!(0,0),
        s!(0,0),
        s!(0,0),
        s!(0,0),
        s!(23,110),
        s!(39,92),
        s!(30,101),
        s!(40,82),
        s!(39,84),
        s!(38,87),
        s!(17,97),
        s!(-4,114),
        s!(16,69),
        s!(3,65),
        s!(28,55),
        s!(26,59),
        s!(41,42),
        s!(64,46),
        s!(45,72),
        s!(27,77),
        s!(5,66),
        s!(8,53),
        s!(11,52),
        s!(13,44),
        s!(28,43),
        s!(31,43),
        s!(17,51),
        s!(18,53),
        s!(5,49),
        s!(5,52),
        s!(14,46),
        s!(22,47),
        s!(25,47),
        s!(23,44),
        s!(14,46),
        s!(13,40),
        s!(-3,44),
        s!(-1,42),
        s!(8,44),
        s!(9,50),
        s!(14,51),
        s!(11,49),
        s!(19,35),
        s!(8,35),
        s!(6,49),
        s!(7,48),
        s!(14,52),
        s!(10,56),
        s!(19,63),
        s!(28,56),
        s!(33,41),
        s!(13,40),
        s!(0,0),
        s!(0,0),
        s!(0,0),
        s!(0,0),
        s!(0,0),
        s!(0,0),
        s!(0,0),
        s!(0,0),
    ],
    knight_psqt: [
        s!(-31,67),
        s!(-43,107),
        s!(-12,115),
        s!(30,98),
        s!(46,105),
        s!(8,84),
        s!(-29,104),
        s!(0,41),
        s!(41,113),
        s!(52,115),
        s!(51,108),
        s!(59,112),
        s!(59,97),
        s!(85,91),
        s!(60,109),
        s!(64,92),
        s!(54,109),
        s!(59,109),
        s!(58,114),
        s!(78,109),
        s!(85,103),
        s!(97,88),
        s!(64,101),
        s!(70,95),
        s!(66,116),
        s!(70,109),
        s!(82,110),
        s!(83,118),
        s!(91,115),
        s!(92,111),
        s!(84,109),
        s!(94,106),
        s!(61,114),
        s!(71,105),
        s!(73,115),
        s!(80,114),
        s!(78,123),
        s!(87,107),
        s!(81,110),
        s!(77,113),
        s!(43,99),
        s!(54,98),
        s!(55,96),
        s!(57,111),
        s!(66,109),
        s!(61,91),
        s!(72,94),
        s!(58,101),
        s!(40,103),
        s!(47,105),
        s!(48,97),
        s!(57,102),
        s!(57,98),
        s!(56,98),
        s!(60,96),
        s!(58,111),
        s!(15,116),
        s!(43,94),
        s!(36,103),
        s!(47,105),
        s!(52,107),
        s!(52,93),
        s!(45,102),
        s!(46,111),
    ],
    bishop_psqt: [
        s!(13,88),
        s!(1,102),
        s!(0,95),
        s!(-23,103),
        s!(-17,96),
        s!(-7,91),
        s!(13,92),
        s!(-2,84),
        s!(44,81),
        s!(26,79),
        s!(42,83),
        s!(26,88),
        s!(39,85),
        s!(45,80),
        s!(23,82),
        s!(34,79),
        s!(47,94),
        s!(52,87),
        s!(37,81),
        s!(56,84),
        s!(49,85),
        s!(77,83),
        s!(58,84),
        s!(69,92),
        s!(42,90),
        s!(49,89),
        s!(58,89),
        s!(60,99),
        s!(68,91),
        s!(62,87),
        s!(62,88),
        s!(45,91),
        s!(47,84),
        s!(48,91),
        s!(53,90),
        s!(68,93),
        s!(66,87),
        s!(62,87),
        s!(53,86),
        s!(68,74),
        s!(47,85),
        s!(59,85),
        s!(50,81),
        s!(51,87),
        s!(56,92),
        s!(51,75),
        s!(65,78),
        s!(60,73),
        s!(59,82),
        s!(50,67),
        s!(60,69),
        s!(48,84),
        s!(51,82),
        s!(57,78),
        s!(60,65),
        s!(63,70),
        s!(57,77),
        s!(63,80),
        s!(50,86),
        s!(46,83),
        s!(55,83),
        s!(40,93),
        s!(59,76),
        s!(72,56),
    ],
    rook_psqt: [
        s!(76,239),
        s!(65,246),
        s!(62,250),
        s!(62,243),
        s!(68,238),
        s!(87,238),
        s!(80,245),
        s!(87,238),
        s!(100,213),
        s!(101,218),
        s!(108,221),
        s!(123,206),
        s!(110,208),
        s!(122,210),
        s!(116,212),
        s!(121,206),
        s!(74,238),
        s!(97,230),
        s!(89,233),
        s!(91,224),
        s!(113,217),
        s!(117,214),
        s!(140,210),
        s!(107,214),
        s!(78,238),
        s!(93,231),
        s!(93,234),
        s!(93,227),
        s!(98,218),
        s!(111,213),
        s!(109,218),
        s!(100,215),
        s!(74,232),
        s!(78,232),
        s!(84,226),
        s!(88,222),
        s!(94,220),
        s!(89,220),
        s!(108,211),
        s!(88,214),
        s!(69,222),
        s!(72,218),
        s!(77,214),
        s!(81,213),
        s!(87,208),
        s!(91,200),
        s!(112,186),
        s!(95,190),
        s!(71,207),
        s!(74,212),
        s!(83,209),
        s!(84,207),
        s!(89,200),
        s!(92,196),
        s!(100,189),
        s!(79,194),
        s!(80,209),
        s!(83,209),
        s!(86,213),
        s!(92,205),
        s!(96,198),
        s!(88,203),
        s!(93,199),
        s!(82,195),
    ],
    queen_psqt: [
        s!(196,365),
        s!(190,366),
        s!(209,364),
        s!(239,344),
        s!(211,368),
        s!(219,371),
        s!(244,324),
        s!(215,359),
        s!(246,329),
        s!(237,325),
        s!(237,345),
        s!(226,353),
        s!(214,376),
        s!(240,350),
        s!(245,336),
        s!(277,340),
        s!(230,357),
        s!(232,346),
        s!(227,364),
        s!(233,357),
        s!(224,367),
        s!(246,351),
        s!(248,341),
        s!(245,351),
        s!(225,366),
        s!(229,368),
        s!(230,369),
        s!(227,371),
        s!(228,371),
        s!(242,354),
        s!(246,373),
        s!(246,359),
        s!(227,360),
        s!(226,380),
        s!(226,374),
        s!(231,380),
        s!(238,371),
        s!(236,371),
        s!(245,362),
        s!(245,368),
        s!(226,349),
        s!(228,360),
        s!(226,376),
        s!(225,370),
        s!(228,373),
        s!(232,369),
        s!(244,351),
        s!(242,347),
        s!(231,340),
        s!(228,344),
        s!(231,344),
        s!(232,355),
        s!(231,357),
        s!(236,332),
        s!(244,302),
        s!(259,287),
        s!(223,345),
        s!(227,342),
        s!(228,347),
        s!(227,359),
        s!(232,339),
        s!(219,339),
        s!(230,329),
        s!(233,326),
    ],
    king_psqt: [
        s!(13,-86),
        s!(-5,-27),
        s!(0,-13),
        s!(-11,12),
        s!(2,-1),
        s!(-23,20),
        s!(18,17),
        s!(59,-94),
        s!(-59,13),
        s!(-23,50),
        s!(-31,59),
        s!(48,32),
        s!(26,45),
        s!(11,65),
        s!(3,64),
        s!(-4,24),
        s!(-52,16),
        s!(14,43),
        s!(-18,64),
        s!(-28,76),
        s!(15,77),
        s!(45,69),
        s!(15,61),
        s!(8,14),
        s!(-18,-4),
        s!(-11,31),
        s!(-39,59),
        s!(-66,81),
        s!(-55,80),
        s!(-18,66),
        s!(-1,39),
        s!(-54,16),
        s!(-18,-17),
        s!(4,12),
        s!(-16,40),
        s!(-51,63),
        s!(-38,61),
        s!(-2,39),
        s!(0,19),
        s!(-58,5),
        s!(-5,-21),
        s!(28,0),
        s!(0,23),
        s!(-8,35),
        s!(-7,34),
        s!(3,22),
        s!(12,5),
        s!(-17,-11),
        s!(46,-33),
        s!(22,-2),
        s!(17,7),
        s!(-11,19),
        s!(-10,21),
        s!(2,12),
        s!(27,-8),
        s!(28,-28),
        s!(16,-61),
        s!(20,-27),
        s!(8,-6),
        s!(-45,8),
        s!(-10,-8),
        s!(-35,6),
        s!(2,-19),
        s!(16,-63),
    ],
    passed_pawn: [
        s!(0,0),
        s!(0,0),
        s!(0,0),
        s!(0,0),
        s!(0,0),
        s!(0,0),
        s!(0,0),
        s!(0,0),
        s!(23,110),
        s!(39,92),
        s!(30,101),
        s!(40,82),
        s!(39,84),
        s!(38,87),
        s!(17,97),
        s!(-4,114),
        s!(12,63),
        s!(40,61),
        s!(18,41),
        s!(17,19),
        s!(8,34),
        s!(13,40),
        s!(1,44),
        s!(-30,61),
        s!(9,41),
        s!(12,36),
        s!(19,14),
        s!(9,10),
        s!(0,10),
        s!(6,17),
        s!(-2,35),
        s!(-8,41),
        s!(-7,29),
        s!(-2,16),
        s!(-10,5),
        s!(-6,-10),
        s!(-14,-5),
        s!(-8,3),
        s!(-6,20),
        s!(-12,25),
        s!(-12,3),
        s!(-15,6),
        s!(-17,-9),
        s!(-14,-23),
        s!(-17,-20),
        s!(-13,-18),
        s!(-19,5),
        s!(-5,-5),
        s!(-25,6),
        s!(-15,-3),
        s!(-23,-17),
        s!(-23,-19),
        s!(-14,-40),
        s!(-23,-29),
        s!(-13,-23),
        s!(-14,-10),
        s!(0,0),
        s!(0,0),
        s!(0,0),
        s!(0,0),
        s!(0,0),
        s!(0,0),
        s!(0,0),
        s!(0,0),
    ],
    knight_mobility: [
        s!(56,84),
        s!(35,61),
        s!(55,94),
        s!(64,116),
        s!(74,126),
        s!(79,139),
        s!(87,143),
        s!(94,151),
        s!(104,145),
    ],
    bishop_mobility: [
        s!(25,36),
        s!(8,8),
        s!(31,50),
        s!(40,71),
        s!(50,81),
        s!(56,90),
        s!(60,100),
        s!(64,104),
        s!(65,109),
        s!(68,110),
        s!(70,112),
        s!(81,105),
        s!(85,108),
        s!(90,96),
    ],
    rook_mobility: [
        s!(69,110),
        s!(57,199),
        s!(80,178),
        s!(88,198),
        s!(92,210),
        s!(95,217),
        s!(96,223),
        s!(98,229),
        s!(101,233),
        s!(103,237),
        s!(105,242),
        s!(105,248),
        s!(106,252),
        s!(105,257),
        s!(106,257),
    ],
    queen_mobility: [
        s!(259,356),
        s!(241,367),
        s!(242,330),
        s!(238,297),
        s!(242,285),
        s!(259,319),
        s!(265,342),
        s!(266,368),
        s!(269,381),
        s!(271,395),
        s!(275,397),
        s!(278,400),
        s!(280,405),
        s!(284,403),
        s!(286,404),
        s!(288,408),
        s!(289,407),
        s!(292,407),
        s!(294,409),
        s!(297,403),
        s!(308,393),
        s!(316,377),
        s!(331,362),
        s!(343,344),
        s!(349,333),
        s!(413,268),
        s!(358,286),
        s!(312,303),
    ],
    virtual_mobility: [
        s!(0,-4),
        s!(5,-6),
        s!(3,-1),
        s!(1,-4),
        s!(0,-1),
        s!(1,-2),
        s!(1,-1),
        s!(0,-1),
        s!(0,4),
        s!(-4,6),
        s!(-7,8),
        s!(-11,11),
        s!(-12,12),
        s!(-16,14),
        s!(-16,13),
        s!(-19,14),
        s!(-21,11),
        s!(-22,12),
        s!(-15,8),
        s!(-2,0),
        s!(2,-2),
        s!(9,-7),
        s!(5,-10),
        s!(25,-19),
        s!(17,-22),
        s!(54,-37),
        s!(68,-42),
        s!(92,-44),
    ],
    king_zone: [
        s!(20,-12),
        s!(21,-1),
        s!(11,0),
        s!(-1,-4),
        s!(-37,17),
        s!(-72,28),
        s!(-125,59),
        s!(-159,65),
        s!(-192,28),
        s!(-215,-4),
        s!(-185,-53),
        s!(-110,-29),
        s!(-63,-24),
        s!(-5,-1),
        s!(0,0),
        s!(0,0),
    ],
    isolated_pawn: [
        s!(0,0),
        s!(-6,-10),
        s!(-3,-8),
        s!(-5,-10),
        s!(4,-14),
        s!(14,-12),
        s!(39,-14),
        s!(0,0),
    ],
    doubled_pawn: [
        s!(0,0),
        s!(-1,-21),
        s!(-7,-16),
        s!(-6,-25),
        s!(24,-53),
        s!(-105,6),
        s!(0,0),
        s!(0,0),
    ],
    protected_pawn: [
        s!(0,0),
        s!(0,0),
        s!(16,13),
        s!(9,9),
        s!(14,13),
        s!(24,48),
        s!(146,53),
        s!(0,0),
    ],
    phalanx_pawn: [
        s!(0,0),
        s!(3,0),
        s!(4,5),
        s!(8,8),
        s!(22,29),
        s!(47,109),
        s!(-60,232),
        s!(0,0),
    ],
    bishop_pair: s!(17,55),
    rook_open_file: s!(8,0),
    rook_semiopen_file: s!(15,3),
    connected_rooks: s!(-3,2),
    major_on_seventh: s!(-21,32),
    queen_open_file: s!(-15,32),
    queen_semiopen_file: s!(3,12),
    pawn_shield: [
        s!(20,-14),
        s!(15,-6),
        s!(13,-10),
    ],
    pawn_storm: [
        s!(0,-23),
        s!(0,-3),
        s!(-8,-1),
    ],
    passers_friendly_king: [
        s!(-13,43),
        s!(-11,27),
        s!(-7,7),
        s!(-4,-3),
        s!(-2,-7),
        s!(9,-10),
        s!(1,-14),
    ],
    passers_enemy_king: [
        s!(-49,-11),
        s!(-1,-13),
        s!(-3,8),
        s!(-5,20),
        s!(-4,26),
        s!(-2,31),
        s!(-13,29),
    ],
    pawn_attacks: [
        s!(8,-15),
        s!(56,24),
        s!(51,51),
        s!(68,25),
        s!(60,-16),
        s!(0,0),
    ],
    knight_attacks: [
        s!(-4,13),
        s!(-1,22),
        s!(24,34),
        s!(56,17),
        s!(44,-21),
        s!(0,0),
    ],
    bishop_attacks: [
        s!(-2,11),
        s!(16,25),
        s!(0,0),
        s!(42,26),
        s!(54,87),
        s!(0,0),
    ],
    rook_attacks: [
        s!(-6,13),
        s!(3,19),
        s!(10,13),
        s!(-23,-15),
        s!(60,23),
        s!(0,0),
    ],
    queen_attacks: [
        s!(-4,15),
        s!(0,11),
        s!(0,23),
        s!(0,2),
        s!(19,3),
        s!(0,0),
    ],
    knight_outposts: s!(17,17),
    bishop_outposts: s!(27,10),
    knight_shelter: s!(4,12),
    bishop_shelter: s!(7,0),
    tempo: s!(24,25),
    safe_checks: [
        s!(9,5),
        s!(66,-10),
        s!(14,19),
        s!(62,-1),
        s!(18,22),
        s!(0,0),
    ],
    unsafe_checks: [
        s!(-1,9),
        s!(10,-2),
        s!(13,11),
        s!(20,-5),
        s!(3,14),
        s!(0,0),
    ],
    bad_bishops: [
        s!(79,130),
        s!(82,124),
        s!(80,116),
        s!(76,109),
        s!(73,100),
        s!(72,89),
        s!(69,81),
        s!(66,68),
        s!(56,53),
    ],
    square_rule: s!(41,219),
    free_passer: [
        s!(0,0),
        s!(3,1),
        s!(1,-3),
        s!(-7,13),
        s!(-17,48),
        s!(-35,144),
        s!(-14,218),
        s!(0,0),
    ],
    protected_passer: [
        s!(0,0),
        s!(24,-30),
        s!(17,-12),
        s!(16,0),
        s!(17,15),
        s!(29,23),
        s!(35,-6),
        s!(0,0),
    ],
    bishop_long_diagonal: s!(11,6),
    push_threats: [
        s!(0,0),
        s!(13,17),
        s!(14,16),
        s!(20,10),
        s!(14,9),
        s!(67,14),
    ],
    forks: [
        s!(-4,-1),
        s!(17,9),
        s!(6,-4),
        s!(-13,8),
        s!(3,22),
        s!(0,0),
    ],
};