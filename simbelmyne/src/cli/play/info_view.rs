use std::time::Duration;

use chess::movegen::moves::Move;

use ratatui::{
    prelude::{Buffer, Constraint, Rect},
    style::{Style, Stylize},
    widgets::{Block, Borders, Cell, Padding, Row, Table, Widget},
};

pub struct InfoView {
    pub depth: usize,
    pub nodes_visited: usize,
    pub leaf_nodes: usize,
    pub beta_cutoffs: usize,
    pub duration: Duration,
    pub score: i32,
    pub best_move: Move,
    pub tt_hits: usize,
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

        let leaf_nodes = Row::new(vec![
            Cell::from("Leaf nodes").blue(),
            Cell::from(format!("{}", self.leaf_nodes)),
        ]);

        let beta_cutoffs = Row::new(vec![
            Cell::from("Beta cutoffs").blue(),
            Cell::from(format!("{}", self.beta_cutoffs)),
        ]);

        let branching_factor = Row::new(vec![
            Cell::from("Branching Factor").blue(),
            Cell::from(format!("{:.2}", (self.nodes_visited as f32).powf(1.0/ self.depth as f32))),
        ]);

        let std_branching_factor = Row::new(vec![
            Cell::from("(Alt) Branching Factor").blue(),
            Cell::from(
                format!("{:.2}",
                if self.nodes_visited == self.leaf_nodes {
                    0.0 
                } else { 
                    (self.nodes_visited - 1) as f32 / (self.nodes_visited - self.leaf_nodes) as f32
                }))
            ]);

        let third_branching_factor = Row::new(vec![
            Cell::from("3rd Branching Factor").blue(),
            Cell::from(format!("{:.2}", (self.leaf_nodes as f32).powf(1.0/ (self.depth as f32 - 1.0)))),
        ]);

        let duration = self.duration.as_millis();
        let duration = if duration == 0 { 1 } else { duration };

        let search_speed = Row::new(vec![
            Cell::from("Search speed").blue(),
            Cell::from(format!(
                "{}knps", 
                self.nodes_visited / duration as usize
            )),
        ]);

        let duration = Row::new(vec![
            Cell::from("Duration").blue(),
            Cell::from(format!("{}ms", self.duration.as_millis())),
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

        let tt_hits = Row::new(vec![
            Cell::from("TT Hits").blue(),
            Cell::from(format!("{}", self.tt_hits)),
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
            leaf_nodes,
            beta_cutoffs,
            branching_factor,
            std_branching_factor,
            third_branching_factor,
            duration,
            search_speed,
            best_move,
            score,
            tt_hits,
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
