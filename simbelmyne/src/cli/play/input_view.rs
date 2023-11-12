use ratatui::{widgets::{Widget, Paragraph, Block, Borders}, prelude::{Buffer, Rect}, style::{Style, Color, Stylize}};

use super::tui::InputMode;

pub struct InputView {
    pub input_mode: InputMode,
    pub input: String,
}

impl Widget for InputView {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let border = match self.input_mode {
            InputMode::Normal => Block::default()
                .borders(Borders::ALL)
                .title("Input (normal)"),

            InputMode::Insert => Block::default()
                .borders(Borders::ALL)
                .title("Input (insert)"),
        };

        let style = match self.input_mode {
            InputMode::Normal => Style::default().fg(Color::DarkGray),
            InputMode::Insert => Style::default().fg(Color::White),
        };

        let input = Paragraph::new(self.input)
            .style(style)
            .block(border);

        Widget::render(input, area, buf);

    }
}
