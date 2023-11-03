use ratatui::widgets::{
    Block, Borders, HighlightSpacing, Padding, Row, StatefulWidget, Table, TableState, Widget,
};
use ratatui::{
    prelude::{Alignment, Buffer, Constraint, Rect},
    style::{Style, Stylize},
};

use super::tui::Diff;

impl Diff {
    fn to_table_row(&self) -> Row {
        let mv = self.mv.to_string();
        let found = self
            .found
            .map(|found| found.to_string())
            .unwrap_or(String::from(""));
        let expected = self
            .expected
            .map(|found| found.to_string())
            .unwrap_or(String::from(""));

        Row::new(vec![mv, found, expected])
    }
}

pub struct DiffTable {
    pub diffs: Vec<Diff>,
    pub selected: usize,
}

impl Widget for DiffTable {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let border = Block::default()
            .borders(Borders::ALL)
            .title("Moves")
            .title_alignment(Alignment::Left)
            .border_style(Style::new().dark_gray())
            .title_style(Style::new().white())
            .padding(Padding::new(3, 3, 2, 2));

        let mut table_state = TableState::default().with_selected(Some(self.selected));
        let rows = self.diffs.iter().map(|diff| {
            diff.to_table_row().style(if diff.found == diff.expected {
                Style::default().dark_gray()
            } else {
                Style::default().red()
            })
        });

        let table = Table::new(rows)
            .header(Row::new(vec!["Move", "Found", "Expected"]).bold().blue())
            .block(Block::new().padding(Padding::new(2, 2, 2, 2)))
            .widths(&[
                Constraint::Length(5),
                Constraint::Length(10),
                Constraint::Length(10),
            ])
            .column_spacing(3)
            .highlight_style(Style::default().white())
            .highlight_spacing(HighlightSpacing::Always)
            .highlight_symbol("> ");

        border.render(area, buf);

        StatefulWidget::render(table, area, buf, &mut table_state);
    }
}
