use ratatui::{
    prelude::{Buffer, Constraint, Rect},
    style::{Style, Stylize},
    widgets::{Block, Borders, Cell, Padding, Row, Table, Widget},
};

use shared::uci::TCType;

pub struct EngineInfo {
    pub name: String,
    pub active: bool,
    pub tc: TCType,
    pub depth: u8,
    pub seldepth: u8,
    pub nodes_visited: u32,
    pub score: i32,
    pub nps: u32,
    pub hashfull: u32,
}

impl Widget for EngineInfo {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let tc = match self.tc {
            TCType::Depth(depth) => {
                Row::new(vec![
                    Cell::from("Max depth").blue(),
                    Cell::from(format!("{}", depth)),
                ])
            },

            TCType::Nodes(nodes) => {
                Row::new(vec![
                    Cell::from("Max nodes").blue(),
                    Cell::from(format!("{}", nodes)),
                ])
            },

            TCType::FixedTime(time) => {
                Row::new(vec![
                    Cell::from("Max time").blue(),
                    Cell::from(format!("{} ms", time.as_millis())),
                ])
            },

            TCType::VariableTime { wtime, .. } => {
                Row::new(vec![
                    Cell::from("Max time").blue(),
                    Cell::from(format!("{} ms", wtime.as_millis())),
                ])
            },

            TCType::Infinite => {
                Row::new(vec![
                    Cell::from("Max time").blue(),
                    Cell::from(format!("inf")),
                ])
            },
        };

        let depth = Row::new(vec![
            Cell::from("Search depth").blue(),
            Cell::from(format!("{}", self.depth)),
        ]);

        let seldepth = Row::new(vec![
            Cell::from("Selective depth").blue(),
            Cell::from(format!("{}", self.seldepth)),
        ]);

        let nodes_visited = Row::new(vec![
            Cell::from("Nodes visited").blue(),
            Cell::from(format!("{}", self.nodes_visited)),
        ]);

        let nps = Row::new(vec![
            Cell::from("nps").blue(),
            Cell::from(format!(
                "{}", 
                self.nps
                
            )),
        ]);

        let score = Row::new(vec![
            Cell::from("Score").blue(),
            Cell::from(format!("{}", self.score)),
        ]);

        let hashfull = Row::new(vec![
            Cell::from("Hashfull").blue(),
            Cell::from(format!("{}%", self.hashfull / 10)),
        ]);

        let style = if self.active { 
            Style::new().white() 
        } else { 
            Style::new().dark_gray() 
        } ;

        let table = Table::new(vec![
            tc,
            depth,
            seldepth,
            nodes_visited,
            score,
            nps,
            hashfull,
        ])
        .column_spacing(1)
        .block(
            Block::new()
                .title(self.name)
                .borders(Borders::ALL)
                .title_style(style)
                .border_style(style)
                .padding(Padding::new(1, 1, 1, 1)),
        )
        .widths(&[Constraint::Min(20), Constraint::Min(20)]);

        Widget::render(table, area, buf);
    }
}
