#[derive(Debug, clap::ValueEnum, Clone, Copy)]
pub enum Preset {
    StartingPos,
    Kiwipete,
    Position3,
    Position4,
    Position5,
    Position6,
}


#[derive(Copy, Clone, Debug)]
pub struct PerftPreset<'a> {
    pub name: &'a str,
    pub description: &'a str,
    pub fen: &'a str,
    pub expected: &'a [usize],
}

impl Preset {
    const COUNT: usize = 6;

    const PRESETS: [PerftPreset<'static>; Preset::COUNT] = [
        PerftPreset {
            name: "Starting position",
            description: "All the pieces in their original position",
            fen: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            expected: &[1, 20, 400, 8902, 197_281, 4_865_609, 119_060_324,  3_195_901_860 ],
        },

        PerftPreset {
            name: "Kiwipete",
            description: "An infamous board state to week out any edge cases",
            fen: "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
            expected: &[1, 48, 2_039, 97_862, 4_085_603, 193_690_690, 8_031_647_685 ],
        },

        PerftPreset {
            name: "Position 3",
            description: "Position 3. Gave me a headache with EP checks",
            fen: "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
            expected: &[1, 14, 191, 2812, 43_238, 674_624, 1_103_0083, 178_633_661, 3_009_794_393],
        },

        PerftPreset {
            name: "Position 4",
            description: "An infamous board state to week out any edge cases",
            fen: "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
            expected: &[1, 6, 264, 9467, 422_333, 15_833_292, 706_045_033 ],
        },

        PerftPreset {
            name: "Position 5",
            description: "Meant to give some trouble even in the very early moves",
            fen: "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8  ",
            expected: &[1, 44, 1486, 62_379, 2_103_487, 89_941_194 ],
        },

        PerftPreset {
            name: "Position 6",
            description: "",
            fen: "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
            expected: &[1, 46, 2_079, 89_890, 3_894_594, 164_075_551, 6_923_051_137, 287_188_994_746,  11_923_589_843_526, 490_154_852_788_714  ],
        },
    ];

    pub fn load_preset(preset: Preset) -> PerftPreset<'static> {
        Preset::PRESETS[preset as usize]
    }
}
