use crate::evaluate::S;
use crate::s;
use super::tuner::EvalWeights;

pub const PARAMS: EvalWeights = EvalWeights {
    piece_values: [
        s!(34,60),
        s!(115,176),
        s!(84,128),
        s!(143,287),
        s!(317,476),
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
        s!(29,112),
        s!(43,96),
        s!(32,105),
        s!(40,86),
        s!(38,89),
        s!(36,91),
        s!(20,99),
        s!(-2,117),
        s!(16,72),
        s!(3,66),
        s!(29,54),
        s!(25,61),
        s!(44,43),
        s!(64,44),
        s!(46,71),
        s!(26,77),
        s!(5,68),
        s!(8,55),
        s!(11,53),
        s!(13,43),
        s!(28,43),
        s!(31,41),
        s!(17,50),
        s!(17,53),
        s!(5,52),
        s!(5,53),
        s!(14,47),
        s!(22,48),
        s!(25,47),
        s!(23,43),
        s!(14,45),
        s!(13,40),
        s!(-3,46),
        s!(-1,43),
        s!(7,45),
        s!(9,51),
        s!(14,51),
        s!(11,47),
        s!(19,34),
        s!(8,34),
        s!(5,51),
        s!(6,50),
        s!(14,54),
        s!(10,57),
        s!(19,64),
        s!(28,56),
        s!(33,40),
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
        s!(-30,66),
        s!(-45,113),
        s!(-15,121),
        s!(27,105),
        s!(43,113),
        s!(8,89),
        s!(-31,110),
        s!(2,42),
        s!(40,118),
        s!(50,121),
        s!(49,113),
        s!(57,117),
        s!(57,103),
        s!(82,97),
        s!(59,115),
        s!(62,98),
        s!(53,114),
        s!(58,114),
        s!(56,118),
        s!(76,115),
        s!(83,109),
        s!(96,92),
        s!(62,106),
        s!(68,100),
        s!(65,122),
        s!(69,115),
        s!(81,115),
        s!(81,123),
        s!(89,120),
        s!(90,116),
        s!(82,115),
        s!(92,112),
        s!(60,120),
        s!(70,110),
        s!(72,120),
        s!(79,119),
        s!(76,129),
        s!(85,113),
        s!(79,115),
        s!(75,119),
        s!(42,104),
        s!(53,103),
        s!(54,101),
        s!(55,116),
        s!(65,114),
        s!(59,97),
        s!(70,100),
        s!(57,106),
        s!(39,108),
        s!(46,111),
        s!(47,102),
        s!(56,107),
        s!(56,103),
        s!(55,103),
        s!(58,101),
        s!(57,117),
        s!(14,122),
        s!(41,99),
        s!(35,108),
        s!(45,110),
        s!(51,113),
        s!(50,98),
        s!(44,107),
        s!(46,116),
    ],
    bishop_psqt: [
        s!(13,91),
        s!(0,106),
        s!(-1,99),
        s!(-26,107),
        s!(-19,101),
        s!(-8,95),
        s!(11,97),
        s!(-1,87),
        s!(44,85),
        s!(25,83),
        s!(41,86),
        s!(25,92),
        s!(38,89),
        s!(44,85),
        s!(22,86),
        s!(33,83),
        s!(46,98),
        s!(51,91),
        s!(36,85),
        s!(55,87),
        s!(48,89),
        s!(76,88),
        s!(57,88),
        s!(67,97),
        s!(41,94),
        s!(49,93),
        s!(57,92),
        s!(59,103),
        s!(67,95),
        s!(61,91),
        s!(62,92),
        s!(44,95),
        s!(46,88),
        s!(47,95),
        s!(52,94),
        s!(67,97),
        s!(65,91),
        s!(61,91),
        s!(52,90),
        s!(66,78),
        s!(46,89),
        s!(58,89),
        s!(49,85),
        s!(50,91),
        s!(55,96),
        s!(50,80),
        s!(64,82),
        s!(59,77),
        s!(58,86),
        s!(50,71),
        s!(59,73),
        s!(47,88),
        s!(50,86),
        s!(56,82),
        s!(59,69),
        s!(62,75),
        s!(55,81),
        s!(62,85),
        s!(49,89),
        s!(45,87),
        s!(54,87),
        s!(39,97),
        s!(58,80),
        s!(71,60),
    ],
    rook_psqt: [
        s!(72,250),
        s!(60,258),
        s!(57,262),
        s!(57,254),
        s!(63,250),
        s!(82,250),
        s!(77,255),
        s!(83,249),
        s!(103,221),
        s!(103,227),
        s!(110,229),
        s!(124,215),
        s!(111,217),
        s!(124,219),
        s!(118,221),
        s!(125,213),
        s!(70,248),
        s!(93,241),
        s!(85,244),
        s!(87,235),
        s!(108,228),
        s!(113,225),
        s!(136,221),
        s!(103,224),
        s!(75,249),
        s!(89,242),
        s!(89,245),
        s!(89,238),
        s!(94,229),
        s!(107,224),
        s!(105,229),
        s!(96,226),
        s!(71,242),
        s!(75,243),
        s!(80,237),
        s!(84,233),
        s!(90,231),
        s!(85,231),
        s!(104,222),
        s!(85,224),
        s!(66,232),
        s!(69,228),
        s!(74,224),
        s!(78,223),
        s!(83,218),
        s!(88,210),
        s!(108,196),
        s!(91,200),
        s!(68,217),
        s!(70,222),
        s!(80,219),
        s!(81,216),
        s!(85,210),
        s!(89,206),
        s!(97,199),
        s!(76,204),
        s!(77,219),
        s!(80,219),
        s!(83,223),
        s!(89,214),
        s!(93,208),
        s!(85,213),
        s!(90,208),
        s!(79,205),
    ],
    queen_psqt: [
        s!(172,410),
        s!(157,419),
        s!(179,417),
        s!(211,394),
        s!(182,420),
        s!(176,441),
        s!(214,377),
        s!(188,411),
        s!(236,358),
        s!(225,359),
        s!(225,383),
        s!(212,396),
        s!(198,423),
        s!(226,393),
        s!(231,374),
        s!(264,374),
        s!(213,389),
        s!(214,383),
        s!(207,407),
        s!(212,404),
        s!(201,419),
        s!(221,406),
        s!(226,387),
        s!(224,393),
        s!(208,398),
        s!(211,405),
        s!(211,411),
        s!(207,418),
        s!(207,421),
        s!(221,402),
        s!(225,417),
        s!(227,399),
        s!(209,395),
        s!(208,417),
        s!(207,414),
        s!(212,423),
        s!(218,416),
        s!(217,413),
        s!(226,401),
        s!(227,404),
        s!(209,382),
        s!(211,395),
        s!(208,412),
        s!(207,407),
        s!(210,410),
        s!(214,406),
        s!(226,387),
        s!(224,382),
        s!(214,372),
        s!(211,377),
        s!(214,378),
        s!(215,388),
        s!(214,390),
        s!(219,366),
        s!(227,336),
        s!(241,320),
        s!(206,378),
        s!(210,374),
        s!(212,379),
        s!(210,390),
        s!(215,372),
        s!(202,371),
        s!(213,360),
        s!(217,356),
    ],
    king_psqt: [
        s!(22,-92),
        s!(5,-34),
        s!(12,-19),
        s!(0,7),
        s!(13,-5),
        s!(-26,20),
        s!(11,17),
        s!(74,-104),
        s!(-60,7),
        s!(-16,48),
        s!(-23,58),
        s!(55,30),
        s!(35,43),
        s!(23,64),
        s!(8,61),
        s!(-7,21),
        s!(-51,11),
        s!(19,41),
        s!(-9,62),
        s!(-18,74),
        s!(19,77),
        s!(54,66),
        s!(17,58),
        s!(14,8),
        s!(-18,-8),
        s!(-9,30),
        s!(-31,60),
        s!(-58,80),
        s!(-50,81),
        s!(-14,66),
        s!(1,37),
        s!(-49,11),
        s!(-21,-18),
        s!(5,12),
        s!(-9,41),
        s!(-44,64),
        s!(-33,62),
        s!(2,40),
        s!(2,21),
        s!(-58,5),
        s!(-6,-19),
        s!(30,1),
        s!(3,24),
        s!(-5,35),
        s!(-4,35),
        s!(5,24),
        s!(13,8),
        s!(-17,-9),
        s!(43,-31),
        s!(20,-1),
        s!(17,7),
        s!(-11,18),
        s!(-10,21),
        s!(2,13),
        s!(26,-5),
        s!(26,-26),
        s!(14,-60),
        s!(19,-26),
        s!(7,-6),
        s!(-45,8),
        s!(-10,-9),
        s!(-35,5),
        s!(1,-19),
        s!(15,-63),
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
        s!(29,112),
        s!(43,96),
        s!(32,105),
        s!(40,86),
        s!(38,89),
        s!(36,91),
        s!(20,99),
        s!(-2,117),
        s!(18,64),
        s!(32,68),
        s!(5,51),
        s!(3,27),
        s!(-12,46),
        s!(1,51),
        s!(-10,55),
        s!(-38,70),
        s!(19,38),
        s!(11,38),
        s!(14,19),
        s!(0,20),
        s!(-12,25),
        s!(-3,29),
        s!(-9,45),
        s!(-13,48),
        s!(4,22),
        s!(-1,15),
        s!(-13,6),
        s!(-11,0),
        s!(-20,5),
        s!(-13,12),
        s!(-8,26),
        s!(-13,29),
        s!(1,-6),
        s!(-13,1),
        s!(-15,-13),
        s!(-10,-26),
        s!(-13,-19),
        s!(-10,-19),
        s!(-16,5),
        s!(0,-7),
        s!(-11,-7),
        s!(-10,-12),
        s!(-18,-27),
        s!(-16,-32),
        s!(-8,-53),
        s!(-17,-39),
        s!(-8,-31),
        s!(-7,-19),
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
        s!(54,89),
        s!(34,64),
        s!(54,98),
        s!(63,122),
        s!(72,133),
        s!(77,146),
        s!(85,150),
        s!(92,159),
        s!(102,153),
    ],
    bishop_mobility: [
        s!(24,39),
        s!(8,10),
        s!(30,53),
        s!(39,74),
        s!(49,85),
        s!(55,94),
        s!(58,105),
        s!(62,109),
        s!(64,114),
        s!(67,116),
        s!(69,118),
        s!(79,111),
        s!(84,114),
        s!(87,102),
    ],
    rook_mobility: [
        s!(66,121),
        s!(54,212),
        s!(78,188),
        s!(85,209),
        s!(89,221),
        s!(92,229),
        s!(94,234),
        s!(95,241),
        s!(98,244),
        s!(100,249),
        s!(102,254),
        s!(102,261),
        s!(103,265),
        s!(101,270),
        s!(102,269),
    ],
    queen_mobility: [
        s!(223,436),
        s!(215,425),
        s!(218,384),
        s!(215,351),
        s!(220,336),
        s!(238,365),
        s!(243,388),
        s!(244,414),
        s!(247,426),
        s!(250,441),
        s!(253,443),
        s!(256,445),
        s!(258,450),
        s!(262,449),
        s!(264,450),
        s!(266,454),
        s!(267,452),
        s!(270,454),
        s!(272,454),
        s!(275,448),
        s!(285,439),
        s!(293,420),
        s!(306,407),
        s!(317,386),
        s!(322,378),
        s!(409,281),
        s!(366,290),
        s!(315,306),
    ],
    virtual_mobility: [
        s!(1,0),
        s!(6,-3),
        s!(4,0),
        s!(2,-2),
        s!(1,1),
        s!(2,0),
        s!(2,0),
        s!(0,0),
        s!(0,5),
        s!(-3,7),
        s!(-6,10),
        s!(-10,12),
        s!(-11,13),
        s!(-16,14),
        s!(-16,13),
        s!(-19,14),
        s!(-22,11),
        s!(-23,11),
        s!(-17,7),
        s!(-3,-1),
        s!(1,-4),
        s!(7,-9),
        s!(3,-12),
        s!(22,-22),
        s!(15,-25),
        s!(51,-40),
        s!(64,-45),
        s!(88,-48),
    ],
    king_zone: [
        s!(21,-13),
        s!(22,-1),
        s!(11,0),
        s!(-1,-4),
        s!(-37,17),
        s!(-72,27),
        s!(-126,59),
        s!(-165,70),
        s!(-214,54),
        s!(-253,48),
        s!(-217,-18),
        s!(-142,-29),
        s!(-92,-37),
        s!(-8,-2),
        s!(0,0),
        s!(0,0),
    ],
    isolated_pawn: [
        s!(0,0),
        s!(-6,-11),
        s!(-3,-8),
        s!(-5,-11),
        s!(4,-15),
        s!(14,-13),
        s!(40,-16),
        s!(0,0),
    ],
    doubled_pawn: [
        s!(0,0),
        s!(-1,-21),
        s!(-7,-16),
        s!(-6,-25),
        s!(24,-54),
        s!(-105,3),
        s!(0,0),
        s!(0,0),
    ],
    protected_pawn: [
        s!(0,0),
        s!(0,0),
        s!(16,14),
        s!(9,9),
        s!(14,14),
        s!(23,51),
        s!(138,65),
        s!(0,0),
    ],
    phalanx_pawn: [
        s!(0,0),
        s!(3,0),
        s!(4,5),
        s!(8,8),
        s!(22,31),
        s!(44,118),
        s!(-92,262),
        s!(0,0),
    ],
    bishop_pair: s!(17,56),
    rook_open_file: s!(8,0),
    rook_semiopen_file: s!(15,4),
    connected_rooks: s!(-3,1),
    major_on_seventh: s!(-28,35),
    queen_open_file: s!(-14,29),
    queen_semiopen_file: s!(3,11),
    pawn_shield: [
        s!(19,-11),
        s!(14,-2),
        s!(12,-7),
    ],
    pawn_storm: [
        s!(15,-25),
        s!(2,0),
        s!(-8,1),
    ],
    passers_friendly_king: [
        s!(-2,52),
        s!(-12,57),
        s!(-14,36),
        s!(-12,18),
        s!(-5,1),
        s!(0,-9),
        s!(9,-14),
        s!(-6,-13),
    ],
    passers_enemy_king: [
        s!(-61,-14),
        s!(-1,-24),
        s!(11,-14),
        s!(0,11),
        s!(-7,28),
        s!(-6,36),
        s!(-21,49),
        s!(-21,38),
    ],
    pawn_attacks: [
        s!(8,-15),
        s!(55,26),
        s!(50,53),
        s!(68,26),
        s!(60,-16),
        s!(0,0),
    ],
    knight_attacks: [
        s!(-4,13),
        s!(-2,35),
        s!(24,35),
        s!(56,19),
        s!(44,-19),
        s!(0,0),
    ],
    bishop_attacks: [
        s!(-2,11),
        s!(16,26),
        s!(0,0),
        s!(42,29),
        s!(54,89),
        s!(0,0),
    ],
    rook_attacks: [
        s!(-6,13),
        s!(3,19),
        s!(10,13),
        s!(-22,-16),
        s!(60,24),
        s!(0,0),
    ],
    queen_attacks: [
        s!(-3,12),
        s!(1,8),
        s!(1,18),
        s!(2,-4),
        s!(41,9),
        s!(0,0),
    ],
    knight_outposts: s!(17,17),
    bishop_outposts: s!(27,10),
    knight_shelter: s!(4,12),
    bishop_shelter: s!(7,0),
    tempo: s!(24,26),
    safe_checks: [
        s!(6,3),
        s!(73,-2),
        s!(16,17),
        s!(57,3),
        s!(19,30),
        s!(0,0),
    ],
    unsafe_checks: [
        s!(-1,7),
        s!(10,-1),
        s!(12,12),
        s!(20,-5),
        s!(3,12),
        s!(0,0),
    ],
    bad_bishops: [
        s!(78,135),
        s!(80,129),
        s!(78,121),
        s!(75,114),
        s!(71,106),
        s!(70,94),
        s!(67,86),
        s!(64,73),
        s!(54,58),
    ],
    square_rule: s!(10,230),
    free_passer: [
        s!(0,0),
        s!(3,2),
        s!(2,-5),
        s!(-6,10),
        s!(-16,44),
        s!(-33,142),
        s!(-20,224),
        s!(0,0),
    ],
    protected_passer: [
        s!(0,0),
        s!(25,-25),
        s!(18,-11),
        s!(17,-1),
        s!(18,15),
        s!(29,22),
        s!(38,-10),
        s!(0,0),
    ],
    bishop_long_diagonal: s!(11,6),
    push_threats: [
        s!(0,0),
        s!(13,17),
        s!(14,16),
        s!(20,11),
        s!(14,10),
        s!(70,12),
    ],
};