use crate::evaluate::S;
use crate::s;

pub const PIECE_VALUES: [S; 6] = [
s!(51,96),
s!(301,366),
s!(315,372),
s!(405,615),
s!(861,1136),
s!(0,0),
];

pub const PAWN_PSQT: [S; 64] = [
s!(0,0),    s!(0,0),    s!(0,0),    s!(0,0),    s!(0,0),    s!(0,0),    s!(0,0),    s!(0,0),    
s!(64,170), s!(76,164), s!(38,166), s!(69,138), s!(57,143), s!(62,143), s!(12,169), s!(0,182),  
s!(18,31),  s!(0,36),   s!(30,25),  s!(24,35),  s!(43,16),  s!(69,17),  s!(46,43),  s!(29,33),  
s!(0,22),   s!(4,17),   s!(4,14),   s!(10,3),   s!(29,4),   s!(27,7),   s!(16,13),  s!(17,6),   
s!(-3,6),   s!(-6,10),  s!(5,4),    s!(13,2),   s!(16,2),   s!(17,4),   s!(6,4),    s!(4,-3),   
s!(-13,2),  s!(-8,1),   s!(-4,4),   s!(0,11),   s!(8,9),    s!(0,8),    s!(16,-5),  s!(1,-7),   
s!(-6,6),   s!(-3,5),   s!(5,9),    s!(1,15),   s!(9,22),   s!(23,13),  s!(26,-3),  s!(3,-4),   
s!(0,0),    s!(0,0),    s!(0,0),    s!(0,0),    s!(0,0),    s!(0,0),    s!(0,0),    s!(0,0),     ];

pub const KNIGHT_PSQT: [S; 64] = [
s!(-81,-38),s!(-82,-5), s!(-63,12), s!(-22,-1), s!(8,0),    s!(-52,-18),s!(-78,-3), s!(-52,-54),
s!(0,6),    s!(8,11),   s!(9,8),    s!(21,7),   s!(18,-4),  s!(42,-8),  s!(17,5),   s!(22,-10), 
s!(10,4),   s!(18,6),   s!(13,20),  s!(32,18),  s!(34,14),  s!(50,-2),  s!(24,-2),  s!(25,-7),  
s!(19,14),  s!(16,13),  s!(28,22),  s!(49,20),  s!(33,27),  s!(46,18),  s!(16,19),  s!(52,3),   
s!(12,13),  s!(15,9),   s!(19,25),  s!(26,23),  s!(31,28),  s!(31,17),  s!(34,7),   s!(27,9),   
s!(-6,2),   s!(4,4),    s!(2,9),    s!(8,23),   s!(20,21),  s!(6,5),    s!(25,1),   s!(10,3),   
s!(-2,5),   s!(4,8),    s!(2,4),    s!(13,5),   s!(12,3),   s!(12,3),   s!(17,1),   s!(18,15),  
s!(-32,17), s!(0,-6),   s!(-9,0),   s!(0,2),    s!(8,4),    s!(9,-4),   s!(2,3),    s!(3,13),    ];

pub const BISHOP_PSQT: [S; 64] = [
s!(-30,3),  s!(-62,10), s!(-56,3),  s!(-91,11), s!(-77,10), s!(-77,0),  s!(-44,3),  s!(-52,-5), 
s!(-15,-9), s!(-21,-3), s!(-14,-6), s!(-30,0),  s!(-24,-5), s!(-12,-5), s!(-34,0),  s!(-23,-6), 
s!(-8,2),   s!(-6,-6),  s!(-8,0),   s!(-2,-5),  s!(-1,-4),  s!(28,0),   s!(10,-2),  s!(8,3),    
s!(-18,-1), s!(-3,0),   s!(-4,0),   s!(14,12),  s!(12,3),   s!(8,2),    s!(4,-4),   s!(-11,2),  
s!(-4,-5),  s!(-17,1),  s!(-3,6),   s!(13,5),   s!(14,4),   s!(-3,0),   s!(-7,0),   s!(13,-15), 
s!(-6,-1),  s!(8,0),    s!(1,1),    s!(3,4),    s!(2,6),    s!(3,0),    s!(9,-8),   s!(13,-8),  
s!(18,5),   s!(2,-12),  s!(16,-15), s!(-6,-4),  s!(0,-2),   s!(7,-9),   s!(18,-8),  s!(17,-10), 
s!(8,-5),   s!(19,4),   s!(-2,-8),  s!(-7,-7),  s!(0,-9),   s!(-8,3),   s!(9,-9),   s!(31,-23),  ];

pub const ROOK_PSQT: [S; 64] = [
s!(-3,24),  s!(-10,29), s!(-18,36), s!(-22,32), s!(-13,26), s!(7,24),   s!(4,26),   s!(12,23),  
s!(28,0),   s!(28,7),   s!(42,9),   s!(54,-1),  s!(38,1),   s!(63,-2),  s!(55,-3),  s!(58,-8),  
s!(-9,25),  s!(18,21),  s!(13,23),  s!(13,19),  s!(39,11),  s!(48,5),   s!(72,2),   s!(36,2),   
s!(-9,26),  s!(2,22),   s!(3,26),   s!(11,20),  s!(13,10),  s!(21,5),   s!(22,9),   s!(14,6),   
s!(-18,18), s!(-18,19), s!(-8,16),  s!(-1,12),  s!(-2,11),  s!(-9,9),   s!(11,2),   s!(-5,5),   
s!(-20,11), s!(-17,8),  s!(-12,4),  s!(-9,4),   s!(-1,-1),  s!(0,-6),   s!(27,-21), s!(5,-14),  
s!(-20,2),  s!(-16,3),  s!(-6,2),   s!(-3,0),   s!(0,-6),   s!(4,-12),  s!(15,-18), s!(-11,-10),
s!(-9,4),   s!(-4,0),   s!(-1,6),   s!(4,0),    s!(9,-8),   s!(1,-3),   s!(7,-8),   s!(-7,-6),   ];

pub const QUEEN_PSQT: [S; 64] = [
s!(-35,23), s!(-40,27), s!(-33,51), s!(-14,48), s!(-28,57), s!(-25,61), s!(14,5),   s!(-21,33), 
s!(34,-17), s!(18,-1),  s!(20,26),  s!(7,41),   s!(-4,67),  s!(25,41),  s!(27,14),  s!(65,4),   
s!(3,19),   s!(2,23),   s!(-3,51),  s!(2,55),   s!(-13,79), s!(17,63),  s!(18,38),  s!(13,40),  
s!(-4,23),  s!(-5,42),  s!(-5,51),  s!(-8,69),  s!(-10,74), s!(0,57),   s!(9,58),   s!(8,45),   
s!(-3,19),  s!(-9,46),  s!(-9,47),  s!(-6,65),  s!(-4,60),  s!(-4,52),  s!(7,39),   s!(14,33),  
s!(0,0),    s!(0,20),   s!(-5,38),  s!(-5,34),  s!(-3,38),  s!(0,33),   s!(16,14),  s!(14,8),   
s!(7,-15),  s!(1,-7),   s!(4,-2),   s!(6,5),    s!(4,8),    s!(9,-15),  s!(15,-41), s!(34,-61), 
s!(-1,-14), s!(1,-13),  s!(3,-7),   s!(2,3),    s!(7,-16),  s!(-9,-12), s!(2,-24),  s!(11,-34),  ];

pub const KING_PSQT: [S; 64] = [
s!(69,-97), s!(85,-55), s!(89,-41), s!(3,-4),   s!(40,-19), s!(-12,-5), s!(30,-16), s!(135,-114),
s!(-41,-11),s!(23,16),  s!(25,18),  s!(116,3),  s!(55,19),  s!(43,35),  s!(37,30),  s!(-25,5),  
s!(-40,-5), s!(75,11),  s!(20,32),  s!(6,43),   s!(45,45),  s!(99,35),  s!(37,32),  s!(10,-2),  
s!(9,-26),  s!(12,6),   s!(-14,30), s!(-55,51), s!(-52,50), s!(-16,38), s!(-9,19),  s!(-72,0),  
s!(-11,-33),s!(13,-6),  s!(-1,17),  s!(-46,40), s!(-38,38), s!(-2,18),  s!(-18,4),  s!(-84,-8), 
s!(0,-33),  s!(39,-12), s!(8,7),    s!(0,19),   s!(0,18),   s!(4,10),   s!(13,-5),  s!(-27,-17),
s!(47,-37), s!(19,-8),  s!(15,0),   s!(-14,10), s!(-14,13), s!(-2,6),   s!(24,-10), s!(22,-28), 
s!(9,-56),  s!(17,-31), s!(3,-13),  s!(-58,4),  s!(-18,-12),s!(-44,0),  s!(-3,-22), s!(11,-61),  ];

pub const PASSED_PAWN_TABLE: [S; 64] = [
s!(0,0),    s!(0,0),    s!(0,0),    s!(0,0),    s!(0,0),    s!(0,0),    s!(0,0),    s!(0,0),    
s!(11,69),  s!(1,65),   s!(19,71),  s!(17,57),  s!(17,55),  s!(2,71),   s!(1,76),   s!(0,72),   
s!(13,160), s!(43,154), s!(18,132), s!(19,94),  s!(4,110),  s!(10,124), s!(-11,134),s!(-36,156),
s!(7,88),   s!(12,81),  s!(24,59),  s!(14,56),  s!(0,55),   s!(10,59),  s!(-10,81), s!(-16,89), 
s!(-10,51), s!(-2,38),  s!(-14,28), s!(-4,15),  s!(-15,18), s!(-12,24), s!(-11,42), s!(-13,44), 
s!(-5,7),   s!(-11,10), s!(-12,-2), s!(-5,-17), s!(-9,-16), s!(-13,-11),s!(-14,10), s!(4,-1),   
s!(-15,6),  s!(-7,-1),  s!(-16,-14),s!(-8,-25), s!(0,-38),  s!(-13,-30),s!(0,-19),  s!(-2,-9),  
s!(0,0),    s!(0,0),    s!(0,0),    s!(0,0),    s!(0,0),    s!(0,0),    s!(0,0),    s!(0,0),     ];

pub const KNIGHT_MOBILITY_BONUS: [S; 9] = [
s!(-48,-53),
s!(-69,-68),
s!(-44,-36),
s!(-33,-15),
s!(-20,-7),
s!(-14,3),
s!(-3,3),
s!(4,6),
s!(19,-2),
];

pub const BISHOP_MOBILITY_BONUS: [S; 14] = [
s!(-43,-39),
s!(-55,-70),
s!(-29,-31),
s!(-19,-9),
s!(-8,0),
s!(1,8),
s!(6,18),
s!(11,22),
s!(12,27),
s!(16,28),
s!(18,29),
s!(29,23),
s!(29,27),
s!(37,16),
];

pub const ROOK_MOBILITY_BONUS: [S; 15] = [
s!(-50,-33),
s!(-44,25),
s!(-27,0),
s!(-21,14),
s!(-16,24),
s!(-12,30),
s!(-10,36),
s!(-7,40),
s!(-4,41),
s!(0,45),
s!(1,48),
s!(3,51),
s!(5,53),
s!(6,53),
s!(9,49),
];

pub const QUEEN_MOBILITY_BONUS: [S; 28] = [
s!(-82,155),
s!(-63,116),
s!(-57,72),
s!(-59,35),
s!(-52,0),
s!(-26,1),
s!(-18,19),
s!(-17,45),
s!(-14,59),
s!(-10,72),
s!(-6,72),
s!(-2,74),
s!(0,79),
s!(5,77),
s!(8,77),
s!(9,80),
s!(12,78),
s!(16,76),
s!(18,77),
s!(24,67),
s!(35,59),
s!(46,38),
s!(62,28),
s!(84,1),
s!(85,-1),
s!(215,-86),
s!(124,-57),
s!(17,-12),
];

pub const VIRTUAL_MOBILITY_PENALTY: [S; 28] = [
s!(16,-13),
s!(19,-15),
s!(16,-8),
s!(14,-13),
s!(11,-9),
s!(12,-10),
s!(11,-11),
s!(9,-11),
s!(8,-4),
s!(2,-3),
s!(-1,0),
s!(-7,1),
s!(-9,2),
s!(-15,4),
s!(-19,5),
s!(-27,7),
s!(-35,4),
s!(-40,5),
s!(-38,1),
s!(-24,-5),
s!(-22,-7),
s!(-18,-11),
s!(-34,-13),
s!(-7,-20),
s!(-27,-24),
s!(15,-31),
s!(15,-35),
s!(66,-38),
];

pub const KING_ZONE_ATTACKS: [S; 16] = [
s!(155,-41),
s!(151,-32),
s!(135,-29),
s!(110,-31),
s!(57,-11),
s!(0,8),
s!(-73,44),
s!(-146,75),
s!(-240,95),
s!(-398,228),
s!(-640,555),
s!(-408,387),
s!(-334,-162),
s!(-188,-59),
s!(-84,-29),
s!(-28,-5),
];

pub const ISOLATED_PAWN_PENALTY: S = s!(-4,-8);

pub const DOUBLED_PAWN_PENALTY: S = s!(-4,-16);

pub const PROTECTED_PAWN_BONUS: S = s!(17,12);

pub const PHALANX_PAWN_BONUS: S = s!(5,5);

pub const BISHOP_PAIR_BONUS: S = s!(21,62);

pub const ROOK_OPEN_FILE_BONUS: S = s!(17,-12);

pub const ROOK_SEMIOPEN_FILE_BONUS: S = s!(11,18);

pub const CONNECTED_ROOKS_BONUS: S = s!(-4,2);

pub const MAJOR_ON_SEVENTH_BONUS: S = s!(-36,38);

pub const QUEEN_OPEN_FILE_BONUS: S = s!(-13,25);

pub const QUEEN_SEMIOPEN_FILE_BONUS: S = s!(3,16);

pub const PAWN_SHIELD_BONUS: [S; 3] = [
s!(21,-15),
s!(18,-7),
s!(14,-9),
];

pub const PAWN_STORM_BONUS: [S; 3] = [
s!(7,-30),
s!(3,-2),
s!(-8,-2),
];

pub const PASSERS_FRIENDLY_KING_BONUS: [S; 7] = [
s!(-3,28),
s!(-7,15),
s!(-2,-2),
s!(2,-13),
s!(6,-17),
s!(21,-22),
s!(9,-22),
];

pub const PASSERS_ENEMY_KING_PENALTY: [S; 7] = [
s!(-50,-37),
s!(5,-20),
s!(1,3),
s!(0,14),
s!(-3,22),
s!(-1,26),
s!(-17,27),
];

pub const PAWN_ATTACKS_ON_MINORS: S = s!(58,31);

pub const PAWN_ATTACKS_ON_ROOKS: S = s!(81,5);

pub const PAWN_ATTACKS_ON_QUEENS: S = s!(69,-32);

pub const MINOR_ATTACKS_ON_ROOKS: S = s!(54,20);

pub const MINOR_ATTACKS_ON_QUEENS: S = s!(56,12);

pub const ROOK_ATTACKS_ON_QUEENS: S = s!(63,22);

pub const KNIGHT_OUTPOSTS: S = s!(26,16);

pub const BISHOP_OUTPOSTS: S = s!(34,3);

pub const TEMPO_BONUS: S = s!(26,23);

pub const SAFE_CHECKS: [S; 6] = [
s!(10,1),
s!(86,-4),
s!(15,19),
s!(65,-2),
s!(27,18),
s!(0,0),
];

