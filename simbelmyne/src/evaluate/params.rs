use crate::evaluate::S;

pub const PIECE_VALUES: [S; 6] = [
S(62,105),
S(321,396),
S(339,367),
S(452,641),
S(925,1231),
S(0,0),
];

pub const PAWN_PSQT: [S; 64] = [
S(0,0),     S(0,0),     S(0,0),     S(0,0),     S(0,0),     S(0,0),     S(0,0),     S(0,0),     
S(71,174),  S(96,164),  S(55,164),  S(87,129),  S(71,133),  S(82,135),  S(22,166),  S(-1,182),  
S(4,38),    S(1,40),    S(41,29),   S(39,39),   S(52,14),   S(77,20),   S(54,49),   S(20,37),   
S(-12,27),  S(2,21),    S(6,18),    S(11,5),    S(38,4),    S(24,10),   S(18,14),   S(8,6),     
S(-26,11),  S(-10,14),  S(0,10),    S(12,7),    S(17,5),    S(9,7),     S(3,3),     S(-11,-3),  
S(-30,5),   S(-23,7),   S(-11,9),   S(-8,15),   S(0,15),    S(-5,10),   S(2,-5),    S(-10,-9),  
S(-26,11),  S(-15,12),  S(-10,17),  S(-20,23),  S(-11,29),  S(4,17),    S(16,0),    S(-17,-4),  
S(0,0),     S(0,0),     S(0,0),     S(0,0),     S(0,0),     S(0,0),     S(0,0),     S(0,0),      ];

pub const KNIGHT_PSQT: [S; 64] = [
S(-166,-83),S(-133,-21),S(-72,0),   S(-38,-12), S(-6,-7),   S(-66,-32), S(-105,-18),S(-117,-106),
S(-32,-20), S(-10,-3),  S(18,0),    S(38,1),    S(6,-1),    S(72,-20),  S(-7,-8),   S(0,-37),   
S(-9,-11),  S(34,0),    S(52,21),   S(55,25),   S(78,13),   S(84,3),    S(36,-2),   S(0,-16),   
S(-4,2),    S(11,17),   S(37,32),   S(65,33),   S(34,38),   S(60,32),   S(1,23),    S(21,-6),   
S(-16,2),   S(1,9),     S(25,31),   S(25,33),   S(35,35),   S(27,22),   S(23,6),    S(-3,-7),   
S(-34,-13), S(-9,2),    S(7,11),    S(17,24),   S(28,21),   S(15,4),    S(11,-4),   S(-13,-12), 
S(-41,-20), S(-31,-8),  S(-12,0),   S(1,0),     S(0,-1),    S(-1,-5),   S(-15,-19), S(-11,-14), 
S(-80,-29), S(-27,-33), S(-39,-12), S(-23,-11), S(-14,-11), S(-11,-23), S(-24,-22), S(-49,-36),  ];

pub const BISHOP_PSQT: [S; 64] = [
S(-30,2),   S(-53,11),  S(-53,3),   S(-92,14),  S(-77,10),  S(-64,2),   S(-29,2),   S(-66,-1),  
S(-14,-9),  S(1,-3),    S(-4,-1),   S(-20,2),   S(3,-7),    S(1,-4),    S(-12,2),   S(-15,-10), 
S(-3,7),    S(18,0),    S(15,6),    S(26,-1),   S(19,3),    S(41,10),   S(22,3),    S(7,11),    
S(-10,3),   S(5,9),     S(13,9),    S(31,22),   S(24,13),   S(18,13),   S(4,5),     S(-11,3),   
S(-5,0),    S(-8,11),   S(2,15),    S(23,15),   S(18,14),   S(4,9),     S(-1,5),    S(2,-8),    
S(-5,2),    S(9,9),     S(10,10),   S(9,10),    S(10,13),   S(12,5),    S(12,1),    S(14,-9),   
S(4,3),     S(12,-8),   S(18,-14),  S(0,0),     S(10,-1),   S(12,-6),   S(28,-5),   S(7,-14),   
S(-5,-12),  S(15,0),    S(0,-6),    S(-9,-1),   S(-2,-2),   S(-7,5),    S(2,-12),   S(10,-32),   ];

pub const ROOK_PSQT: [S; 64] = [
S(2,28),    S(-10,35),  S(-9,43),   S(-17,40),  S(-3,33),   S(22,26),   S(19,26),   S(29,23),   
S(-10,30),  S(-15,40),  S(4,42),    S(21,32),   S(5,33),    S(35,20),   S(27,17),   S(38,11),   
S(-18,25),  S(9,21),    S(2,23),    S(1,18),    S(30,7),    S(39,2),    S(78,-3),   S(32,-1),   
S(-18,26),  S(-4,19),   S(-6,25),   S(1,18),    S(1,7),     S(17,0),    S(19,3),    S(9,1),     
S(-28,20),  S(-30,21),  S(-20,19),  S(-10,13),  S(-9,10),   S(-14,9),   S(3,1),     S(-11,2),   
S(-30,15),  S(-26,12),  S(-19,9),   S(-14,9),   S(-6,3),    S(-3,-2),   S(22,-19),  S(1,-13),   
S(-30,8),   S(-20,10),  S(-9,9),    S(-8,7),    S(-1,0),    S(0,-5),    S(17,-16),  S(-16,-5),  
S(-7,11),   S(-8,9),    S(-2,14),   S(3,6),     S(6,0),     S(1,1),     S(7,0),     S(-4,0),     ];

pub const QUEEN_PSQT: [S; 64] = [
S(-46,31),  S(-50,44),  S(-37,67),  S(-13,56),  S(-24,61),  S(-15,57),  S(30,1),    S(-27,37),  
S(-6,7),    S(-31,34),  S(-34,73),  S(-47,95),  S(-57,118), S(-8,67),   S(-13,47),  S(29,41),   
S(-3,9),    S(-9,19),   S(-9,46),   S(-9,60),   S(-2,74),   S(10,63),   S(25,31),   S(3,41),    
S(-16,21),  S(-5,24),   S(-9,38),   S(-12,62),  S(-10,70),  S(-3,60),   S(2,59),    S(0,46),    
S(-6,13),   S(-13,37),  S(-7,33),   S(-2,49),   S(-2,47),   S(-1,41),   S(6,36),    S(5,37),    
S(-6,0),    S(1,16),    S(0,28),    S(1,19),    S(3,26),    S(7,24),    S(17,12),   S(7,13),    
S(-2,-3),   S(2,-4),    S(11,-7),   S(14,0),    S(12,4),    S(17,-28),  S(21,-50),  S(28,-63),  
S(-5,-5),   S(-6,-8),   S(2,-8),    S(9,2),     S(8,-18),   S(-10,-19), S(0,-35),   S(-2,-31),   ];

pub const KING_PSQT: [S; 64] = [
S(6,-110),  S(33,-60),  S(53,-44),  S(-38,0),   S(-21,-12), S(1,-11),   S(53,-24),  S(44,-116), 
S(-66,-19), S(13,17),   S(-9,30),   S(85,18),   S(36,36),   S(34,47),   S(65,30),   S(4,-3),    
S(-82,-5),  S(69,19),   S(7,50),    S(-8,66),   S(37,65),   S(98,52),   S(54,40),   S(-6,0),    
S(-40,-18), S(-14,23),  S(-31,55),  S(-80,82),  S(-65,79),  S(-34,60),  S(-39,36),  S(-106,6),  
S(-58,-24), S(-21,12),  S(-33,43),  S(-80,70),  S(-67,67),  S(-34,42),  S(-48,21),  S(-125,1),  
S(-23,-31), S(14,1),    S(-19,28),  S(-29,41),  S(-26,40),  S(-26,30),  S(-10,8),   S(-47,-10), 
S(40,-43),  S(11,-6),   S(-1,8),    S(-33,19),  S(-29,21),  S(-23,15),  S(18,-6),   S(17,-30),  
S(7,-79),   S(15,-43),  S(2,-19),   S(-72,-5),  S(-22,-22), S(-58,-5),  S(-4,-30),  S(12,-77),   ];

pub const PASSED_PAWN_TABLE: [S; 64] = [
S(0,0),     S(0,0),     S(0,0),     S(0,0),     S(0,0),     S(0,0),     S(0,0),     S(0,0),     
S(18,73),   S(14,65),   S(36,69),   S(35,48),   S(31,45),   S(3,63),    S(11,73),   S(4,72),    
S(34,166),  S(53,164),  S(25,128),  S(20,80),   S(19,100),  S(16,116),  S(-10,132), S(-33,162), 
S(27,94),   S(25,88),   S(26,60),   S(21,52),   S(0,51),    S(19,58),   S(-2,87),   S(-12,96),  
S(12,58),   S(5,50),    S(-11,34),  S(-3,23),   S(-16,28),  S(-4,34),   S(-7,58),   S(-1,56),   
S(6,18),    S(1,23),    S(-14,14),  S(-9,3),    S(-7,4),    S(-5,8),    S(2,37),    S(13,20),   
S(0,16),    S(6,15),    S(-9,10),   S(-8,7),    S(9,-10),   S(1,0),     S(19,13),   S(9,16),    
S(0,0),     S(0,0),     S(0,0),     S(0,0),     S(0,0),     S(0,0),     S(0,0),     S(0,0),      ];

pub const KNIGHT_MOBILITY_BONUS: [S; 9] = [
S(-32,-14),
S(-15,-10),
S(-8,-2),
S(-4,-4),
S(-2,0),
S(-3,4),
S(-5,5),
S(-5,6),
S(-3,1),
];

pub const BISHOP_MOBILITY_BONUS: [S; 14] = [
S(-38,-17),
S(-30,-7),
S(-23,2),
S(-20,12),
S(-13,26),
S(-7,40),
S(-2,43),
S(0,50),
S(0,57),
S(1,56),
S(2,54),
S(5,54),
S(2,57),
S(22,46),
];

pub const ROOK_MOBILITY_BONUS: [S; 15] = [
S(-46,17),
S(-37,35),
S(-33,40),
S(-29,44),
S(-32,52),
S(-25,55),
S(-22,58),
S(-19,61),
S(-19,69),
S(-17,73),
S(-17,76),
S(-17,81),
S(-20,85),
S(-18,88),
S(-24,87),
];

pub const QUEEN_MOBILITY_BONUS: [S; 28] = [
S(-9,-86),
S(-10,-77),
S(-19,0),
S(-16,16),
S(-15,21),
S(-12,31),
S(-9,40),
S(-11,60),
S(-10,71),
S(-8,75),
S(-8,86),
S(-9,93),
S(-10,101),
S(-10,107),
S(-9,113),
S(-9,118),
S(-8,123),
S(-11,134),
S(-9,140),
S(-8,141),
S(1,138),
S(3,141),
S(2,142),
S(5,144),
S(26,123),
S(82,106),
S(88,96),
S(100,110),
];

pub const VIRTUAL_MOBILITY_PENALTY: [S; 28] = [
S(36,-17),
S(43,-18),
S(37,-19),
S(34,-23),
S(30,-22),
S(29,-23),
S(25,-23),
S(19,-23),
S(14,-18),
S(4,-14),
S(-3,-12),
S(-12,-10),
S(-21,-8),
S(-35,-5),
S(-48,-4),
S(-64,-2),
S(-81,-4),
S(-93,-3),
S(-100,-7),
S(-95,-14),
S(-99,-18),
S(-104,-22),
S(-121,-25),
S(-95,-35),
S(-113,-41),
S(-80,-52),
S(-72,-59),
S(-35,-62),
];

pub const KING_ZONE_ATTACKS: [S; 16] = [
S(171,-53),
S(165,-47),
S(147,-39),
S(118,-40),
S(62,-23),
S(-3,5),
S(-79,38),
S(-123,47),
S(-169,6),
S(-192,19),
S(-173,7),
S(-175,-55),
S(-158,-76),
S(-98,-36),
S(-84,-29),
S(-28,-5),
];

pub const ISOLATED_PAWN_PENALTY: S = S(-8,-9);

pub const DOUBLED_PAWN_PENALTY: S = S(1,-19);

pub const CONNECTED_PAWN_BONUS: [S; 3] = [
S(-1,-7),
S(16,5),
S(29,8),
];

pub const PHALANX_PAWN_BONUS: [S; 3] = [
S(3,0),
S(12,4),
S(9,-1),
];

pub const BISHOP_PAIR_BONUS: S = S(23,67);

pub const ROOK_OPEN_FILE_BONUS: S = S(29,2);

pub const PAWN_SHIELD_BONUS: [S; 3] = [
S(25,-11),
S(19,-2),
S(18,-10),
];

pub const PAWN_STORM_BONUS: [S; 3] = [
S(-57,-37),
S(13,0),
S(-12,3),
];

