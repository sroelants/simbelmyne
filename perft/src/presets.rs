#[derive(Debug, clap::ValueEnum, Clone, Copy)]
pub enum Preset {
    StartingPos,
    Kiwipete,
    Position3,
}


#[derive(Copy, Clone, Debug)]
pub struct PerftPreset<'a> {
    pub name: &'a str,
    pub description: &'a str,
    pub fen: &'a str,
    pub expected: &'a [usize],
}

impl Preset {
    const COUNT: usize = 3;

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
            description: "An infamous board state to week out any edge cases",
            fen: "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
            expected: &[1, 14, 191, 2812, 43_238, 674_624, 1_103_0083, 178_633_661, 3_009_794_393],
        },
    ];

    pub fn load_preset(preset: Preset) -> PerftPreset<'static> {
        Preset::PRESETS[preset as usize]
    }
}
