use crate::evaluate::S;

use crate::s;

pub const PIECE_VALUES: [S; 6] = [
s!(51,96),
s!(299,365),
s!(314,371),
s!(402,611),
s!(860,1131),
s!(0,0),
];

pub const PAWN_PSQT: [S; 64] = [
s!(0,0),    s!(0,0),    s!(0,0),    s!(0,0),    s!(0,0),    s!(0,0),    s!(0,0),    s!(0,0),    
s!(61,170), s!(77,164), s!(38,166), s!(67,137), s!(56,142), s!(62,143), s!(13,169), s!(-1,182), 
s!(18,31),  s!(-1,37),  s!(30,25),  s!(24,35),  s!(44,17),  s!(68,18),  s!(46,43),  s!(29,33),  
s!(1,23),   s!(4,17),   s!(5,14),   s!(10,4),   s!(29,4),   s!(27,8),   s!(16,14),  s!(17,6),   
s!(-2,6),   s!(-6,11),  s!(6,5),    s!(13,3),   s!(16,3),   s!(17,4),   s!(6,4),    s!(5,-3),   
s!(-12,3),  s!(-8,1),   s!(-4,4),   s!(0,11),   s!(8,10),   s!(0,8),    s!(16,-5),  s!(1,-6),   
s!(-5,6),   s!(-3,5),   s!(5,9),    s!(1,15),   s!(10,22),  s!(23,13),  s!(26,-3),  s!(3,-4),   
s!(0,0),    s!(0,0),    s!(0,0),    s!(0,0),    s!(0,0),    s!(0,0),    s!(0,0),    s!(0,0),     ];

pub const KNIGHT_PSQT: [S; 64] = [
s!(-81,-38),s!(-83,-5), s!(-63,11), s!(-21,-1), s!(8,0),    s!(-52,-19),s!(-78,-3), s!(-52,-54),
s!(0,6),    s!(7,10),   s!(8,8),    s!(20,7),   s!(18,-4),  s!(41,-8),  s!(17,4),   s!(22,-11), 
s!(10,4),   s!(17,6),   s!(13,20),  s!(32,18),  s!(34,14),  s!(49,-2),  s!(24,-3),  s!(25,-7),  
s!(18,14),  s!(15,12),  s!(28,21),  s!(48,20),  s!(32,27),  s!(46,18),  s!(16,18),  s!(51,3),   
s!(11,13),  s!(14,9),   s!(19,25),  s!(26,23),  s!(30,28),  s!(30,16),  s!(34,6),   s!(27,8),   
s!(-6,1),   s!(3,4),    s!(1,8),    s!(7,23),   s!(20,20),  s!(5,5),    s!(24,1),   s!(10,3),   
s!(-2,5),   s!(4,8),    s!(2,4),    s!(13,4),   s!(11,3),   s!(11,3),   s!(16,0),   s!(18,15),  
s!(-32,17), s!(0,-6),   s!(-10,0),  s!(0,2),    s!(7,4),    s!(8,-4),   s!(1,3),    s!(3,12),    ];

pub const BISHOP_PSQT: [S; 64] = [
s!(-30,3),  s!(-62,9),  s!(-56,3),  s!(-91,11), s!(-76,10), s!(-77,0),  s!(-44,3),  s!(-52,-5), 
s!(-15,-9), s!(-21,-3), s!(-14,-6), s!(-30,0),  s!(-24,-4), s!(-13,-5), s!(-34,0),  s!(-23,-6), 
s!(-8,2),   s!(-6,-5),  s!(-9,0),   s!(-3,-5),  s!(-2,-4),  s!(28,0),   s!(10,-1),  s!(8,3),    
s!(-18,-1), s!(-4,0),   s!(-4,0),   s!(14,11),  s!(12,3),   s!(8,2),    s!(4,-4),   s!(-11,2),  
s!(-5,-5),  s!(-17,1),  s!(-3,6),   s!(12,5),   s!(14,4),   s!(-3,0),   s!(-8,0),   s!(13,-15), 
s!(-6,-1),  s!(8,0),    s!(1,1),    s!(3,4),    s!(2,6),    s!(3,0),    s!(8,-8),   s!(13,-8),  
s!(18,5),   s!(2,-12),  s!(16,-14), s!(-7,-4),  s!(0,-2),   s!(7,-9),   s!(18,-8),  s!(17,-10), 
s!(7,-5),   s!(19,4),   s!(-2,-8),  s!(-8,-7),  s!(0,-9),   s!(-9,3),   s!(9,-8),   s!(31,-23),  ];

pub const ROOK_PSQT: [S; 64] = [
s!(-3,25),  s!(-11,30), s!(-18,36), s!(-22,33), s!(-14,27), s!(6,26),   s!(4,27),   s!(11,24),  
s!(29,0),   s!(29,8),   s!(42,10),  s!(55,0),   s!(39,1),   s!(63,-1),  s!(57,-2),  s!(59,-6),  
s!(-8,26),  s!(19,21),  s!(14,24),  s!(14,20),  s!(39,11),  s!(48,6),   s!(73,3),   s!(37,2),   
s!(-9,27),  s!(3,23),   s!(3,27),   s!(11,20),  s!(13,10),  s!(21,6),   s!(22,10),  s!(14,7),   
s!(-17,19), s!(-18,19), s!(-8,16),  s!(-1,13),  s!(-2,11),  s!(-9,10),  s!(12,2),   s!(-5,6),   
s!(-19,11), s!(-17,8),  s!(-12,4),  s!(-9,4),   s!(0,-1),   s!(0,-6),   s!(28,-20), s!(6,-13),  
s!(-20,2),  s!(-16,4),  s!(-5,2),   s!(-3,0),   s!(1,-6),   s!(5,-11),  s!(16,-18), s!(-11,-9), 
s!(-8,5),   s!(-4,1),   s!(-1,6),   s!(4,0),    s!(9,-7),   s!(1,-2),   s!(8,-7),   s!(-7,-5),   ];

pub const QUEEN_PSQT: [S; 64] = [
s!(-34,22), s!(-40,27), s!(-34,52), s!(-16,49), s!(-28,57), s!(-24,60), s!(14,4),   s!(-23,34), 
s!(33,-16), s!(18,-1),  s!(20,26),  s!(7,40),   s!(-3,66),  s!(25,40),  s!(27,14),  s!(64,5),   
s!(2,20),   s!(2,23),   s!(-3,51),  s!(2,55),   s!(-12,78), s!(18,61),  s!(18,38),  s!(14,40),  
s!(-4,24),  s!(-5,42),  s!(-4,50),  s!(-7,67),  s!(-9,72),  s!(1,56),   s!(9,57),   s!(9,44),   
s!(-3,20),  s!(-9,46),  s!(-9,47),  s!(-5,65),  s!(-4,59),  s!(-3,51),  s!(8,38),   s!(14,33),  
s!(0,0),    s!(0,20),   s!(-5,37),  s!(-5,34),  s!(-2,37),  s!(0,32),   s!(16,13),  s!(14,7),   
s!(7,-15),  s!(1,-7),   s!(5,-2),   s!(6,5),    s!(4,7),    s!(9,-15),  s!(15,-40), s!(34,-59), 
s!(-1,-13), s!(1,-12),  s!(3,-7),   s!(2,3),    s!(7,-15),  s!(-9,-11), s!(1,-23),  s!(11,-33),  ];

pub const KING_PSQT: [S; 64] = [
s!(68,-96), s!(82,-54), s!(89,-41), s!(0,-3),   s!(43,-19), s!(-10,-5), s!(31,-16), s!(145,-115),
s!(-40,-11),s!(24,16),  s!(22,19),  s!(120,2),  s!(55,19),  s!(42,35),  s!(37,30),  s!(-25,5),  
s!(-39,-5), s!(74,11),  s!(18,32),  s!(2,43),   s!(40,45),  s!(97,35),  s!(37,33),  s!(13,-2),  
s!(8,-26),  s!(11,6),   s!(-16,30), s!(-58,52), s!(-54,51), s!(-18,39), s!(-10,19), s!(-72,0),  
s!(-12,-33),s!(11,-6),  s!(-3,18),  s!(-48,41), s!(-40,39), s!(-3,18),  s!(-17,4),  s!(-84,-8), 
s!(0,-33),  s!(39,-12), s!(7,8),    s!(0,19),   s!(0,19),   s!(3,10),   s!(13,-4),  s!(-27,-16),
s!(47,-37), s!(20,-8),  s!(15,1),   s!(-14,10), s!(-14,13), s!(-3,6),   s!(24,-10), s!(22,-27), 
s!(9,-56),  s!(17,-31), s!(3,-13),  s!(-58,4),  s!(-18,-12),s!(-44,0),  s!(-3,-22), s!(11,-60),  ];

pub const PASSED_PAWN_TABLE: [S; 64] = [
s!(0,0),    s!(0,0),    s!(0,0),    s!(0,0),    s!(0,0),    s!(0,0),    s!(0,0),    s!(0,0),    
s!(8,69),   s!(2,65),   s!(19,71),  s!(15,56),  s!(16,54),  s!(2,71),   s!(2,76),   s!(-1,72),  
s!(9,159),  s!(44,153), s!(18,131), s!(18,93),  s!(3,109),  s!(9,124),  s!(-10,133),s!(-39,156),
s!(4,88),   s!(13,81),  s!(24,58),  s!(15,55),  s!(0,55),   s!(9,59),   s!(-8,81),  s!(-18,90), 
s!(-13,52), s!(0,38),   s!(-12,27), s!(-2,14),  s!(-14,18), s!(-14,25), s!(-10,42), s!(-15,45), 
s!(-9,8),   s!(-9,10),  s!(-10,-2), s!(-3,-17), s!(-8,-16), s!(-13,-10),s!(-12,10), s!(3,-1),   
s!(-20,8),  s!(-6,-1),  s!(-14,-14),s!(-7,-25), s!(0,-38),  s!(-17,-29),s!(0,-18),  s!(-4,-8),  
s!(0,0),    s!(0,0),    s!(0,0),    s!(0,0),    s!(0,0),    s!(0,0),    s!(0,0),    s!(0,0),     ];

pub const KNIGHT_MOBILITY_BONUS: [S; 9] = [
s!(-49,-59),
s!(-68,-67),
s!(-42,-36),
s!(-31,-15),
s!(-19,-7),
s!(-13,3),
s!(-2,3),
s!(5,7),
s!(21,-2),
];

pub const BISHOP_MOBILITY_BONUS: [S; 14] = [
s!(-45,-43),
s!(-55,-69),
s!(-29,-31),
s!(-18,-9),
s!(-8,0),
s!(1,8),
s!(7,17),
s!(12,22),
s!(13,27),
s!(16,27),
s!(18,28),
s!(30,23),
s!(30,26),
s!(37,15),
];

pub const ROOK_MOBILITY_BONUS: [S; 15] = [
s!(-51,-46),
s!(-51,21),
s!(-27,0),
s!(-20,14),
s!(-15,25),
s!(-11,31),
s!(-9,37),
s!(-7,41),
s!(-3,42),
s!(0,45),
s!(2,48),
s!(4,51),
s!(6,53),
s!(8,53),
s!(11,49),
];

pub const QUEEN_MOBILITY_BONUS: [S; 28] = [
s!(-65,90),
s!(-52,55),
s!(-54,35),
s!(-56,2),
s!(-48,-23),
s!(-25,-3),
s!(-18,18),
s!(-17,46),
s!(-13,60),
s!(-10,73),
s!(-6,74),
s!(-2,76),
s!(0,80),
s!(5,79),
s!(8,79),
s!(9,82),
s!(12,80),
s!(16,79),
s!(18,79),
s!(24,70),
s!(35,62),
s!(46,42),
s!(61,32),
s!(84,4),
s!(84,2),
s!(216,-84),
s!(125,-54),
s!(10,-6),
];

pub const VIRTUAL_MOBILITY_PENALTY: [S; 28] = [
s!(16,-13),
s!(19,-16),
s!(16,-9),
s!(14,-13),
s!(11,-10),
s!(12,-11),
s!(11,-12),
s!(9,-12),
s!(8,-5),
s!(2,-3),
s!(-1,-1),
s!(-7,1),
s!(-9,2),
s!(-15,4),
s!(-19,4),
s!(-27,6),
s!(-34,4),
s!(-39,4),
s!(-36,1),
s!(-22,-5),
s!(-21,-8),
s!(-15,-11),
s!(-31,-13),
s!(-4,-20),
s!(-22,-24),
s!(19,-32),
s!(20,-35),
s!(72,-39),
];

pub const KING_ZONE_ATTACKS: [S; 16] = [
s!(155,-40),
s!(151,-32),
s!(134,-28),
s!(110,-30),
s!(58,-11),
s!(0,8),
s!(-73,44),
s!(-146,76),
s!(-230,88),
s!(-395,219),
s!(-674,576),
s!(-419,400),
s!(-332,-160),
s!(-188,-59),
s!(-84,-29),
s!(-28,-5),
];

pub const ISOLATED_PAWN_PENALTY: S = s!(-4,-8);

pub const DOUBLED_PAWN_PENALTY: S = s!(-3,-16);

pub const PROTECTED_PAWN_BONUS: S = s!(17,12);

pub const PHALANX_PAWN_BONUS: S = s!(5,5);

pub const BISHOP_PAIR_BONUS: S = s!(21,61);

pub const ROOK_OPEN_FILE_BONUS: S = s!(17,-12);

pub const ROOK_SEMIOPEN_FILE_BONUS: S = s!(12,20);

pub const CONNECTED_ROOKS_BONUS: S = s!(-4,2);

pub const MAJOR_ON_SEVENTH_BONUS: S = s!(-35,37);

pub const QUEEN_OPEN_FILE_BONUS: S = s!(-13,24);

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
s!(-4,27),
s!(-9,15),
s!(-6,-2),
s!(-1,-14),
s!(2,-18),
s!(17,-22),
s!(7,-22),
];

pub const PASSERS_ENEMY_KING_PENALTY: [S; 7] = [
s!(-50,-36),
s!(6,-19),
s!(2,3),
s!(0,14),
s!(-2,22),
s!(0,26),
s!(-14,28),
];

pub const PAWN_ATTACKS_ON_MINORS: S = s!(58,31);

pub const PAWN_ATTACKS_ON_ROOKS: S = s!(81,5);

pub const PAWN_ATTACKS_ON_QUEENS: S = s!(69,-32);

pub const MINOR_ATTACKS_ON_ROOKS: S = s!(54,20);

pub const MINOR_ATTACKS_ON_QUEENS: S = s!(56,9);

pub const ROOK_ATTACKS_ON_QUEENS: S = s!(64,19);

pub const KNIGHT_OUTPOSTS: S = s!(26,16);

pub const BISHOP_OUTPOSTS: S = s!(34,3);

pub const KNIGHT_SHELTER: S = s!(6,10);

pub const BISHOP_SHELTER: S = s!(10,2);

pub const TEMPO_BONUS: S = s!(26,23);

pub const SAFE_CHECKS: [S; 6] = [
s!(10,1),
s!(85,-4),
s!(15,19),
s!(65,-2),
s!(27,19),
s!(0,0),
];

pub const UNSAFE_CHECKS: [S; 6] = [
s!(-4,5),
s!(12,0),
s!(15,12),
s!(23,-5),
s!(6,9),
s!(0,0),
];

pub const BAD_BISHOPS: [S; 9] = [
s!(11,17),
s!(9,15),
s!(5,7),
s!(0,0),
s!(-4,-10),
s!(-6,-23),
s!(-12,-35),
s!(-18,-52),
s!(-36,-65),
];

pub const SUPPORTED_PASSER: S = s!(13,8);
