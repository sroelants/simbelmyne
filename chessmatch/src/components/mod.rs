use chess::piece::Color;
use ratatui::{Frame, prelude::{Rect, Layout, Direction, Constraint}};

use crate::tui::State;
use self::game_history::GameHistory;
use self::board_view::BoardView;
use self::engine_info::EngineInfo;

mod board_view;
mod engine_info;
mod game_history;

struct LayoutChunks {
    board: Rect,
    history: Rect,
    white: Rect,
    black: Rect,
}

impl LayoutChunks {
    pub fn new(container: Rect) -> Self {
        let app_width = 120;
        let app_height = 40;

        // Get a centered rect to render the chunks into
        let vertically_centered_rect = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min((container.height - app_height) / 2),
                Constraint::Min(app_height),
                Constraint::Min((container.height - app_height) / 2),
            ])
            .split(container)[1];

        let centered_rect = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min((container.width - app_width) / 2),
                Constraint::Min(app_width),
                Constraint::Min((container.width - app_width) / 2),
            ])
            .split(vertically_centered_rect)[1];

        let horizontal_sections = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(80), Constraint::Min(40)])
            .split(centered_rect);

        let left_sections = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(app_height - 10), Constraint::Min(10)])
            .split(horizontal_sections[0]);

        let right_sections = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(horizontal_sections[1]);
        

        Self {
            board: left_sections[0],
            history: left_sections[1],
            white: right_sections[0],
            black: right_sections[1]
        }
    }
}

pub fn view(state: &mut State, f: &mut Frame) {
    let term_rect = f.size();

    let board_view = BoardView { board: state.board, highlights: Vec::new() };
    let chunks = LayoutChunks::new(term_rect);

    let white = &state.engines[Color::White as usize];
    let black = &state.engines[Color::Black as usize];

    let white_info = EngineInfo {
        name: white.config.name.clone(),
        tc: white.tc,
        active: state.board.current.is_white(),
        depth: white.search_info.depth.unwrap_or_default(),
        seldepth: white.search_info.seldepth.unwrap_or_default(),
        nodes_visited: white.search_info.nodes.unwrap_or_default(),
        score: white.search_info.score.unwrap_or_default(),
        nps: white.search_info.nps.unwrap_or_default(),
        hashfull: white.search_info.hashfull.unwrap_or_default(),
    };

    let black_info = EngineInfo {
        name: black.config.name.clone(),
        tc: black.tc,
        active: state.board.current.is_black(),
        depth: black.search_info.depth.unwrap_or_default(),
        seldepth: black.search_info.seldepth.unwrap_or_default(),
        nodes_visited: black.search_info.nodes.unwrap_or_default(),
        score: black.search_info.score.unwrap_or_default(),
        nps: black.search_info.nps.unwrap_or_default(),
        hashfull: black.search_info.hashfull.unwrap_or_default(),
    };

    let game_history = GameHistory {
        mode: state.mode.clone(),
        initial_board: state.initial_board,
        history: state.history.clone(),
    };

    f.render_widget(board_view, chunks.board);
    f.render_widget(white_info, chunks.white);
    f.render_widget(black_info, chunks.black);
    f.render_widget(game_history, chunks.history);
}


