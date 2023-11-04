use ratatui::{
    prelude::{Buffer, Constraint, Rect},
    style::{Style, Stylize},
    widgets::{Block, Borders, Cell, Padding, Row, Table, Widget},
};

pub struct InfoView {
    pub starting_pos: String,
    pub current_pos: String,
    pub search_depth: usize,
    pub current_depth: usize,
    pub total_found: usize,
    pub total_expected: usize,
}

impl Widget for InfoView {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let starting_fen = Row::new(vec![
            Cell::from("Starting position").blue(),
            Cell::from(format!("{}", self.starting_pos)),
        ]);

        let current_fen = Row::new(vec![
            Cell::from("Current position").blue(),
            Cell::from(format!("{}", self.current_pos)),
        ]);

        let search_depth = Row::new(vec![
            Cell::from("Search depth").blue(),
            Cell::from(format!("{}", self.search_depth)),
        ]);

        let current_depth = Row::new(vec![
            Cell::from("Current depth").blue(),
            Cell::from(format!("{}", self.current_depth)),
        ]);

        let total_found = Row::new(vec![
            Cell::from("Total found").blue(),
            Cell::from(format!("{}", self.total_found)),
        ]);

        let total_expected = Row::new(vec![
            Cell::from("Total expected").blue(),
            Cell::from(format!("{}", self.total_expected)),
        ]);

        let table = Table::new(vec![
            starting_fen,
            current_fen,
            search_depth,
            current_depth,
            total_found,
            total_expected,
        ])
        .column_spacing(1)
        .block(
            Block::new()
                .title("Information")
                .borders(Borders::ALL)
                .title_style(Style::new().white())
                .border_style(Style::new().dark_gray())
                .padding(Padding::new(1, 1, 1, 1)),
        )
        .widths(&[Constraint::Length(20), Constraint::Min(100)]);

        Widget::render(table, area, buf);
    }
}
