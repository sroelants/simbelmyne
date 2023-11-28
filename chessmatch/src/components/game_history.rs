use chess::board::Board;
use chess::piece::Color;
use ratatui::widgets::Widget;
use ratatui::layout::Rect;
use ratatui::buffer::Buffer;
use ratatui::widgets::Paragraph;
use ratatui::text::Line;
use ratatui::style::Stylize;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;

use crate::tui::GameMode;
use crate::tui::GameResult;

pub struct GameHistory {
    pub mode: GameMode,
    pub initial_board: Board,
    pub history: Vec<GameResult>,
}

impl Widget for GameHistory {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let border = Block::default()
            .borders(Borders::ALL)
            .title("Game History");

        let num_games = match &self.mode {
            GameMode::Infinite => 0,
            GameMode::Suite(boards) => boards.len()
        };

        let current_game = Line::from(vec![ 
            "Current Game".blue(),
            " (".dark_gray(),
            (self.history.len()+1).to_string().dark_gray(),
            "/".dark_gray(),
            (num_games).to_string().dark_gray(),
            ")".dark_gray(),
            ":".into(),
        ]);

        let fen = Line::from(self.initial_board.to_fen());

        let white_wins = self.history
            .iter()
            .filter(|&res| res == &GameResult::Win(Color::White))
            .count();

        let black_wins = self.history
            .iter()
            .filter(|&res| res == &GameResult::Win(Color::Black))
            .count();

        let draws = self.history.len() - white_wins - black_wins;

        let results = Line::from("Results: ".blue());
        let white_wins = Line::from(vec![
            "White: ".into(),
            white_wins.to_string().into()
        ]);

        let black_wins = Line::from(vec![
            "Black: ".into(),
            black_wins.to_string().into()
        ]);

        let draws = Line::from(vec![
            "Draws: ".into(),
            draws.to_string().into()
        ]);
 
        let paragraph = Paragraph::new(vec![
            current_game,
            fen,
            Line::from(vec![]),
            results,
            white_wins,
            black_wins,
            draws,
        ])
            .block(border);

        paragraph.render(area, buf);
    }
}
