use self::tui::init_tui;

mod board_view;
mod diff_table;
mod engine;
mod info_view;
mod tui;

pub fn run_debug(depth: usize, fen: String) -> anyhow::Result<()> {
    init_tui(depth, fen)
}
