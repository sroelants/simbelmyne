use self::tui::init_tui;

pub mod tui;
pub mod engine;

pub fn run_debug(depth: usize, fen: String) -> anyhow::Result<()> {
    init_tui(depth, fen)
}

