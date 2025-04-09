use crate::evaluate::S;
use crate::s;
use super::tuner::EvalWeights;

pub const PARAMS: EvalWeights = EvalWeights {
    piece_values: [
        s!(33,63),
        s!(115,179),
        s!(83,129),
        s!(141,290),
        s!(307,486),
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
        s!(21,118),
        s!(37,101),
        s!(28,110),
        s!(39,91),
        s!(37,93),
        s!(35,96),
        s!(17,104),
        s!(-6,123),
        s!(19,74),
        s!(4,71),
        s!(29,62),
        s!(26,68),
        s!(42,51),
        s!(66,54),
        s!(46,80),
        s!(29,82),
        s!(7,68),
        s!(9,57),
        s!(12,55),
        s!(14,47),
        s!(28,47),
        s!(31,47),
        s!(18,56),
        s!(19,56),
        s!(6,51),
        s!(5,54),
        s!(14,49),
        s!(22,50),
        s!(25,50),
        s!(22,48),
        s!(14,49),
        s!(14,43),
        s!(-2,45),
        s!(-1,44),
        s!(7,47),
        s!(8,53),
        s!(13,53),
        s!(9,51),
        s!(18,38),
        s!(9,37),
        s!(7,50),
        s!(6,50),
        s!(14,55),
        s!(9,58),
        s!(18,66),
        s!(28,58),
        s!(32,43),
        s!(14,41),
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
        s!(-28,61),
        s!(-37,108),
        s!(-5,116),
        s!(37,100),
        s!(49,110),
        s!(18,90),
        s!(-31,111),
        s!(2,38),
        s!(42,115),
        s!(50,120),
        s!(51,115),
        s!(61,119),
        s!(59,108),
        s!(81,104),
        s!(53,119),
        s!(62,96),
        s!(54,112),
        s!(58,115),
        s!(57,121),
        s!(76,116),
        s!(85,111),
        s!(93,98),
        s!(62,109),
        s!(64,102),
        s!(65,119),
        s!(68,114),
        s!(79,118),
        s!(81,126),
        s!(89,123),
        s!(89,119),
        s!(81,114),
        s!(91,111),
        s!(59,118),
        s!(69,111),
        s!(71,123),
        s!(79,122),
        s!(76,131),
        s!(85,115),
        s!(79,114),
        s!(74,118),
        s!(41,102),
        s!(51,103),
        s!(53,103),
        s!(55,117),
        s!(63,115),
        s!(58,98),
        s!(68,99),
        s!(56,104),
        s!(38,106),
        s!(45,108),
        s!(46,103),
        s!(54,107),
        s!(55,104),
        s!(53,104),
        s!(57,100),
        s!(56,115),
        s!(13,117),
        s!(40,95),
        s!(33,107),
        s!(43,108),
        s!(49,110),
        s!(48,96),
        s!(42,103),
        s!(44,111),
    ],
    bishop_psqt: [
        s!(16,89),
        s!(8,102),
        s!(7,95),
        s!(-13,104),
        s!(-10,98),
        s!(2,93),
        s!(22,92),
        s!(1,83),
        s!(46,84),
        s!(26,83),
        s!(45,87),
        s!(30,95),
        s!(43,91),
        s!(47,88),
        s!(23,87),
        s!(34,84),
        s!(48,98),
        s!(52,91),
        s!(37,86),
        s!(56,90),
        s!(49,93),
        s!(76,90),
        s!(58,90),
        s!(68,97),
        s!(43,94),
        s!(49,95),
        s!(57,95),
        s!(59,105),
        s!(66,97),
        s!(62,94),
        s!(62,95),
        s!(46,94),
        s!(47,89),
        s!(46,96),
        s!(53,97),
        s!(66,99),
        s!(65,94),
        s!(61,95),
        s!(52,92),
        s!(67,78),
        s!(47,89),
        s!(58,89),
        s!(49,86),
        s!(51,95),
        s!(55,98),
        s!(49,80),
        s!(63,81),
        s!(59,76),
        s!(58,86),
        s!(50,71),
        s!(59,76),
        s!(47,92),
        s!(50,89),
        s!(55,84),
        s!(59,71),
        s!(62,74),
        s!(56,79),
        s!(62,85),
        s!(49,89),
        s!(44,87),
        s!(53,87),
        s!(38,97),
        s!(57,80),
        s!(71,58),
    ],
    rook_psqt: [
        s!(68,250),
        s!(58,258),
        s!(56,263),
        s!(56,255),
        s!(62,251),
        s!(83,252),
        s!(78,257),
        s!(82,250),
        s!(101,222),
        s!(101,227),
        s!(107,230),
        s!(123,214),
        s!(110,217),
        s!(127,219),
        s!(121,221),
        s!(126,214),
        s!(68,251),
        s!(91,245),
        s!(86,249),
        s!(90,241),
        s!(110,233),
        s!(117,227),
        s!(139,221),
        s!(104,225),
        s!(74,251),
        s!(87,245),
        s!(89,248),
        s!(88,242),
        s!(94,232),
        s!(108,225),
        s!(107,229),
        s!(96,226),
        s!(69,242),
        s!(73,243),
        s!(79,238),
        s!(82,235),
        s!(89,232),
        s!(85,230),
        s!(104,221),
        s!(84,224),
        s!(65,232),
        s!(67,229),
        s!(73,226),
        s!(77,224),
        s!(81,220),
        s!(86,211),
        s!(107,197),
        s!(91,199),
        s!(67,216),
        s!(70,223),
        s!(79,220),
        s!(80,217),
        s!(84,210),
        s!(88,207),
        s!(96,200),
        s!(75,202),
        s!(76,219),
        s!(79,220),
        s!(82,224),
        s!(87,216),
        s!(91,210),
        s!(84,213),
        s!(89,210),
        s!(79,204),
    ],
    queen_psqt: [
        s!(167,418),
        s!(154,426),
        s!(177,425),
        s!(207,404),
        s!(180,427),
        s!(173,449),
        s!(213,383),
        s!(183,421),
        s!(234,367),
        s!(220,373),
        s!(224,393),
        s!(210,407),
        s!(200,430),
        s!(225,404),
        s!(227,385),
        s!(263,383),
        s!(211,397),
        s!(212,392),
        s!(206,416),
        s!(213,413),
        s!(201,428),
        s!(222,414),
        s!(225,396),
        s!(225,400),
        s!(206,407),
        s!(209,414),
        s!(211,418),
        s!(204,429),
        s!(206,431),
        s!(220,411),
        s!(224,425),
        s!(224,408),
        s!(208,400),
        s!(206,426),
        s!(206,422),
        s!(211,432),
        s!(216,425),
        s!(215,421),
        s!(224,410),
        s!(225,410),
        s!(207,389),
        s!(209,404),
        s!(206,420),
        s!(206,416),
        s!(208,418),
        s!(212,414),
        s!(224,394),
        s!(222,388),
        s!(212,378),
        s!(209,384),
        s!(212,386),
        s!(213,396),
        s!(212,397),
        s!(216,373),
        s!(224,343),
        s!(240,324),
        s!(204,384),
        s!(208,382),
        s!(209,387),
        s!(207,397),
        s!(212,378),
        s!(199,378),
        s!(211,365),
        s!(216,362),
    ],
    king_psqt: [
        s!(25,-90),
        s!(2,-31),
        s!(11,-16),
        s!(-1,10),
        s!(10,-3),
        s!(-29,22),
        s!(19,14),
        s!(77,-101),
        s!(-49,6),
        s!(-1,34),
        s!(-18,51),
        s!(62,25),
        s!(42,36),
        s!(28,53),
        s!(27,45),
        s!(4,16),
        s!(-40,8),
        s!(36,26),
        s!(4,51),
        s!(-8,64),
        s!(34,64),
        s!(68,49),
        s!(46,35),
        s!(21,5),
        s!(-14,-7),
        s!(5,18),
        s!(-23,48),
        s!(-48,69),
        s!(-35,67),
        s!(-1,54),
        s!(15,25),
        s!(-52,14),
        s!(-18,-17),
        s!(9,7),
        s!(-9,37),
        s!(-42,58),
        s!(-29,55),
        s!(4,34),
        s!(4,16),
        s!(-59,7),
        s!(-6,-18),
        s!(30,0),
        s!(2,23),
        s!(-5,35),
        s!(-5,34),
        s!(4,22),
        s!(13,5),
        s!(-18,-7),
        s!(45,-31),
        s!(21,0),
        s!(15,9),
        s!(-13,21),
        s!(-12,23),
        s!(0,14),
        s!(26,-6),
        s!(27,-25),
        s!(14,-59),
        s!(18,-25),
        s!(6,-3),
        s!(-47,11),
        s!(-12,-6),
        s!(-36,8),
        s!(1,-18),
        s!(16,-61),
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
        s!(21,118),
        s!(37,101),
        s!(28,110),
        s!(39,91),
        s!(37,93),
        s!(35,96),
        s!(17,104),
        s!(-6,123),
        s!(10,67),
        s!(39,64),
        s!(18,42),
        s!(17,18),
        s!(7,33),
        s!(12,40),
        s!(-1,46),
        s!(-32,64),
        s!(8,42),
        s!(12,38),
        s!(19,14),
        s!(9,10),
        s!(0,9),
        s!(6,16),
        s!(-2,35),
        s!(-9,42),
        s!(-8,30),
        s!(-2,17),
        s!(-11,5),
        s!(-6,-11),
        s!(-15,-5),
        s!(-8,3),
        s!(-7,21),
        s!(-13,26),
        s!(-14,5),
        s!(-17,6),
        s!(-18,-9),
        s!(-14,-24),
        s!(-18,-20),
        s!(-14,-19),
        s!(-21,5),
        s!(-6,-6),
        s!(-26,9),
        s!(-15,-2),
        s!(-22,-17),
        s!(-22,-19),
        s!(-13,-41),
        s!(-22,-29),
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
        s!(53,96),
        s!(34,70),
        s!(54,104),
        s!(63,125),
        s!(72,135),
        s!(76,147),
        s!(84,150),
        s!(90,157),
        s!(100,151),
    ],
    bishop_mobility: [
        s!(22,49),
        s!(8,19),
        s!(31,58),
        s!(40,79),
        s!(49,89),
        s!(55,97),
        s!(58,106),
        s!(62,109),
        s!(63,114),
        s!(67,115),
        s!(69,117),
        s!(81,109),
        s!(86,113),
        s!(92,100),
    ],
    rook_mobility: [
        s!(64,124),
        s!(53,219),
        s!(77,195),
        s!(84,215),
        s!(89,226),
        s!(92,233),
        s!(93,238),
        s!(95,243),
        s!(97,246),
        s!(99,250),
        s!(101,255),
        s!(101,260),
        s!(102,264),
        s!(101,269),
        s!(103,269),
    ],
    queen_mobility: [
        s!(216,458),
        s!(205,438),
        s!(207,397),
        s!(205,362),
        s!(213,342),
        s!(231,376),
        s!(237,399),
        s!(237,425),
        s!(240,437),
        s!(242,451),
        s!(246,452),
        s!(249,455),
        s!(251,459),
        s!(254,457),
        s!(256,458),
        s!(257,462),
        s!(259,460),
        s!(261,461),
        s!(263,462),
        s!(266,455),
        s!(276,445),
        s!(285,426),
        s!(298,413),
        s!(309,392),
        s!(314,383),
        s!(410,276),
        s!(372,278),
        s!(312,308),
    ],
    virtual_mobility: [
        s!(2,-7),
        s!(7,-8),
        s!(5,-3),
        s!(4,-5),
        s!(2,-2),
        s!(4,-3),
        s!(3,-3),
        s!(2,-3),
        s!(1,2),
        s!(-2,3),
        s!(-5,6),
        s!(-9,8),
        s!(-10,10),
        s!(-15,12),
        s!(-16,11),
        s!(-20,13),
        s!(-24,11),
        s!(-25,12),
        s!(-19,9),
        s!(-7,2),
        s!(-3,0),
        s!(0,-2),
        s!(-3,-4),
        s!(14,-12),
        s!(4,-13),
        s!(40,-26),
        s!(49,-28),
        s!(76,-30),
    ],
    king_zone: [
        s!(21,-12),
        s!(21,0),
        s!(12,0),
        s!(-1,-3),
        s!(-37,15),
        s!(-72,27),
        s!(-125,56),
        s!(-165,67),
        s!(-216,53),
        s!(-262,51),
        s!(-223,-16),
        s!(-141,-24),
        s!(-99,-40),
        s!(-12,-3),
        s!(0,0),
        s!(0,0),
    ],
    isolated_pawn: [
        s!(0,0),
        s!(-6,-11),
        s!(-3,-8),
        s!(-5,-10),
        s!(3,-13),
        s!(14,-13),
        s!(42,-19),
        s!(0,0),
    ],
    doubled_pawn: [
        s!(0,0),
        s!(-1,-22),
        s!(-7,-18),
        s!(-6,-28),
        s!(24,-57),
        s!(-104,2),
        s!(0,0),
        s!(0,0),
    ],
    protected_pawn: [
        s!(0,0),
        s!(0,0),
        s!(16,11),
        s!(8,7),
        s!(12,10),
        s!(21,41),
        s!(142,39),
        s!(0,0),
    ],
    phalanx_pawn: [
        s!(0,0),
        s!(3,0),
        s!(4,5),
        s!(8,8),
        s!(21,30),
        s!(46,115),
        s!(-97,264),
        s!(0,0),
    ],
    bishop_pair: s!(17,56),
    rook_open_file: s!(9,-1),
    rook_semiopen_file: s!(15,6),
    connected_rooks: s!(-2,0),
    major_on_seventh: s!(-29,32),
    queen_open_file: s!(-14,28),
    queen_semiopen_file: s!(3,11),
    pawn_shield: [
        s!(20,-13),
        s!(15,-6),
        s!(12,-10),
    ],
    pawn_storm: [
        s!(-18,-4),
        s!(0,-5),
        s!(-8,-2),
    ],
    passers_friendly_king: [
        s!(-16,48),
        s!(-12,28),
        s!(-7,7),
        s!(-3,-3),
        s!(-1,-8),
        s!(9,-12),
        s!(2,-16),
    ],
    passers_enemy_king: [
        s!(-57,7),
        s!(0,-15),
        s!(-3,8),
        s!(-5,21),
        s!(-5,27),
        s!(-3,32),
        s!(-14,30),
    ],
    unsafe_threats: [
        [
            s!(9,-16),
            s!(56,31),
            s!(60,70),
            s!(65,35),
            s!(62,-8),
            s!(0,0),
        ],
        [
            s!(-5,6),
            s!(-3,25),
            s!(21,32),
            s!(47,34),
            s!(43,8),
            s!(0,0),
        ],
        [
            s!(-3,0),
            s!(13,22),
            s!(-2,-11),
            s!(34,46),
            s!(44,142),
            s!(0,0),
        ],
        [
            s!(-9,-5),
            s!(-2,0),
            s!(1,-13),
            s!(-29,-36),
            s!(43,65),
            s!(0,0),
        ],
        [
            s!(-5,10),
            s!(-1,-6),
            s!(-2,-4),
            s!(0,-27),
            s!(10,-1),
            s!(0,0),
        ],
        [
            s!(-30,19),
            s!(-30,13),
            s!(-32,18),
            s!(2,-17),
            s!(0,0),
            s!(0,0),
        ],
    ],
    safe_threats: [
        [
            s!(2,-15),
            s!(53,26),
            s!(48,50),
            s!(69,25),
            s!(60,-17),
            s!(0,0),
        ],
        [
            s!(-1,19),
            s!(-2,20),
            s!(28,37),
            s!(61,13),
            s!(44,-27),
            s!(0,0),
        ],
        [
            s!(0,19),
            s!(24,27),
            s!(2,11),
            s!(46,25),
            s!(61,76),
            s!(0,0),
        ],
        [
            s!(0,19),
            s!(17,30),
            s!(22,28),
            s!(-19,-13),
            s!(65,13),
            s!(0,0),
        ],
        [
            s!(-2,12),
            s!(7,19),
            s!(4,32),
            s!(5,0),
            s!(17,10),
            s!(0,0),
        ],
        [
            s!(-24,48),
            s!(-17,41),
            s!(-8,32),
            s!(19,4),
            s!(0,0),
            s!(0,0),
        ],
    ],
    knight_outposts: s!(13,13),
    bishop_outposts: s!(22,7),
    knight_shelter: s!(5,12),
    bishop_shelter: s!(7,0),
    tempo: s!(24,29),
    safe_checks: [
        s!(7,2),
        s!(73,-1),
        s!(16,16),
        s!(56,3),
        s!(20,29),
        s!(0,0),
    ],
    unsafe_checks: [
        s!(0,7),
        s!(10,-1),
        s!(12,11),
        s!(21,-6),
        s!(3,11),
        s!(0,0),
    ],
    bad_bishops: [
        s!(77,136),
        s!(79,130),
        s!(77,122),
        s!(74,114),
        s!(70,104),
        s!(69,92),
        s!(66,81),
        s!(63,65),
        s!(54,47),
    ],
    square_rule: s!(-8,225),
    free_passer: [
        s!(0,0),
        s!(2,1),
        s!(1,-3),
        s!(-7,13),
        s!(-18,49),
        s!(-38,147),
        s!(-24,227),
        s!(0,0),
    ],
    protected_passer: [
        s!(0,0),
        s!(27,-33),
        s!(19,-13),
        s!(17,-1),
        s!(18,15),
        s!(29,25),
        s!(37,-3),
        s!(0,0),
    ],
    bishop_long_diagonal: s!(11,8),
    push_threats: [
        s!(0,0),
        s!(13,19),
        s!(13,16),
        s!(19,8),
        s!(13,10),
        s!(68,14),
    ],
};