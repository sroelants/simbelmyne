use crate::evaluate::S;
use crate::s;

pub const PIECE_VALUES: [S; 6] = [
s!(52,97),
s!(306,364),
s!(315,372),
s!(394,614),
s!(837,1101),
s!(0,0),
];

pub const PAWN_PSQT: [S; 64] = [
s!(0,0),    s!(0,0),    s!(0,0),    s!(0,0),    s!(0,0),    s!(0,0),    s!(0,0),    s!(0,0),    
s!(60,145), s!(79,134), s!(44,135), s!(71,106), s!(60,109), s!(60,109), s!(14,131), s!(-8,151), 
s!(18,26),  s!(4,30),   s!(30,20),  s!(24,29),  s!(48,8),   s!(66,12),  s!(51,36),  s!(29,29),  
s!(1,18),   s!(6,11),   s!(7,10),   s!(9,0),    s!(25,0),   s!(30,2),   s!(18,8),   s!(17,2),   
s!(-1,3),   s!(-2,5),   s!(7,0),    s!(15,0),   s!(19,0),   s!(17,-1),  s!(8,0),    s!(6,-6),   
s!(-11,0),  s!(-7,-2),  s!(0,0),    s!(0,6),    s!(7,5),    s!(2,3),    s!(16,-10), s!(2,-9),   
s!(-4,2),   s!(-1,0),   s!(4,5),    s!(1,12),   s!(11,16),  s!(21,8),   s!(26,-8),  s!(4,-7),   
s!(0,0),    s!(0,0),    s!(0,0),    s!(0,0),    s!(0,0),    s!(0,0),    s!(0,0),    s!(0,0),     ];

pub const KNIGHT_PSQT: [S; 64] = [
s!(-92,-26),s!(-91,0),  s!(-70,11), s!(-21,-2), s!(-1,2),   s!(-48,-16),s!(-81,-1), s!(-64,-41),
s!(-13,12), s!(-1,12),  s!(-2,5),   s!(7,6),    s!(4,-2),   s!(33,-10), s!(7,8),    s!(8,-3),   
s!(0,6),    s!(4,3),    s!(3,9),    s!(25,5),   s!(31,2),   s!(47,-14), s!(8,-2),   s!(16,-3),  
s!(10,13),  s!(9,4),    s!(24,5),   s!(27,12),  s!(36,7),   s!(32,4),   s!(22,3),   s!(38,5),   
s!(2,9),    s!(9,0),    s!(13,7),   s!(23,8),   s!(18,15),  s!(24,0),   s!(16,0),   s!(16,6),   
s!(-13,-2), s!(-2,-5),  s!(-1,-5),  s!(1,8),    s!(11,6),   s!(4,-10),  s!(17,-7),  s!(2,0),    
s!(-14,0),  s!(-6,1),   s!(-7,-4),  s!(2,0),    s!(2,-2),   s!(1,-3),   s!(4,-3),   s!(4,10),   
s!(-43,15), s!(-13,-5), s!(-20,0),  s!(-8,2),   s!(-2,4),   s!(-3,-7),  s!(-10,3),  s!(-8,8),    ];

pub const BISHOP_PSQT: [S; 64] = [
s!(-38,-5), s!(-51,8),  s!(-52,1),  s!(-79,9),  s!(-74,4),  s!(-62,-1), s!(-37,-3), s!(-59,-10),
s!(-7,-8),  s!(-29,-10),s!(-8,-7),  s!(-24,-4), s!(-10,-8), s!(-6,-12), s!(-31,-8), s!(-18,-10),
s!(-5,2),   s!(1,-5),   s!(-17,-7), s!(5,-6),   s!(-2,-6),  s!(27,-7),  s!(8,-8),   s!(19,1),   
s!(-11,0),  s!(-12,0),  s!(3,-1),   s!(7,6),    s!(13,0),   s!(2,0),    s!(0,-3),   s!(-11,0),  
s!(-9,-7),  s!(-9,0),   s!(-3,-1),  s!(14,0),   s!(9,-5),   s!(3,-4),   s!(-7,-5),  s!(7,-14),  
s!(-3,-3),  s!(7,-6),   s!(0,-8),   s!(0,-2),   s!(6,1),    s!(-1,-13), s!(14,-11), s!(9,-15),  
s!(11,-9),  s!(0,-21),  s!(11,-21), s!(-2,-4),  s!(0,-6),   s!(8,-13),  s!(9,-22),  s!(15,-19), 
s!(6,-15),  s!(12,-2),  s!(0,-4),   s!(-6,-7),  s!(5,-7),   s!(-12,3),  s!(11,-16), s!(21,-33),  ];

pub const ROOK_PSQT: [S; 64] = [
s!(-15,25), s!(-22,28), s!(-27,34), s!(-35,32), s!(-27,27), s!(-3,23),  s!(-6,24),  s!(1,21),   
s!(21,0),   s!(24,3),   s!(34,4),   s!(48,-5),  s!(31,-3),  s!(45,-3),  s!(40,-1),  s!(46,-6),  
s!(-11,19), s!(14,12),  s!(7,14),   s!(5,9),    s!(28,2),   s!(35,0),   s!(57,-2),  s!(22,1),   
s!(-10,21), s!(1,15),   s!(1,18),   s!(5,13),   s!(4,4),    s!(16,-1),  s!(12,5),   s!(6,4),    
s!(-17,15), s!(-19,15), s!(-10,12), s!(-3,8),   s!(-3,7),   s!(-11,5),  s!(4,-1),   s!(-7,1),   
s!(-19,6),  s!(-17,3),  s!(-12,0),  s!(-9,0),   s!(-3,-4),  s!(1,-10),  s!(21,-23), s!(5,-19),  
s!(-18,-5), s!(-15,-2), s!(-6,-2),  s!(-4,-4),  s!(0,-10),  s!(3,-14),  s!(10,-20), s!(-10,-16),
s!(-8,-1),  s!(-3,-3),  s!(-2,1),   s!(4,-6),   s!(8,-12),  s!(1,-7),   s!(5,-11),  s!(-7,-11),  ];

pub const QUEEN_PSQT: [S; 64] = [
s!(-35,17), s!(-42,18), s!(-34,36), s!(-16,30), s!(-34,40), s!(-33,47), s!(9,-5),   s!(-21,22), 
s!(26,-14), s!(14,-9),  s!(14,12),  s!(1,23),   s!(-13,47), s!(15,20),  s!(21,0),   s!(57,0),   
s!(1,14),   s!(2,10),   s!(-5,32),  s!(0,31),   s!(-14,49), s!(12,27),  s!(15,11),  s!(11,19),  
s!(-6,22),  s!(-4,30),  s!(-4,36),  s!(-7,46),  s!(-8,45),  s!(2,28),   s!(9,36),   s!(11,23),  
s!(-1,13),  s!(-8,39),  s!(-7,36),  s!(-2,49),  s!(0,40),   s!(0,34),   s!(9,24),   s!(16,22),  
s!(0,0),    s!(1,15),   s!(-1,32),  s!(-1,27),  s!(1,30),   s!(4,25),   s!(18,5),   s!(16,1),   
s!(5,-12),  s!(2,-5),   s!(4,0),    s!(7,9),    s!(6,10),   s!(10,-13), s!(16,-40), s!(34,-56), 
s!(-2,-4),  s!(1,-9),   s!(2,-1),   s!(0,12),   s!(6,-8),   s!(-7,-9),  s!(3,-21),  s!(11,-29),  ];

pub const KING_PSQT: [S; 64] = [
s!(88,-106),s!(97,-66), s!(89,-44), s!(25,-10), s!(58,-21), s!(5,-6),   s!(43,-10), s!(151,-108),
s!(-53,-10),s!(30,15),  s!(25,23),  s!(132,4),  s!(57,29),  s!(33,48),  s!(15,40),  s!(-12,12), 
s!(-54,-2), s!(56,17),  s!(18,39),  s!(1,54),   s!(44,57),  s!(85,49),  s!(17,44),  s!(8,6),    
s!(-8,-19), s!(-2,13),  s!(-26,41), s!(-51,61), s!(-48,61), s!(-27,51), s!(-20,28), s!(-93,11), 
s!(-25,-27),s!(3,0),    s!(-9,27),  s!(-46,49), s!(-40,47), s!(-7,28),  s!(-23,11), s!(-92,0),  
s!(-1,-32), s!(32,-8),  s!(4,14),   s!(0,25),   s!(0,25),   s!(4,14),   s!(12,-1),  s!(-27,-15),
s!(51,-44), s!(23,-12), s!(17,0),   s!(-12,9),  s!(-11,12), s!(1,5),    s!(26,-13), s!(25,-32), 
s!(17,-70), s!(22,-39), s!(9,-18),  s!(-50,-3), s!(-13,-17),s!(-38,-5), s!(1,-28),  s!(15,-68),  ];

pub const PASSED_PAWN_TABLE: [S; 64] = [
s!(0,0),    s!(0,0),    s!(0,0),    s!(0,0),    s!(0,0),    s!(0,0),    s!(0,0),    s!(0,0),    
s!(8,44),   s!(5,35),   s!(25,40),  s!(19,25),  s!(20,21),  s!(1,37),   s!(5,38),   s!(-8,41),  
s!(19,82),  s!(53,75),  s!(25,56),  s!(21,26),  s!(4,41),   s!(14,50),  s!(-1,56),  s!(-34,78), 
s!(20,60),  s!(21,53),  s!(29,30),  s!(17,26),  s!(6,23),   s!(15,27),  s!(0,47),   s!(-5,56),  
s!(-6,54),  s!(3,38),   s!(-8,27),  s!(-3,15),  s!(-10,14), s!(-5,19),  s!(-4,37),  s!(-8,41),  
s!(-14,35), s!(-14,33), s!(-16,19), s!(-12,6),  s!(-15,5),  s!(-13,7),  s!(-19,33), s!(-4,22),  
s!(-26,40), s!(-13,30), s!(-21,19), s!(-22,16), s!(-10,-2), s!(-21,7),  s!(-11,19), s!(-15,26), 
s!(0,0),    s!(0,0),    s!(0,0),    s!(0,0),    s!(0,0),    s!(0,0),    s!(0,0),    s!(0,0),     ];

pub const KNIGHT_MOBILITY_BONUS: [S; 9] = [
s!(-43,-52),
s!(-66,-79),
s!(-43,-49),
s!(-33,-26),
s!(-22,-16),
s!(-16,-3),
s!(-7,0),
s!(0,7),
s!(12,1),
];

pub const BISHOP_MOBILITY_BONUS: [S; 14] = [
s!(-41,-49),
s!(-57,-78),
s!(-32,-40),
s!(-21,-20),
s!(-11,-10),
s!(-4,-2),
s!(0,7),
s!(4,10),
s!(6,15),
s!(10,15),
s!(13,17),
s!(26,8),
s!(31,14),
s!(46,-2),
];

pub const ROOK_MOBILITY_BONUS: [S; 15] = [
s!(-46,-76),
s!(-50,-6),
s!(-27,-25),
s!(-19,-5),
s!(-14,5),
s!(-11,12),
s!(-9,18),
s!(-7,24),
s!(-4,27),
s!(-1,30),
s!(0,35),
s!(0,41),
s!(0,45),
s!(0,49),
s!(3,45),
];

pub const QUEEN_MOBILITY_BONUS: [S; 28] = [
s!(-60,72),
s!(-48,32),
s!(-45,-2),
s!(-51,-24),
s!(-44,-45),
s!(-22,-26),
s!(-16,-1),
s!(-15,25),
s!(-12,38),
s!(-9,52),
s!(-6,54),
s!(-3,57),
s!(-1,62),
s!(2,60),
s!(4,62),
s!(6,65),
s!(8,64),
s!(11,64),
s!(12,65),
s!(17,58),
s!(27,51),
s!(37,32),
s!(52,24),
s!(74,-3),
s!(73,-2),
s!(199,-87),
s!(127,-66),
s!(75,-61),
];

pub const VIRTUAL_MOBILITY_PENALTY: [S; 28] = [
s!(6,-9),
s!(12,-12),
s!(9,-4),
s!(8,-7),
s!(7,-4),
s!(8,-5),
s!(8,-6),
s!(7,-6),
s!(7,0),
s!(2,0),
s!(0,2),
s!(-4,4),
s!(-4,4),
s!(-9,6),
s!(-11,6),
s!(-16,7),
s!(-22,4),
s!(-24,5),
s!(-20,2),
s!(-3,-4),
s!(0,-7),
s!(6,-11),
s!(-5,-12),
s!(24,-21),
s!(11,-24),
s!(58,-34),
s!(51,-35),
s!(102,-40),
];

pub const KING_ZONE_ATTACKS: [S; 16] = [
s!(143,-33),
s!(144,-21),
s!(133,-20),
s!(119,-25),
s!(77,-3),
s!(31,10),
s!(-31,41),
s!(-93,71),
s!(-165,77),
s!(-339,223),
s!(-587,547),
s!(-312,329),
s!(-310,-177),
s!(-197,-62),
s!(-84,-29),
s!(-28,-5),
];

pub const ISOLATED_PAWN_PENALTY: S = s!(-3,-9);

pub const DOUBLED_PAWN_PENALTY: S = s!(-4,-13);

pub const PROTECTED_PAWN_BONUS: S = s!(16,12);

pub const PHALANX_PAWN_BONUS: S = s!(5,4);

pub const BISHOP_PAIR_BONUS: S = s!(19,53);

pub const ROOK_OPEN_FILE_BONUS: S = s!(9,-1);

pub const ROOK_SEMIOPEN_FILE_BONUS: S = s!(17,1);

pub const CONNECTED_ROOKS_BONUS: S = s!(-4,4);

pub const MAJOR_ON_SEVENTH_BONUS: S = s!(-30,29);

pub const QUEEN_OPEN_FILE_BONUS: S = s!(-12,20);

pub const QUEEN_SEMIOPEN_FILE_BONUS: S = s!(5,8);

pub const PAWN_SHIELD_BONUS: [S; 3] = [
s!(20,-13),
s!(16,-5),
s!(12,-8),
];

pub const PAWN_STORM_BONUS: [S; 3] = [
s!(-36,-29),
s!(6,1),
s!(-8,-1),
];

pub const PASSERS_FRIENDLY_KING_BONUS: [S; 7] = [
s!(-8,12),
s!(-8,5),
s!(-6,-2),
s!(-5,-7),
s!(-3,-8),
s!(2,-9),
s!(-2,-10),
];

pub const PASSERS_ENEMY_KING_PENALTY: [S; 7] = [
s!(-55,-34),
s!(0,-16),
s!(0,5),
s!(0,15),
s!(0,22),
s!(1,26),
s!(-15,27),
];

pub const PAWN_ATTACKS: [S; 6] = [
s!(12,-3),
s!(59,22),
s!(53,49),
s!(73,26),
s!(64,-21),
s!(0,0),
];

pub const KNIGHT_ATTACKS: [S; 6] = [
s!(-5,10),
s!(-4,15),
s!(27,32),
s!(63,14),
s!(48,-25),
s!(0,0),
];

pub const BISHOP_ATTACKS: [S; 6] = [
s!(-3,11),
s!(17,23),
s!(0,0),
s!(46,25),
s!(61,80),
s!(0,0),
];

pub const ROOK_ATTACKS: [S; 6] = [
s!(-7,13),
s!(2,18),
s!(10,13),
s!(-28,-7),
s!(67,14),
s!(0,0),
];

pub const QUEEN_ATTACKS: [S; 6] = [
s!(-3,10),
s!(1,8),
s!(0,21),
s!(3,-6),
s!(18,8),
s!(0,0),
];

pub const KNIGHT_OUTPOSTS: S = s!(25,19);

pub const BISHOP_OUTPOSTS: S = s!(35,9);

pub const KNIGHT_SHELTER: S = s!(4,12);

pub const BISHOP_SHELTER: S = s!(8,2);

pub const TEMPO_BONUS: S = s!(26,24);

pub const SAFE_CHECKS: [S; 6] = [
s!(6,-1),
s!(82,-3),
s!(19,15),
s!(67,-2),
s!(27,18),
s!(0,0),
];

pub const UNSAFE_CHECKS: [S; 6] = [
s!(-4,4),
s!(10,0),
s!(14,10),
s!(21,-5),
s!(5,8),
s!(0,0),
];

pub const BAD_BISHOPS: [S; 9] = [
s!(8,5),
s!(7,4),
s!(4,-2),
s!(0,-8),
s!(-4,-15),
s!(-7,-25),
s!(-13,-32),
s!(-19,-45),
s!(-35,-57),
];

pub const SQUARE_RULE: S = s!(-3266,203);

pub const FREE_PASSER: [S; 8] = [
s!(0,0),
s!(5,2),
s!(1,0),
s!(-7,17),
s!(-17,47),
s!(-25,126),
s!(-2,178),
s!(0,0),
];

pub const PROTECTED_PASSER: [S; 8] = [
s!(0,0),
s!(26,-26),
s!(22,-11),
s!(18,-3),
s!(19,9),
s!(37,17),
s!(49,-8),
s!(0,0),
];

pub const BISHOP_LONG_DIAGONAL: S = s!(13,5);

