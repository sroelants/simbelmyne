use crate::evaluate::S;

pub const PIECE_VALUES: [S; 6] = [
S(57,103),
S(328,391),
S(334,366),
S(438,625),
S(902,1213),
S(0,0),
];

pub const PAWN_PSQT: [S; 64] = [
S(0,0),     S(0,0),     S(0,0),     S(0,0),     S(0,0),     S(0,0),     S(0,0),     S(0,0),     
S(66,171),  S(89,162),  S(46,162),  S(75,128),  S(62,131),  S(70,134),  S(17,164),  S(-2,179),  
S(7,39),    S(-9,44),   S(27,32),   S(14,44),   S(41,19),   S(63,23),   S(46,51),   S(18,39),   
S(-11,29),  S(-4,22),   S(-1,19),   S(0,6),     S(26,6),    S(17,10),   S(8,16),    S(9,8),     
S(-19,12),  S(-14,15),  S(-1,10),   S(5,7),     S(10,5),    S(5,7),     S(-1,5),    S(-7,-1),   
S(-26,7),   S(-17,6),   S(-11,9),   S(-8,15),   S(2,14),    S(-9,11),   S(5,-5),    S(-9,-7),   
S(-17,11),  S(-8,12),   S(0,15),    S(-11,19),  S(0,26),    S(12,15),   S(21,-1),   S(-8,-3),   
S(0,0),     S(0,0),     S(0,0),     S(0,0),     S(0,0),     S(0,0),     S(0,0),     S(0,0),      ];

pub const KNIGHT_PSQT: [S; 64] = [
S(-103,-37),S(-105,-5), S(-55,8),   S(-19,-3),  S(10,1),    S(-42,-22), S(-82,-2),  S(-62,-58), 
S(0,-1),    S(10,5),    S(30,2),    S(45,4),    S(21,0),    S(79,-17),  S(20,0),    S(33,-19),  
S(10,-2),   S(35,2),    S(36,19),   S(44,22),   S(63,12),   S(73,1),    S(38,0),    S(20,-5),   
S(17,11),   S(16,16),   S(32,27),   S(61,30),   S(34,33),   S(52,28),   S(8,23),    S(45,3),    
S(8,15),    S(10,9),    S(17,28),   S(25,29),   S(28,33),   S(27,20),   S(34,7),    S(23,9),    
S(-10,0),   S(-2,4),    S(0,10),    S(1,24),    S(15,21),   S(2,5),     S(18,1),    S(5,2),     
S(-10,1),   S(-5,3),    S(-2,0),    S(9,2),     S(7,0),     S(6,0),     S(7,-1),    S(10,12),   
S(-46,8),   S(-2,-12),  S(-21,-4),  S(-9,-4),   S(0,-1),    S(0,-9),    S(-2,0),    S(-15,4),    ];

pub const BISHOP_PSQT: [S; 64] = [
S(-25,0),   S(-61,10),  S(-55,5),   S(-90,14),  S(-74,9),   S(-66,0),   S(-35,1),   S(-59,-4),  
S(-11,-10), S(-3,-6),   S(-11,-2),  S(-19,1),   S(0,-8),    S(-2,-6),   S(-15,0),   S(-12,-9),  
S(-5,6),    S(10,-2),   S(6,4),     S(17,-3),   S(7,1),     S(36,5),    S(14,1),    S(6,8),     
S(-13,2),   S(0,7),     S(7,4),     S(27,18),   S(18,9),    S(10,11),   S(6,1),     S(-13,5),   
S(-2,0),    S(-14,8),   S(0,12),    S(17,12),   S(15,11),   S(0,6),     S(-6,5),    S(13,-10),  
S(-1,2),    S(10,7),    S(3,8),     S(7,10),    S(7,13),    S(6,5),     S(9,-3),    S(17,-4),   
S(22,9),    S(7,-9),    S(18,-11),  S(-3,0),    S(3,3),     S(7,-5),    S(23,-4),   S(17,-6),   
S(8,-4),    S(23,3),    S(5,-7),    S(-8,-2),   S(-2,-5),   S(-5,8),    S(9,-7),    S(26,-23),   ];

pub const ROOK_PSQT: [S; 64] = [
S(-7,26),   S(-22,33),  S(-28,42),  S(-35,39),  S(-23,33),  S(10,25),   S(7,26),    S(18,22),   
S(-8,25),   S(-13,35),  S(2,39),    S(18,29),   S(4,30),    S(30,17),   S(28,13),   S(35,8),    
S(-12,22),  S(16,19),   S(10,21),   S(13,17),   S(37,7),    S(49,0),    S(79,-5),   S(40,-4),   
S(-12,24),  S(4,18),    S(4,24),    S(10,17),   S(13,7),    S(24,0),    S(27,2),    S(16,0),    
S(-21,18),  S(-23,19),  S(-13,16),  S(-2,11),   S(-6,10),   S(-9,8),    S(10,0),    S(-5,0),    
S(-27,12),  S(-20,7),   S(-15,4),   S(-13,4),   S(-3,-1),   S(-1,-7),   S(26,-23),  S(2,-16),   
S(-27,3),   S(-18,4),   S(-6,1),    S(-4,-1),   S(0,-7),    S(3,-12),   S(19,-20),  S(-15,-10), 
S(-10,4),   S(-7,3),    S(-2,7),    S(2,0),     S(7,-6),    S(-3,-3),   S(7,-5),    S(-5,-6),    ];

pub const QUEEN_PSQT: [S; 64] = [
S(-39,17),  S(-62,44),  S(-55,73),  S(-36,66),  S(-45,69),  S(-36,66),  S(10,8),    S(-27,31),  
S(3,-2),    S(-27,30),  S(-33,68),  S(-46,92),  S(-56,114), S(-10,64),  S(-10,41),  S(40,29),   
S(3,9),     S(-5,20),   S(-9,53),   S(-7,63),   S(-16,83),  S(16,60),   S(21,34),   S(14,37),   
S(-8,19),   S(-7,35),   S(-8,42),   S(-14,65),  S(-15,77),  S(-1,60),   S(6,61),    S(6,46),    
S(-2,17),   S(-11,37),  S(-8,38),   S(-3,52),   S(-6,57),   S(-2,46),   S(7,37),    S(14,32),   
S(0,-5),    S(2,11),    S(-3,27),   S(0,25),    S(0,30),    S(5,22),    S(18,4),    S(15,4),    
S(7,-18),   S(4,-13),   S(11,-11),  S(12,-3),   S(10,0),    S(14,-27),  S(20,-52),  S(35,-68),  
S(1,-21),   S(2,-20),   S(7,-19),   S(13,-10),  S(11,-29),  S(-9,-24),  S(2,-37),   S(13,-48),   ];

pub const KING_PSQT: [S; 64] = [
S(56,-121), S(56,-65),  S(86,-51),  S(-38,-1),  S(15,-21),  S(20,-17),  S(76,-32),  S(166,-142),
S(-71,-19), S(27,13),   S(5,25),    S(109,11),  S(56,30),   S(50,41),   S(75,26),   S(5,-6),    
S(-82,-7),  S(71,17),   S(19,46),   S(3,62),    S(49,60),   S(106,47),  S(61,36),   S(-6,-1),   
S(-38,-20), S(-6,20),   S(-21,51),  S(-59,77),  S(-48,73),  S(-21,57),  S(-28,32),  S(-101,4),  
S(-52,-26), S(-12,9),   S(-20,39),  S(-60,65),  S(-48,62),  S(-23,38),  S(-40,18),  S(-122,0),  
S(-24,-32), S(19,-1),   S(-13,25),  S(-20,38),  S(-16,37),  S(-20,27),  S(-5,5),    S(-48,-12), 
S(37,-45),  S(11,-7),   S(2,6),     S(-28,17),  S(-26,20),  S(-18,13),  S(19,-8),   S(15,-32),  
S(2,-79),   S(12,-44),  S(0,-21),   S(-67,-6),  S(-17,-22), S(-56,-6),  S(-6,-31),  S(7,-78),    ];

pub const PASSED_PAWN_TABLE: [S; 64] = [
S(0,0),     S(0,0),     S(0,0),     S(0,0),     S(0,0),     S(0,0),     S(0,0),     S(0,0),     
S(13,70),   S(7,63),    S(27,67),   S(23,47),   S(22,43),   S(-2,62),   S(6,71),    S(-1,69),   
S(30,162),  S(55,157),  S(24,123),  S(33,73),   S(13,95),   S(16,113),  S(-10,128), S(-33,156), 
S(26,91),   S(21,86),   S(27,58),   S(22,51),   S(0,50),    S(16,58),   S(-7,86),   S(-12,93),  
S(10,57),   S(5,47),    S(-12,35),  S(-3,24),   S(-16,28),  S(-6,34),   S(-6,56),   S(-3,55),   
S(7,16),    S(-3,24),   S(-12,13),  S(-8,3),    S(-9,4),    S(-4,8),    S(-3,36),   S(11,19),   
S(0,15),    S(1,15),    S(-12,10),  S(-5,6),    S(5,-9),    S(-2,0),    S(11,13),   S(7,16),    
S(0,0),     S(0,0),     S(0,0),     S(0,0),     S(0,0),     S(0,0),     S(0,0),     S(0,0),      ];

pub const KNIGHT_MOBILITY_BONUS: [S; 9] = [
S(-59,-62),
S(-78,-79),
S(-53,-45),
S(-41,-24),
S(-29,-15),
S(-24,-4),
S(-12,-4),
S(-3,-1),
S(12,-9),
];

pub const BISHOP_MOBILITY_BONUS: [S; 14] = [
S(-52,-25),
S(-68,-55),
S(-39,-18),
S(-28,4),
S(-16,15),
S(-6,24),
S(0,34),
S(5,39),
S(6,44),
S(11,44),
S(13,45),
S(25,38),
S(25,40),
S(32,31),
];

pub const ROOK_MOBILITY_BONUS: [S; 15] = [
S(-69,-34),
S(-56,25),
S(-40,17),
S(-33,32),
S(-27,43),
S(-23,50),
S(-22,57),
S(-19,62),
S(-13,62),
S(-8,66),
S(-4,70),
S(-3,75),
S(-1,78),
S(1,78),
S(3,77),
];

pub const QUEEN_MOBILITY_BONUS: [S; 28] = [
S(-89,68),
S(-88,40),
S(-86,27),
S(-75,-16),
S(-62,-34),
S(-34,-21),
S(-26,1),
S(-25,36),
S(-21,52),
S(-18,69),
S(-13,71),
S(-10,75),
S(-7,83),
S(-3,83),
S(-1,87),
S(0,95),
S(0,98),
S(1,104),
S(1,112),
S(3,112),
S(11,114),
S(17,104),
S(33,101),
S(47,88),
S(42,97),
S(163,33),
S(90,60),
S(78,55),
];

pub const VIRTUAL_MOBILITY_PENALTY: [S; 28] = [
S(33,-14),
S(40,-19),
S(34,-16),
S(31,-19),
S(27,-17),
S(25,-19),
S(21,-19),
S(15,-18),
S(11,-13),
S(1,-10),
S(-6,-7),
S(-15,-5),
S(-24,-4),
S(-39,-1),
S(-52,0),
S(-67,1),
S(-86,0),
S(-99,0),
S(-105,-3),
S(-102,-10),
S(-108,-14),
S(-115,-18),
S(-133,-20),
S(-110,-30),
S(-131,-36),
S(-101,-46),
S(-102,-52),
S(-62,-55),
];

pub const KING_ZONE_ATTACKS: [S; 16] = [
S(170,-55),
S(164,-48),
S(147,-43),
S(120,-44),
S(66,-29),
S(0,-1),
S(-78,35),
S(-158,76),
S(-270,106),
S(-463,269),
S(-735,602),
S(-506,398),
S(-311,-137),
S(-140,-48),
S(-84,-29),
S(-28,-5),
];

pub const ISOLATED_PAWN_PENALTY: S = S(-7,-9);

pub const DOUBLED_PAWN_PENALTY: S = S(3,-18);

pub const CONNECTED_PAWN_BONUS: [S; 3] = [
S(-5,-8),
S(11,4),
S(26,6),
];

pub const PHALANX_PAWN_BONUS: [S; 3] = [
S(2,-1),
S(9,3),
S(6,-1),
];

pub const BISHOP_PAIR_BONUS: S = S(24,67);

pub const ROOK_OPEN_FILE_BONUS: S = S(26,2);

pub const PAWN_SHIELD_BONUS: [S; 3] = [
S(23,-11),
S(18,-2),
S(17,-9),
];

pub const PAWN_STORM_BONUS: [S; 3] = [
S(-48,-37),
S(16,0),
S(-10,4),
];

