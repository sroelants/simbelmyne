use crate::evaluate::S;
use crate::s;
use super::tuner::EvalWeights;

pub const PARAMS: EvalWeights = EvalWeights {
    piece_values: [
        s!(34,61),
        s!(114,176),
        s!(83,128),
        s!(142,288),
        s!(307,485),
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
        s!(19,117),
        s!(36,98),
        s!(26,108),
        s!(38,87),
        s!(36,90),
        s!(35,93),
        s!(14,102),
        s!(-8,121),
        s!(17,71),
        s!(4,66),
        s!(28,56),
        s!(25,62),
        s!(43,42),
        s!(65,47),
        s!(46,74),
        s!(28,79),
        s!(5,67),
        s!(8,54),
        s!(11,53),
        s!(13,45),
        s!(27,44),
        s!(30,44),
        s!(17,52),
        s!(17,54),
        s!(5,51),
        s!(5,53),
        s!(14,48),
        s!(21,49),
        s!(24,49),
        s!(23,46),
        s!(13,47),
        s!(13,41),
        s!(-3,45),
        s!(-1,43),
        s!(7,46),
        s!(8,52),
        s!(14,52),
        s!(10,50),
        s!(19,36),
        s!(8,36),
        s!(5,51),
        s!(6,49),
        s!(14,54),
        s!(9,58),
        s!(19,65),
        s!(28,58),
        s!(33,42),
        s!(13,41),
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
        s!(-29,65),
        s!(-43,111),
        s!(-13,120),
        s!(28,104),
        s!(45,111),
        s!(9,87),
        s!(-32,109),
        s!(2,41),
        s!(40,118),
        s!(50,121),
        s!(49,114),
        s!(57,117),
        s!(57,102),
        s!(82,96),
        s!(58,115),
        s!(62,97),
        s!(53,114),
        s!(57,114),
        s!(56,119),
        s!(76,115),
        s!(83,109),
        s!(95,93),
        s!(62,107),
        s!(68,101),
        s!(64,122),
        s!(69,115),
        s!(80,115),
        s!(81,124),
        s!(89,121),
        s!(90,117),
        s!(81,116),
        s!(92,113),
        s!(60,120),
        s!(69,110),
        s!(71,121),
        s!(79,120),
        s!(76,130),
        s!(85,113),
        s!(79,115),
        s!(75,119),
        s!(42,104),
        s!(53,103),
        s!(54,102),
        s!(55,116),
        s!(64,115),
        s!(59,97),
        s!(70,100),
        s!(56,106),
        s!(39,107),
        s!(46,110),
        s!(47,103),
        s!(55,107),
        s!(56,103),
        s!(55,104),
        s!(58,101),
        s!(57,117),
        s!(13,121),
        s!(41,99),
        s!(35,108),
        s!(45,111),
        s!(51,113),
        s!(50,98),
        s!(44,106),
        s!(45,116),
    ],
    bishop_psqt: [
        s!(13,91),
        s!(0,106),
        s!(0,98),
        s!(-25,107),
        s!(-19,100),
        s!(-9,95),
        s!(12,96),
        s!(-2,86),
        s!(44,84),
        s!(25,83),
        s!(41,86),
        s!(25,92),
        s!(38,89),
        s!(43,84),
        s!(22,86),
        s!(33,83),
        s!(45,98),
        s!(51,91),
        s!(36,85),
        s!(55,88),
        s!(47,90),
        s!(75,88),
        s!(56,88),
        s!(67,97),
        s!(41,94),
        s!(48,93),
        s!(57,93),
        s!(58,104),
        s!(67,96),
        s!(61,92),
        s!(61,92),
        s!(44,95),
        s!(46,88),
        s!(46,95),
        s!(52,95),
        s!(67,98),
        s!(65,91),
        s!(61,92),
        s!(51,91),
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
        s!(49,70),
        s!(59,73),
        s!(47,88),
        s!(50,86),
        s!(56,82),
        s!(59,69),
        s!(62,74),
        s!(55,80),
        s!(62,85),
        s!(49,89),
        s!(44,87),
        s!(54,87),
        s!(39,97),
        s!(58,80),
        s!(71,60),
    ],
    rook_psqt: [
        s!(72,250),
        s!(60,259),
        s!(56,263),
        s!(57,255),
        s!(62,252),
        s!(81,252),
        s!(75,257),
        s!(82,250),
        s!(103,221),
        s!(104,227),
        s!(110,230),
        s!(125,215),
        s!(112,218),
        s!(125,219),
        s!(119,220),
        s!(124,214),
        s!(69,249),
        s!(92,243),
        s!(84,246),
        s!(86,236),
        s!(108,229),
        s!(112,227),
        s!(135,222),
        s!(102,226),
        s!(74,250),
        s!(88,243),
        s!(89,246),
        s!(88,240),
        s!(93,230),
        s!(106,225),
        s!(105,230),
        s!(96,226),
        s!(70,243),
        s!(74,243),
        s!(79,237),
        s!(84,234),
        s!(90,232),
        s!(85,232),
        s!(104,222),
        s!(84,225),
        s!(66,232),
        s!(68,228),
        s!(73,225),
        s!(77,224),
        s!(82,220),
        s!(87,211),
        s!(107,197),
        s!(91,201),
        s!(67,217),
        s!(70,223),
        s!(79,220),
        s!(80,218),
        s!(85,211),
        s!(88,207),
        s!(96,200),
        s!(76,204),
        s!(76,219),
        s!(80,219),
        s!(82,223),
        s!(88,215),
        s!(92,209),
        s!(85,214),
        s!(90,209),
        s!(79,204),
    ],
    queen_psqt: [
        s!(166,416),
        s!(153,423),
        s!(171,426),
        s!(202,403),
        s!(174,428),
        s!(167,451),
        s!(207,385),
        s!(180,420),
        s!(231,363),
        s!(220,365),
        s!(220,389),
        s!(207,401),
        s!(193,429),
        s!(221,400),
        s!(226,380),
        s!(259,379),
        s!(207,396),
        s!(208,391),
        s!(200,415),
        s!(206,412),
        s!(195,427),
        s!(214,415),
        s!(219,394),
        s!(217,401),
        s!(202,405),
        s!(205,412),
        s!(205,418),
        s!(200,427),
        s!(201,428),
        s!(215,409),
        s!(219,423),
        s!(220,407),
        s!(204,401),
        s!(202,424),
        s!(201,421),
        s!(206,430),
        s!(212,422),
        s!(210,420),
        s!(220,409),
        s!(221,411),
        s!(203,389),
        s!(205,401),
        s!(202,418),
        s!(201,413),
        s!(204,416),
        s!(208,412),
        s!(220,394),
        s!(218,388),
        s!(208,377),
        s!(205,382),
        s!(208,384),
        s!(210,395),
        s!(208,396),
        s!(213,372),
        s!(221,342),
        s!(236,326),
        s!(200,384),
        s!(204,380),
        s!(206,386),
        s!(204,397),
        s!(209,378),
        s!(196,377),
        s!(207,365),
        s!(212,361),
    ],
    king_psqt: [
        s!(28,-95),
        s!(7,-35),
        s!(10,-18),
        s!(-2,8),
        s!(11,-4),
        s!(-24,19),
        s!(19,16),
        s!(79,-104),
        s!(-61,13),
        s!(-19,51),
        s!(-28,60),
        s!(55,30),
        s!(33,44),
        s!(11,68),
        s!(2,66),
        s!(-9,26),
        s!(-53,18),
        s!(14,45),
        s!(-15,65),
        s!(-25,78),
        s!(18,79),
        s!(49,70),
        s!(14,63),
        s!(6,16),
        s!(-18,-4),
        s!(-11,32),
        s!(-37,61),
        s!(-63,82),
        s!(-52,81),
        s!(-17,68),
        s!(-1,40),
        s!(-57,18),
        s!(-18,-18),
        s!(4,12),
        s!(-14,41),
        s!(-48,64),
        s!(-36,62),
        s!(0,40),
        s!(0,21),
        s!(-60,6),
        s!(-4,-21),
        s!(30,0),
        s!(0,24),
        s!(-7,36),
        s!(-6,35),
        s!(3,23),
        s!(13,5),
        s!(-18,-10),
        s!(45,-34),
        s!(21,-2),
        s!(16,8),
        s!(-12,20),
        s!(-11,22),
        s!(1,13),
        s!(26,-8),
        s!(27,-28),
        s!(16,-62),
        s!(19,-27),
        s!(8,-6),
        s!(-45,9),
        s!(-11,-8),
        s!(-35,6),
        s!(2,-20),
        s!(16,-64),
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
        s!(19,117),
        s!(36,98),
        s!(26,108),
        s!(38,87),
        s!(36,90),
        s!(35,93),
        s!(14,102),
        s!(-8,121),
        s!(11,67),
        s!(39,64),
        s!(17,45),
        s!(17,19),
        s!(6,38),
        s!(13,42),
        s!(0,47),
        s!(-31,65),
        s!(8,42),
        s!(11,38),
        s!(19,15),
        s!(9,11),
        s!(0,11),
        s!(6,18),
        s!(-2,37),
        s!(-9,43),
        s!(-8,29),
        s!(-2,17),
        s!(-11,5),
        s!(-6,-10),
        s!(-14,-5),
        s!(-8,3),
        s!(-7,21),
        s!(-13,26),
        s!(-13,4),
        s!(-16,6),
        s!(-17,-9),
        s!(-14,-24),
        s!(-17,-21),
        s!(-14,-19),
        s!(-20,5),
        s!(-6,-6),
        s!(-26,6),
        s!(-16,-3),
        s!(-23,-17),
        s!(-23,-20),
        s!(-13,-41),
        s!(-23,-30),
        s!(-14,-23),
        s!(-14,-11),
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
        s!(54,90),
        s!(33,65),
        s!(53,100),
        s!(62,123),
        s!(72,133),
        s!(77,146),
        s!(85,150),
        s!(92,159),
        s!(101,152),
    ],
    bishop_mobility: [
        s!(24,40),
        s!(8,11),
        s!(30,54),
        s!(39,75),
        s!(49,86),
        s!(55,95),
        s!(58,105),
        s!(62,109),
        s!(63,114),
        s!(67,116),
        s!(69,118),
        s!(79,110),
        s!(84,114),
        s!(88,100),
    ],
    rook_mobility: [
        s!(65,124),
        s!(54,212),
        s!(77,189),
        s!(84,210),
        s!(89,221),
        s!(92,229),
        s!(93,235),
        s!(95,242),
        s!(97,245),
        s!(100,249),
        s!(101,255),
        s!(102,262),
        s!(102,266),
        s!(101,271),
        s!(102,270),
    ],
    queen_mobility: [
        s!(210,451),
        s!(206,436),
        s!(209,394),
        s!(206,363),
        s!(212,343),
        s!(230,374),
        s!(236,396),
        s!(236,422),
        s!(239,434),
        s!(242,449),
        s!(245,451),
        s!(248,453),
        s!(251,458),
        s!(254,457),
        s!(256,458),
        s!(258,462),
        s!(260,460),
        s!(262,462),
        s!(264,462),
        s!(267,456),
        s!(277,447),
        s!(285,428),
        s!(299,415),
        s!(310,393),
        s!(313,388),
        s!(407,283),
        s!(373,280),
        s!(308,317),
    ],
    virtual_mobility: [
        s!(1,0),
        s!(6,-3),
        s!(4,0),
        s!(2,-3),
        s!(1,0),
        s!(3,-1),
        s!(3,-1),
        s!(1,-1),
        s!(0,4),
        s!(-3,5),
        s!(-5,8),
        s!(-9,10),
        s!(-10,12),
        s!(-15,13),
        s!(-16,12),
        s!(-19,14),
        s!(-23,11),
        s!(-24,12),
        s!(-18,8),
        s!(-4,0),
        s!(0,-3),
        s!(5,-7),
        s!(2,-10),
        s!(21,-20),
        s!(12,-23),
        s!(50,-38),
        s!(60,-41),
        s!(86,-44),
    ],
    king_zone: [
        s!(22,-13),
        s!(22,-1),
        s!(12,0),
        s!(-1,-4),
        s!(-38,17),
        s!(-73,29),
        s!(-128,62),
        s!(-168,75),
        s!(-221,66),
        s!(-259,59),
        s!(-221,-11),
        s!(-147,-23),
        s!(-101,-38),
        s!(-10,-3),
        s!(0,0),
        s!(0,0),
    ],
    isolated_pawn: [
        s!(0,0),
        s!(-6,-11),
        s!(-3,-8),
        s!(-5,-10),
        s!(4,-14),
        s!(14,-13),
        s!(42,-17),
        s!(0,0),
    ],
    doubled_pawn: [
        s!(0,0),
        s!(-1,-21),
        s!(-7,-16),
        s!(-6,-26),
        s!(24,-54),
        s!(-105,5),
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
        s!(147,58),
        s!(0,0),
    ],
    phalanx_pawn: [
        s!(0,0),
        s!(3,0),
        s!(4,5),
        s!(8,8),
        s!(22,30),
        s!(45,115),
        s!(-89,258),
        s!(0,0),
    ],
    bishop_pair: s!(17,56),
    rook_open_file: s!(8,-1),
    rook_semiopen_file: s!(15,3),
    connected_rooks: s!(-3,1),
    major_on_seventh: s!(-29,37),
    queen_open_file: s!(-14,28),
    queen_semiopen_file: s!(3,10),
    pawn_shield: [
        s!(20,-14),
        s!(15,-6),
        s!(12,-10),
    ],
    pawn_storm: [
        s!(1,-24),
        s!(0,-3),
        s!(-8,-1),
    ],
    passers_friendly_king: [
        s!(-15,46),
        s!(-12,28),
        s!(-7,8),
        s!(-4,-3),
        s!(-2,-7),
        s!(9,-11),
        s!(1,-15),
    ],
    passers_enemy_king: [
        s!(-48,-11),
        s!(-1,-14),
        s!(-4,9),
        s!(-6,21),
        s!(-5,28),
        s!(-3,33),
        s!(-14,31),
    ],
    pawn_attacks: [
        s!(8,-15),
        s!(55,25),
        s!(50,53),
        s!(68,26),
        s!(60,-16),
        s!(0,0),
    ],
    knight_attacks: [
        s!(-4,13),
        s!(-2,36),
        s!(24,35),
        s!(55,19),
        s!(44,-20),
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
        s!(-6,12),
        s!(3,19),
        s!(10,13),
        s!(-22,-14),
        s!(60,24),
        s!(0,0),
    ],
    queen_attacks: [
        s!(-3,11),
        s!(1,8),
        s!(0,18),
        s!(2,-4),
        s!(47,10),
        s!(0,0),
    ],
    knight_outposts: s!(17,18),
    bishop_outposts: s!(26,11),
    knight_shelter: s!(4,12),
    bishop_shelter: s!(7,0),
    tempo: s!(23,26),
    safe_checks: [
        s!(7,4),
        s!(73,-3),
        s!(16,17),
        s!(57,3),
        s!(20,29),
        s!(0,0),
    ],
    unsafe_checks: [
        s!(0,9),
        s!(10,-1),
        s!(12,11),
        s!(20,-5),
        s!(3,11),
        s!(0,0),
    ],
    bad_bishops: [
        s!(77,135),
        s!(80,130),
        s!(78,121),
        s!(74,114),
        s!(71,106),
        s!(70,94),
        s!(67,86),
        s!(64,72),
        s!(54,58),
    ],
    square_rule: s!(-6,229),
    free_passer: [
        s!(0,0),
        s!(3,1),
        s!(1,-4),
        s!(-7,13),
        s!(-18,49),
        s!(-38,149),
        s!(-21,227),
        s!(0,0),
    ],
    protected_passer: [
        s!(0,0),
        s!(26,-32),
        s!(19,-14),
        s!(18,-1),
        s!(18,14),
        s!(30,22),
        s!(39,-11),
        s!(0,0),
    ],
    bishop_long_diagonal: s!(11,6),
    push_threats: [
        s!(0,0),
        s!(13,17),
        s!(14,16),
        s!(20,11),
        s!(14,11),
        s!(67,14),
    ],
};