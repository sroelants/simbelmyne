use chess::board::Board;
use chess::piece::Piece;
use chess::square::Square;
use ratatui::{
    prelude::{Buffer, Constraint, Rect},
    style::{Style, Stylize},
    text::Line,
    widgets::{Block, Borders, Cell, Row, Table, Widget},
};

use super::centered;

pub struct BoardView {
    pub board: Board,
    pub highlights: Vec<Square>
}

fn square_to_cell(piece: Option<Piece>) -> Cell<'static> {
    match piece {
        Some(piece) => to_padded_cell(piece.to_string()),
        None => to_padded_cell(String::from("")),
    }
}

const CELL_WIDTH: usize = 5;
const CELL_HEIGHT: usize = 3;

fn to_padded_cell(val: String) -> Cell<'static> {
    let lines = vec![
        vec![Line::from(""); CELL_HEIGHT / 2],
        vec![Line::from(format!("{:^CELL_WIDTH$}", val))],
        vec![Line::from(""); CELL_HEIGHT / 2],
    ]
    .concat();

    Cell::from(lines)
}

impl Widget for BoardView {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let width = 10 * CELL_WIDTH as u16;
        let height = 10 * CELL_HEIGHT as u16;

        let rect = centered(area, width, height);

        let file_labels: Vec<_> = vec!["", "a", "b", "c", "d", "e", "f", "g", "h", ""]
            .into_iter()
            .map(|label| to_padded_cell(label.to_owned()))
            .collect();

        let file_labels = Row::new(file_labels).height(CELL_HEIGHT as u16).dark_gray();

        let mut rows: Vec<Row> = Vec::new();
        // Push top heading
        rows.push(file_labels.clone());

        let mut current_rank: Vec<Cell> = Vec::new();
        let ranks = self.board.piece_list.chunks(8);
        let ranks = ranks
            .into_iter()
            .enumerate()
            .collect::<Vec<_>>()
            .into_iter()
            .rev();

        for (rank, squares) in ranks {
            let rank_label = to_padded_cell((rank + 1).to_string()).dark_gray();
            current_rank.push(rank_label.clone());

            for (file, &square) in squares.iter().enumerate() {
                let cell = if (file + rank) % 2 == 0 {
                    // Dark squares
                    if self.highlights.contains(&Square::new(rank, file)) {
                        square_to_cell(square).on_light_blue()
                    } else {
                        square_to_cell(square)
                    }
                } else {
                    // Light squares
                    if self.highlights.contains(&Square::new(rank, file)) {
                        square_to_cell(square).on_light_blue()
                    } else {
                        square_to_cell(square).on_dark_gray()
                    }
                };

                current_rank.push(cell);
            }

            current_rank.push(rank_label);

            rows.push(Row::new(current_rank).height(CELL_HEIGHT as u16));
            current_rank = Vec::new();
        }

        // Push bottom heading
        rows.push(file_labels);

        let table = Table::new(rows)
            .widths(&[Constraint::Length(CELL_WIDTH as u16); 10])
            .column_spacing(0);

        let border = Block::new()
            .title("Board")
            .borders(Borders::ALL)
            .title_style(Style::new().white())
            .border_style(Style::new().dark_gray());

        Widget::render(border, area, buf);
        Widget::render(table, rect, buf);
    }
}
