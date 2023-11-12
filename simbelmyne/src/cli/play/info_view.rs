use std::time::Duration;

use chess::movegen::moves::Move;

use ratatui::{
    prelude::{Buffer, Constraint, Rect},
    style::{Style, Stylize},
    widgets::{Block, Borders, Cell, Padding, Row, Table, Widget},
};

use crate::search::Score;

pub struct InfoView {
    pub depth: usize,
    pub nodes_visited: Option<usize>,
    pub duration: Option<Duration>,
    pub checkmates: Option<usize>,
    pub score: Option<Score>,
    pub best_move: Option<Move>
}

impl Widget for InfoView {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let search_depth = Row::new(vec![
            Cell::from("Search depth").blue(),
            Cell::from(format!("{}", self.depth)),
        ]);

        let nodes_visited = Row::new(vec![
            Cell::from("Nodes visited").blue(),
            Cell::from(format!("{}", self.nodes_visited.unwrap_or(0))),
        ]);

        let checkmates = Row::new(vec![
            Cell::from("Checkmates").blue(),
            Cell::from(format!("{}", self.checkmates.unwrap_or(0))),
        ]);

        let duration = Row::new(vec![
            Cell::from("Duration").blue(),
            Cell::from(format!("{}ms", self.duration.map(|d| d.as_millis()).unwrap_or(0))),
        ]);

        let search_speed = Row::new(vec![
            Cell::from("Search speed").blue(),
            Cell::from(format!(
                "{}knps", 
                self.nodes_visited.unwrap_or(0) / self.duration.map(|d| d.as_millis()).unwrap_or(1) as usize
            )),
        ]);

        let best_move = Row::new(vec![
            Cell::from("Best move").blue(),
            Cell::from(format!(
                "{}", 
                self.best_move
                    .map(|mv| mv.to_string())
                    .unwrap_or("".to_string())
            )),
        ]);

        let score = Row::new(vec![
            Cell::from("Score").blue(),
            Cell::from(format!("{}", self.score.unwrap_or(0))),
        ]);

        let table = Table::new(vec![
            search_depth,
            nodes_visited,
            checkmates,
            duration,
            search_speed,
            best_move,
            score,
        ])
        .column_spacing(1)
        .block(
            Block::new()
                .title("Search results")
                .borders(Borders::ALL)
                .title_style(Style::new().white())
                .border_style(Style::new().dark_gray())
                .padding(Padding::new(1, 1, 1, 1)),
        )
        .widths(&[Constraint::Min(15), Constraint::Min(25)]);

        Widget::render(table, area, buf);
    }

}
