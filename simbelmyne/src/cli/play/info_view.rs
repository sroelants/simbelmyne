use chess::movegen::moves::Move;

use ratatui::{
    prelude::{Buffer, Constraint, Rect},
    style::{Style, Stylize},
    widgets::{Block, Borders, Cell, Padding, Row, Table, Widget},
};

pub struct InfoView {
    pub depth: usize,
    pub nodes_visited: u32,
    pub duration: u64,
    pub score: i32,
    pub best_move: Move,
    pub tt_occupancy: usize,
    pub tt_inserts: usize,
    pub tt_overwrites: usize,
}

impl Widget for InfoView {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let search_depth = Row::new(vec![
            Cell::from("Search depth").blue(),
            Cell::from(format!("{}", self.depth)),
        ]);

        let nodes_visited = Row::new(vec![
            Cell::from("Nodes visited").blue(),
            Cell::from(format!("{}", self.nodes_visited)),
        ]);

        let branching_factor = Row::new(vec![
            Cell::from("Branching Factor").blue(),
            Cell::from(format!("{:.2}", (self.nodes_visited as f32).powf(1.0/ self.depth as f32))),
        ]);

        let duration = if self.duration == 0 { 1 } else { self.duration };

        let search_speed = Row::new(vec![
            Cell::from("Search speed").blue(),
            Cell::from(format!(
                "{}knps", 
                self.nodes_visited / duration as u32
            )),
        ]);

        let duration = Row::new(vec![
            Cell::from("Duration").blue(),
            Cell::from(format!("{}ms", self.duration)),
        ]);


        let best_move = Row::new(vec![
            Cell::from("Best move").blue(),
            Cell::from(format!(
                "{}", 
                self.best_move.to_string()
            )),
        ]);

        let score = Row::new(vec![
            Cell::from("Score").blue(),
            Cell::from(format!("{}", self.score)),
        ]);

        let tt_occ = Row::new(vec![
            Cell::from("TT occupancy").blue(),
            Cell::from(format!("{}%", self.tt_occupancy)),
        ]);

        let tt_inserts = Row::new(vec![
            Cell::from("TT inserts").blue(),
            Cell::from(format!("{}", self.tt_inserts)),
        ]);

        let tt_overwrites = Row::new(vec![
            Cell::from("TT overwrites").blue(),
            Cell::from(format!("{}", self.tt_overwrites)),
        ]);


        let table = Table::new(vec![
            search_depth,
            nodes_visited,
            branching_factor,
            duration,
            search_speed,
            best_move,
            score,
            tt_occ,
            tt_inserts,
            tt_overwrites,
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
        .widths(&[Constraint::Min(20), Constraint::Min(20)]);

        Widget::render(table, area, buf);
    }

}
