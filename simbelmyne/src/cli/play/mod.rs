use self::tui::init_tui;

mod tui;
mod input_view;
mod info_view;
mod board_view;

pub fn run_play(fen: String, depth: usize) -> anyhow::Result<()> {
    init_tui(fen, depth)
}
