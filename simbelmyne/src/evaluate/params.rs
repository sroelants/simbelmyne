use crate::evaluate::S;
use crate::s;
use super::tuner::EvalWeights;

pub const PARAMS: EvalWeights = EvalWeights {
    piece_values: [
        s!(37,50),
        s!(125,132),
        s!(91,96),
        s!(161,221),
        s!(325,377),
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
        s!(23,92),
        s!(37,81),
        s!(26,89),
        s!(35,73),
        s!(34,74),
        s!(32,78),
        s!(13,85),
        s!(-7,96),
        s!(18,59),
        s!(4,58),
        s!(29,47),
        s!(25,54),
        s!(43,36),
        s!(65,38),
        s!(47,62),
        s!(30,62),
        s!(6,55),
        s!(9,45),
        s!(12,44),
        s!(14,36),
        s!(28,35),
        s!(31,35),
        s!(18,42),
        s!(19,41),
        s!(6,41),
        s!(6,44),
        s!(16,38),
        s!(23,39),
        s!(26,38),
        s!(24,36),
        s!(15,37),
        s!(14,32),
        s!(-2,37),
        s!(0,35),
        s!(9,37),
        s!(10,42),
        s!(16,41),
        s!(12,40),
        s!(20,28),
        s!(9,28),
        s!(7,42),
        s!(7,41),
        s!(16,44),
        s!(11,48),
        s!(21,53),
        s!(29,46),
        s!(34,33),
        s!(14,32),
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
        s!(-28,56),
        s!(-28,82),
        s!(-7,94),
        s!(35,80),
        s!(56,84),
        s!(13,68),
        s!(-19,82),
        s!(-1,41),
        s!(45,92),
        s!(56,94),
        s!(55,88),
        s!(65,89),
        s!(62,79),
        s!(88,73),
        s!(64,90),
        s!(66,77),
        s!(59,88),
        s!(64,88),
        s!(62,92),
        s!(83,88),
        s!(88,85),
        s!(101,71),
        s!(68,82),
        s!(72,79),
        s!(71,94),
        s!(73,89),
        s!(86,89),
        s!(87,95),
        s!(95,94),
        s!(95,90),
        s!(87,90),
        s!(97,86),
        s!(65,93),
        s!(75,85),
        s!(76,94),
        s!(84,93),
        s!(81,102),
        s!(90,88),
        s!(84,89),
        s!(80,92),
        s!(47,80),
        s!(57,79),
        s!(58,80),
        s!(60,91),
        s!(69,90),
        s!(63,76),
        s!(75,77),
        s!(61,83),
        s!(44,81),
        s!(51,84),
        s!(51,80),
        s!(60,84),
        s!(60,81),
        s!(59,80),
        s!(62,79),
        s!(62,91),
        s!(19,93),
        s!(46,76),
        s!(40,82),
        s!(50,86),
        s!(55,88),
        s!(54,76),
        s!(48,84),
        s!(49,89),
    ],
    bishop_psqt: [
        s!(17,71),
        s!(4,83),
        s!(4,75),
        s!(-20,83),
        s!(-17,80),
        s!(-5,73),
        s!(20,72),
        s!(0,65),
        s!(46,66),
        s!(27,66),
        s!(44,69),
        s!(29,72),
        s!(43,68),
        s!(47,65),
        s!(25,67),
        s!(36,66),
        s!(49,77),
        s!(54,71),
        s!(38,69),
        s!(58,70),
        s!(51,70),
        s!(79,68),
        s!(61,68),
        s!(71,76),
        s!(45,74),
        s!(51,74),
        s!(60,74),
        s!(62,82),
        s!(70,76),
        s!(63,74),
        s!(65,73),
        s!(48,73),
        s!(49,69),
        s!(50,75),
        s!(55,75),
        s!(70,77),
        s!(67,72),
        s!(64,73),
        s!(54,72),
        s!(69,62),
        s!(49,71),
        s!(61,69),
        s!(52,66),
        s!(53,72),
        s!(58,76),
        s!(53,63),
        s!(67,64),
        s!(61,61),
        s!(62,67),
        s!(52,54),
        s!(62,56),
        s!(50,70),
        s!(53,67),
        s!(59,63),
        s!(62,53),
        s!(65,56),
        s!(59,60),
        s!(64,72),
        s!(52,70),
        s!(48,67),
        s!(57,67),
        s!(42,77),
        s!(61,62),
        s!(74,44),
    ],
    rook_psqt: [
        s!(77,196),
        s!(73,199),
        s!(67,204),
        s!(61,203),
        s!(68,198),
        s!(89,195),
        s!(84,197),
        s!(93,193),
        s!(109,174),
        s!(111,178),
        s!(120,179),
        s!(133,170),
        s!(118,172),
        s!(131,172),
        s!(126,173),
        s!(132,169),
        s!(82,191),
        s!(106,185),
        s!(99,187),
        s!(98,183),
        s!(118,177),
        s!(123,174),
        s!(145,171),
        s!(112,174),
        s!(86,192),
        s!(101,186),
        s!(101,189),
        s!(100,185),
        s!(104,178),
        s!(119,170),
        s!(115,177),
        s!(105,176),
        s!(80,188),
        s!(86,187),
        s!(90,184),
        s!(94,182),
        s!(100,180),
        s!(95,179),
        s!(112,173),
        s!(93,175),
        s!(75,181),
        s!(78,177),
        s!(82,175),
        s!(86,175),
        s!(91,171),
        s!(95,164),
        s!(114,154),
        s!(98,158),
        s!(76,169),
        s!(79,172),
        s!(88,171),
        s!(89,169),
        s!(93,164),
        s!(97,160),
        s!(104,155),
        s!(83,159),
        s!(85,172),
        s!(89,171),
        s!(91,175),
        s!(97,168),
        s!(100,162),
        s!(93,167),
        s!(98,163),
        s!(85,163),
    ],
    queen_psqt: [
        s!(179,320),
        s!(173,321),
        s!(180,335),
        s!(200,328),
        s!(181,340),
        s!(188,341),
        s!(224,296),
        s!(194,321),
        s!(233,291),
        s!(221,296),
        s!(222,316),
        s!(209,326),
        s!(195,347),
        s!(222,324),
        s!(228,305),
        s!(263,303),
        s!(213,316),
        s!(214,312),
        s!(207,332),
        s!(211,332),
        s!(199,346),
        s!(222,328),
        s!(226,313),
        s!(223,320),
        s!(209,322),
        s!(213,327),
        s!(212,333),
        s!(207,342),
        s!(208,343),
        s!(221,327),
        s!(228,334),
        s!(227,323),
        s!(211,316),
        s!(210,337),
        s!(209,336),
        s!(213,346),
        s!(218,339),
        s!(218,335),
        s!(228,324),
        s!(229,326),
        s!(211,305),
        s!(213,319),
        s!(210,333),
        s!(209,330),
        s!(212,332),
        s!(216,327),
        s!(228,310),
        s!(226,307),
        s!(217,293),
        s!(214,299),
        s!(216,303),
        s!(218,312),
        s!(217,313),
        s!(221,292),
        s!(227,269),
        s!(242,255),
        s!(209,299),
        s!(213,296),
        s!(214,302),
        s!(212,314),
        s!(217,296),
        s!(205,295),
        s!(216,285),
        s!(222,276),
    ],
    king_psqt: [
        s!(70,-77),
        s!(74,-41),
        s!(71,-26),
        s!(15,1),
        s!(41,-8),
        s!(-4,7),
        s!(35,4),
        s!(118,-79),
        s!(-54,6),
        s!(16,27),
        s!(10,32),
        s!(110,11),
        s!(44,31),
        s!(22,50),
        s!(7,47),
        s!(-9,22),
        s!(-43,9),
        s!(50,26),
        s!(7,43),
        s!(-12,54),
        s!(25,56),
        s!(68,50),
        s!(11,48),
        s!(4,15),
        s!(-1,-7),
        s!(6,19),
        s!(-24,42),
        s!(-58,59),
        s!(-50,60),
        s!(-20,50),
        s!(-7,31),
        s!(-72,16),
        s!(-11,-17),
        s!(15,6),
        s!(-2,29),
        s!(-42,48),
        s!(-27,47),
        s!(6,29),
        s!(0,15),
        s!(-68,6),
        s!(0,-19),
        s!(35,-1),
        s!(8,17),
        s!(1,27),
        s!(1,27),
        s!(7,18),
        s!(13,5),
        s!(-22,-5),
        s!(44,-27),
        s!(22,-1),
        s!(18,6),
        s!(-9,15),
        s!(-8,18),
        s!(2,11),
        s!(25,-3),
        s!(24,-19),
        s!(13,-49),
        s!(18,-22),
        s!(7,-5),
        s!(-46,8),
        s!(-12,-4),
        s!(-35,6),
        s!(0,-14),
        s!(12,-50),
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
        s!(23,92),
        s!(37,81),
        s!(26,89),
        s!(35,73),
        s!(34,74),
        s!(32,78),
        s!(13,85),
        s!(-7,96),
        s!(12,53),
        s!(39,52),
        s!(13,41),
        s!(11,19),
        s!(2,33),
        s!(6,40),
        s!(-6,36),
        s!(-38,55),
        s!(10,32),
        s!(12,30),
        s!(17,14),
        s!(5,13),
        s!(-3,12),
        s!(4,15),
        s!(-7,30),
        s!(-11,34),
        s!(-7,23),
        s!(-1,12),
        s!(-13,6),
        s!(-8,-5),
        s!(-16,-3),
        s!(-10,3),
        s!(-9,17),
        s!(-11,18),
        s!(-12,3),
        s!(-14,5),
        s!(-18,-6),
        s!(-14,-17),
        s!(-17,-16),
        s!(-15,-13),
        s!(-17,7),
        s!(-3,-3),
        s!(-24,3),
        s!(-13,-4),
        s!(-21,-14),
        s!(-20,-17),
        s!(-10,-35),
        s!(-20,-24),
        s!(-9,-14),
        s!(-11,-8),
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
        s!(59,68),
        s!(38,46),
        s!(59,73),
        s!(69,92),
        s!(79,100),
        s!(84,112),
        s!(92,114),
        s!(99,121),
        s!(109,115),
    ],
    bishop_mobility: [
        s!(25,31),
        s!(9,5),
        s!(33,37),
        s!(42,56),
        s!(52,65),
        s!(59,73),
        s!(62,82),
        s!(67,84),
        s!(68,88),
        s!(72,88),
        s!(75,90),
        s!(87,82),
        s!(91,86),
        s!(105,72),
    ],
    rook_mobility: [
        s!(68,93),
        s!(62,161),
        s!(85,141),
        s!(93,159),
        s!(97,169),
        s!(100,175),
        s!(102,181),
        s!(104,186),
        s!(107,189),
        s!(109,192),
        s!(111,196),
        s!(112,201),
        s!(112,205),
        s!(112,209),
        s!(114,206),
    ],
    queen_mobility: [
        s!(213,362),
        s!(219,332),
        s!(220,301),
        s!(213,281),
        s!(221,261),
        s!(242,277),
        s!(247,300),
        s!(248,324),
        s!(251,337),
        s!(254,350),
        s!(257,352),
        s!(260,354),
        s!(262,359),
        s!(266,358),
        s!(268,359),
        s!(269,362),
        s!(271,361),
        s!(274,361),
        s!(275,362),
        s!(279,357),
        s!(288,350),
        s!(296,334),
        s!(310,326),
        s!(330,302),
        s!(331,302),
        s!(439,227),
        s!(350,259),
        s!(241,298),
    ],
    virtual_mobility: [
        s!(2,-2),
        s!(6,-4),
        s!(4,0),
        s!(2,-2),
        s!(1,0),
        s!(3,0),
        s!(3,0),
        s!(1,0),
        s!(1,4),
        s!(-1,5),
        s!(-4,7),
        s!(-8,9),
        s!(-8,10),
        s!(-13,11),
        s!(-14,10),
        s!(-18,12),
        s!(-23,9),
        s!(-25,10),
        s!(-21,7),
        s!(-6,0),
        s!(-2,-2),
        s!(4,-5),
        s!(-6,-7),
        s!(19,-14),
        s!(4,-18),
        s!(50,-26),
        s!(46,-28),
        s!(93,-33),
    ],
    king_zone: [
        s!(22,-12),
        s!(23,-2),
        s!(13,-1),
        s!(0,-5),
        s!(-38,14),
        s!(-79,27),
        s!(-137,55),
        s!(-195,85),
        s!(-250,79),
        s!(-317,115),
        s!(-241,20),
        s!(-145,-21),
        s!(-103,-44),
        s!(-9,-3),
        s!(0,0),
        s!(0,0),
    ],
    isolated_pawn: [
        s!(0,0),
        s!(-6,-8),
        s!(-3,-6),
        s!(-6,-7),
        s!(3,-11),
        s!(14,-12),
        s!(39,-13),
        s!(0,0),
    ],
    doubled_pawn: [
        s!(0,0),
        s!(-1,-18),
        s!(-7,-14),
        s!(-7,-22),
        s!(24,-46),
        s!(-95,-5),
        s!(0,0),
        s!(0,0),
    ],
    protected_pawn: [
        s!(0,0),
        s!(0,0),
        s!(17,11),
        s!(9,8),
        s!(14,11),
        s!(26,38),
        s!(136,44),
        s!(0,0),
    ],
    phalanx_pawn: [
        s!(0,0),
        s!(3,0),
        s!(4,4),
        s!(8,7),
        s!(23,24),
        s!(50,85),
        s!(-42,172),
        s!(0,0),
    ],
    bishop_pair: s!(18,47),
    rook_open_file: s!(9,-1),
    rook_semiopen_file: s!(15,1),
    connected_rooks: s!(-3,3),
    major_on_seventh: s!(-24,25),
    queen_open_file: s!(-12,19),
    queen_semiopen_file: s!(4,7),
    pawn_shield: [
        s!(20,-12),
        s!(14,-5),
        s!(12,-8),
    ],
    pawn_storm: [
        s!(6,-22),
        s!(0,-4),
        s!(-7,-2),
    ],
    passers_friendly_king: [
        s!(-15,38),
        s!(-13,22),
        s!(-8,6),
        s!(-4,-2),
        s!(-1,-5),
        s!(11,-8),
        s!(2,-10),
    ],
    passers_enemy_king: [
        s!(-52,-7),
        s!(0,-9),
        s!(-2,8),
        s!(-4,18),
        s!(-6,24),
        s!(-6,28),
        s!(-17,27),
    ],
    pawn_attacks: [
        s!(7,-14),
        s!(57,20),
        s!(52,44),
        s!(70,19),
        s!(61,-21),
        s!(0,0),
    ],
    knight_attacks: [
        s!(-4,12),
        s!(-3,34),
        s!(24,29),
        s!(57,13),
        s!(44,-23),
        s!(0,0),
    ],
    bishop_attacks: [
        s!(-2,10),
        s!(16,21),
        s!(0,0),
        s!(43,22),
        s!(56,73),
        s!(0,0),
    ],
    rook_attacks: [
        s!(-6,12),
        s!(3,17),
        s!(10,12),
        s!(-28,-2),
        s!(63,11),
        s!(0,0),
    ],
    queen_attacks: [
        s!(-3,9),
        s!(1,7),
        s!(0,19),
        s!(2,-5),
        s!(52,22),
        s!(0,0),
    ],
    knight_outposts: s!(17,15),
    bishop_outposts: s!(28,8),
    knight_shelter: s!(4,11),
    bishop_shelter: s!(7,0),
    tempo: s!(24,22),
    safe_checks: [
        s!(8,2),
        s!(76,-3),
        s!(17,12),
        s!(61,-2),
        s!(24,17),
        s!(0,0),
    ],
    unsafe_checks: [
        s!(0,6),
        s!(10,-1),
        s!(13,9),
        s!(19,-5),
        s!(4,8),
        s!(0,0),
    ],
    bad_bishops: [
        s!(89,98),
        s!(88,97),
        s!(86,92),
        s!(82,86),
        s!(78,80),
        s!(76,70),
        s!(72,64),
        s!(68,52),
        s!(57,43),
    ],
    square_rule: s!(-27,161),
    free_passer: [
        s!(0,0),
        s!(5,0),
        s!(3,-4),
        s!(-5,9),
        s!(-16,37),
        s!(-34,112),
        s!(-8,157),
        s!(0,0),
    ],
    protected_passer: [
        s!(0,0),
        s!(26,-26),
        s!(21,-13),
        s!(19,-3),
        s!(21,8),
        s!(35,12),
        s!(40,-9),
        s!(0,0),
    ],
    bishop_long_diagonal: s!(11,5),
    push_threats: [
        s!(0,0),
        s!(13,15),
        s!(14,13),
        s!(22,7),
        s!(14,8),
        s!(69,10),
    ],
};