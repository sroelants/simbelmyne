use crate::evaluate::S;

pub const PIECE_VALUES: [S; 6] = [
S(75,99),
S(330,383),
S(342,354),
S(464,629),
S(992,1177),
S(0,0),
];

pub const PAWN_PSQT: [S; 64] = [
S(0,0),     S(0,0),     S(0,0),     S(0,0),     S(0,0),     S(0,0),     S(0,0),     S(0,0),     
S(63,172),  S(85,162),  S(45,162),  S(80,125),  S(63,128),  S(77,130),  S(0,162),   S(-23,180), 
S(-2,40),   S(1,40),    S(36,27),   S(38,38),   S(52,12),   S(84,16),   S(64,43),   S(30,33),   
S(-18,31),  S(-3,23),   S(7,21),    S(12,6),    S(34,5),    S(26,9),    S(15,14),   S(6,8),     
S(-29,15),  S(-13,16),  S(1,13),    S(17,9),    S(18,8),    S(14,8),    S(1,5),     S(-9,0),    
S(-27,11),  S(-17,12),  S(-3,12),   S(1,16),    S(12,17),   S(9,12),    S(17,-1),   S(1,-5),    
S(-27,14),  S(-16,14),  S(-12,19),  S(-17,25),  S(-3,32),   S(3,20),    S(15,1),    S(-20,0),   
S(0,0),     S(0,0),     S(0,0),     S(0,0),     S(0,0),     S(0,0),     S(0,0),     S(0,0),      ];

pub const KNIGHT_PSQT: [S; 64] = [
S(-170,-78),S(-122,-24),S(-68,-4),  S(-36,-14), S(10,-14),  S(-66,-31), S(-49,-34), S(-116,-102),
S(-31,-19), S(-15,-4),  S(17,-2),   S(40,-3),   S(15,-7),   S(86,-26),  S(-10,-8),  S(13,-40),  
S(-12,-11), S(27,-1),   S(47,19),   S(58,21),   S(99,4),    S(104,-4),  S(56,-10),  S(19,-22),  
S(-10,2),   S(6,16),    S(32,30),   S(58,32),   S(36,32),   S(61,28),   S(19,15),   S(25,-8),   
S(-23,3),   S(-7,8),    S(16,30),   S(17,31),   S(28,34),   S(21,20),   S(17,5),    S(-9,-7),   
S(-41,-12), S(-17,2),   S(0,9),     S(11,22),   S(22,19),   S(9,2),     S(5,-6),    S(-19,-13), 
S(-48,-18), S(-38,-7),  S(-21,0),   S(-5,0),    S(-5,-2),   S(-7,-7),   S(-20,-21), S(-19,-15), 
S(-90,-25), S(-33,-31), S(-46,-12), S(-29,-11), S(-22,-11), S(-18,-24), S(-31,-23), S(-57,-33),  ];

pub const BISHOP_PSQT: [S; 64] = [
S(-30,5),   S(-49,12),  S(-53,6),   S(-89,16),  S(-66,10),  S(-56,2),   S(-18,0),   S(-54,-2),  
S(-14,-7),  S(-3,0),    S(-8,1),    S(-22,3),   S(6,-8),    S(8,-5),    S(0,2),     S(-3,-12),  
S(-4,10),   S(16,3),    S(12,7),    S(29,-3),   S(23,1),    S(54,5),    S(39,0),    S(25,7),    
S(-11,6),   S(1,11),    S(15,7),    S(32,19),   S(25,11),   S(19,12),   S(0,7),     S(-7,3),    
S(-9,2),    S(-8,11),   S(0,16),    S(22,14),   S(17,13),   S(0,10),    S(-6,6),    S(0,-6),    
S(-4,3),    S(8,9),     S(7,10),    S(6,10),    S(6,13),    S(9,6),     S(9,1),     S(12,-7),   
S(4,5),     S(10,-5),   S(15,-12),  S(-5,2),    S(6,0),     S(10,-6),   S(26,-4),   S(6,-13),   
S(-7,-10),  S(14,1),    S(-2,-3),   S(-12,1),   S(-8,0),    S(-11,6),   S(4,-12),   S(4,-28),    ];

pub const ROOK_PSQT: [S; 64] = [
S(5,28),    S(-8,34),   S(-2,40),   S(-3,35),   S(17,27),   S(36,23),   S(35,23),   S(45,19),   
S(-15,31),  S(-19,41),  S(1,42),    S(20,31),   S(7,31),    S(44,18),   S(37,16),   S(63,5),    
S(-30,28),  S(-2,24),   S(-11,26),  S(-11,22),  S(21,10),   S(41,1),    S(90,-7),   S(52,-6),   
S(-31,29),  S(-18,22),  S(-22,29),  S(-15,22),  S(-12,10),  S(8,2),     S(18,3),    S(13,0),    
S(-40,23),  S(-40,23),  S(-33,21),  S(-25,16),  S(-23,13),  S(-21,11),  S(1,1),     S(-13,2),   
S(-41,17),  S(-37,14),  S(-31,11),  S(-25,10),  S(-17,4),   S(-10,-2),  S(19,-20),  S(-4,-13),  
S(-41,10),  S(-32,12),  S(-21,11),  S(-19,9),   S(-12,1),   S(-8,-4),   S(11,-15),  S(-25,-3),  
S(-18,13),  S(-18,11),  S(-13,15),  S(-8,7),    S(-4,0),    S(-7,3),    S(2,-1),    S(-13,-1),   ];

pub const QUEEN_PSQT: [S; 64] = [
S(-39,21),  S(-42,36),  S(-22,53),  S(8,42),    S(8,44),    S(30,30),   S(57,-13),  S(2,22),    
S(-12,3),   S(-43,33),  S(-38,66),  S(-51,89),  S(-42,105), S(3,58),    S(-6,41),   S(40,32),   
S(-13,4),   S(-21,14),  S(-25,46),  S(-14,55),  S(0,67),    S(40,48),   S(55,11),   S(49,18),   
S(-29,16),  S(-23,21),  S(-23,32),  S(-23,51),  S(-22,65),  S(-8,57),   S(-2,53),   S(4,42),    
S(-22,8),   S(-28,30),  S(-23,25),  S(-18,43),  S(-18,40),  S(-15,37),  S(-7,33),   S(-2,36),   
S(-20,-6),  S(-14,10),  S(-15,21),  S(-14,12),  S(-14,19),  S(-6,19),   S(3,8),     S(-2,6),    
S(-16,-7),  S(-12,-9),  S(-3,-13),  S(-1,-7),   S(-3,-1),   S(2,-32),   S(9,-55),   S(19,-67),  
S(-21,-9),  S(-21,-11), S(-12,-15), S(-5,-4),   S(-7,-25),  S(-25,-21), S(-8,-38),  S(-13,-35),  ];

pub const KING_PSQT: [S; 64] = [
S(-49,-95), S(18,-56),  S(16,-36),  S(-22,-3),  S(-48,-7),  S(-35,-3),  S(25,-17),  S(-3,-97),  
S(-29,-28), S(-6,19),   S(-29,33),  S(24,29),   S(5,41),    S(7,50),    S(15,37),   S(-6,-3),   
S(-58,-9),  S(34,26),   S(-20,57),  S(-34,71),  S(-4,73),   S(50,63),   S(36,45),   S(-8,1),    
S(-41,-16), S(-47,31),  S(-64,64),  S(-99,89),  S(-96,89),  S(-66,71),  S(-60,45),  S(-90,6),   
S(-66,-20), S(-50,19),  S(-83,55),  S(-120,82), S(-118,81), S(-75,53),  S(-82,29),  S(-125,5),  
S(-26,-30), S(-1,3),    S(-51,34),  S(-65,49),  S(-60,47),  S(-57,34),  S(-24,7),   S(-50,-11), 
S(49,-45),  S(22,-12),  S(9,1),     S(-23,13),  S(-25,16),  S(-12,7),   S(29,-14),  S(21,-34),  
S(34,-85),  S(59,-56),  S(48,-37),  S(-33,-19), S(23,-42),  S(-17,-21), S(46,-49),  S(45,-88),   ];

pub const PASSED_PAWN_TABLE: [S; 64] = [
S(0,0),     S(0,0),     S(0,0),     S(0,0),     S(0,0),     S(0,0),     S(0,0),     S(0,0),     
S(10,71),   S(3,63),    S(26,67),   S(28,44),   S(23,40),   S(-13,58),  S(-12,69),  S(-7,70),   
S(32,162),  S(45,159),  S(24,123),  S(16,72),   S(10,92),   S(10,109),  S(-20,128), S(-41,155), 
S(27,90),   S(23,83),   S(21,54),   S(20,47),   S(3,46),    S(13,53),   S(-6,81),   S(-9,90),   
S(10,56),   S(3,47),    S(-16,31),  S(-7,21),   S(-18,25),  S(-8,30),   S(-11,55),  S(0,53),    
S(3,16),    S(-8,21),   S(-22,12),  S(-18,4),   S(-20,4),   S(-13,6),   S(-12,34),  S(11,17),   
S(-1,15),   S(1,15),    S(-16,11),  S(-17,6),   S(-5,-9),   S(-3,-1),   S(14,11),   S(6,16),    
S(0,0),     S(0,0),     S(0,0),     S(0,0),     S(0,0),     S(0,0),     S(0,0),     S(0,0),      ];

pub const KNIGHT_MOBILITY_BONUS: [S; 9] = [
S(-35,-73),
S(-26,-6),
S(-18,0),
S(-14,-1),
S(-12,2),
S(-13,7),
S(-14,8),
S(-15,8),
S(-13,3),
];

pub const BISHOP_MOBILITY_BONUS: [S; 14] = [
S(-47,-22),
S(-39,-6),
S(-30,1),
S(-27,12),
S(-20,26),
S(-13,38),
S(-7,42),
S(-3,49),
S(-3,56),
S(-1,54),
S(1,52),
S(6,51),
S(6,55),
S(32,41),
];

pub const ROOK_MOBILITY_BONUS: [S; 15] = [
S(-59,10),
S(-51,29),
S(-47,32),
S(-44,36),
S(-46,44),
S(-40,48),
S(-36,51),
S(-31,52),
S(-30,60),
S(-28,65),
S(-26,68),
S(-26,72),
S(-26,76),
S(-20,78),
S(-24,77),
];

pub const QUEEN_MOBILITY_BONUS: [S; 28] = [
S(-25,-87),
S(-26,-94),
S(-36,-26),
S(-34,-5),
S(-32,1),
S(-30,8),
S(-26,17),
S(-28,38),
S(-27,48),
S(-25,52),
S(-24,63),
S(-25,71),
S(-24,80),
S(-24,87),
S(-23,93),
S(-22,99),
S(-20,104),
S(-23,116),
S(-20,122),
S(-19,123),
S(-8,122),
S(-6,125),
S(-4,125),
S(6,122),
S(52,84),
S(107,72),
S(86,82),
S(92,100),
];

pub const VIRTUAL_MOBILITY_PENALTY: [S; 28] = [
S(23,-7),
S(33,-19),
S(30,-16),
S(27,-24),
S(23,-22),
S(22,-25),
S(18,-26),
S(13,-27),
S(7,-19),
S(-2,-18),
S(-10,-16),
S(-21,-13),
S(-31,-11),
S(-45,-9),
S(-58,-8),
S(-75,-6),
S(-86,-11),
S(-92,-11),
S(-94,-16),
S(-89,-24),
S(-94,-29),
S(-96,-34),
S(-96,-41),
S(-77,-51),
S(-75,-62),
S(-59,-71),
S(-56,-79),
S(-46,-78),
];

pub const ISOLATED_PAWN_PENALTY: S = S(-15,-14);

pub const DOUBLED_PAWN_PENALTY: S = S(-4,-19);

pub const BISHOP_PAIR_BONUS: S = S(22,67);

pub const ROOK_OPEN_FILE_BONUS: S = S(29,1);

pub const PAWN_SHIELD_BONUS: S = S(15,-10);


