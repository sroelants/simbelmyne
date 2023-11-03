use std::collections::HashSet;

use chess::{
    board::Board,
    movegen::moves::Move,
};
use crossterm::event::KeyCode;
use ratatui::{
    prelude::{CrosstermBackend, Direction, Layout, Rect},
    Frame, Terminal,
};
use ratatui::prelude::Constraint;

use crate::debug::engine::Engine;

use super::{
    diff_table::DiffTable,
    engine::{Simbelmyne, Stockfish}, board_view::BoardView, info_view::InfoView,
};

#[derive(Debug, Clone)]
pub struct Diff {
    pub mv: Move,
    pub found: Option<usize>,
    pub expected: Option<usize>,
}

pub struct State {
    stockfish: Stockfish,
    simbelmyne: Simbelmyne,
    move_list: Vec<Diff>,
    selected: usize,
    depth: usize,
    initial_board: Board,
    board_stack: Vec<Board>,
    should_quit: bool,
}

impl State {
    fn new(depth: usize, fen: String) -> State {
        let initial_board = fen.parse().unwrap();
        let stockfish = Stockfish::new().unwrap();
        let simbelmyne = Simbelmyne {};

        let mut state = State {
            stockfish,
            simbelmyne,
            move_list: vec![],
            selected: 0,
            depth,
            initial_board,
            board_stack: vec![initial_board],
            should_quit: false,
        };

        let move_list = state.get_diff().unwrap();
        state.move_list = move_list;

        state
    }

    fn get_diff(&mut self) -> anyhow::Result<Vec<Diff>> {
        let current_board = self.board_stack.last().unwrap();
        let depth = self.depth - (self.board_stack.len() - 1);

        let our_results = self.simbelmyne.perft(*current_board, depth)?;
        let stockfish_results = self.stockfish.perft(*current_board, depth)?;

        let moves = our_results
            .keys()
            .chain(stockfish_results.keys())
            .collect::<HashSet<_>>();

        let mut move_list = Vec::new();
        for mv in moves {
            move_list.push(Diff {
                mv: mv.parse().unwrap(),
                found: our_results.get(mv).copied(),
                expected: stockfish_results.get(mv).copied(),
            });
        }

        move_list.sort_by(|diff1, diff2| Ord::cmp(&diff1.mv.to_string(), &diff2.mv.to_string()));

        Ok(move_list)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Message {
    Up,
    Down,
    Select,
    Back,
    Quit,
}

fn view(state: &mut State, f: &mut Frame) {
    let term_rect = f.size();
    let layout = create_layout(term_rect);
    let current_board = state.board_stack.last().unwrap();

    let move_table = DiffTable {
        diffs: state.move_list.clone(),
        selected: state.selected,
    };

    let board_view = BoardView {
        board: *current_board,
    };

    let info_view = InfoView {
        starting_pos: state.initial_board.to_fen(),
        current_pos: current_board.to_fen(),
        search_depth: state.depth,
        current_depth: state.board_stack.len() - 1,
        total_found: state.move_list.iter().map(|d| d.found.unwrap_or(0)).sum(),
        total_expected: state
            .move_list
            .iter()
            .map(|d| d.expected.unwrap_or(0))
            .sum(),
    };

    f.render_widget(move_table, layout.table);
    f.render_widget(board_view, layout.board);
    f.render_widget(info_view, layout.info);
}

struct LayoutChunks {
    table: Rect,
    board: Rect,
    info: Rect,
}

fn create_layout(container: Rect) -> LayoutChunks {
    let app_width = 120;
    let app_height = 50;

    let centered_rect = Layout::default()
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
        .split(centered_rect)[1];

    let sections = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(40), Constraint::Min(50)])
        .split(centered_rect);

    let left_panel = sections[0];
    let right_panel = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(sections[1]);

    let board_panel = right_panel[0];
    let info_panel = right_panel[1];

    LayoutChunks {
        table: left_panel,
        board: board_panel,
        info: info_panel,
    }
}



fn initialize_panic_handler() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        crossterm::execute!(std::io::stderr(), crossterm::terminal::LeaveAlternateScreen).unwrap();
        crossterm::terminal::disable_raw_mode().unwrap();
        original_hook(panic_info);
    }));
}

fn handle_event(_: &State) -> anyhow::Result<Option<Message>> {
    let message = if crossterm::event::poll(std::time::Duration::from_millis(16))? {
        if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
            match key.code {
                KeyCode::Char('j') => Message::Down,
                KeyCode::Char('k') => Message::Up,
                KeyCode::Char('q') | KeyCode::Esc => Message::Quit,
                KeyCode::Char('h') => Message::Back,
                KeyCode::Char('l') | KeyCode::Enter => Message::Select,
                _ => return Ok(None),
            }
        } else {
            return Ok(None);
        }
    } else {
        return Ok(None);
    };

    Ok(Some(message))
}

fn update(state: &mut State, message: Message) -> Option<Message> {
    match message {
        Message::Up => {
            if 0 < state.selected {
                state.selected -= 1
            }
        }

        Message::Down => {
            if state.selected < state.move_list.len() - 1 {
                state.selected += 1
            }
        }

        Message::Quit => state.should_quit = true,

        Message::Select => {
            let current_depth = state.board_stack.len();
            if current_depth == state.depth {
                return None;
            }

            let current_board = state.board_stack.last().unwrap();
            let selected_move = state.move_list[state.selected].mv;

            let new_board = current_board.play_move(selected_move);
            state.board_stack.push(new_board);

            let new_move_list = state.get_diff().unwrap();
            state.move_list = new_move_list;
            state.selected = 0;
        }

        Message::Back => {
            let current_depth = state.board_stack.len();
            if current_depth == 1 {
                return None;
            }

            state.board_stack.pop();

            let new_move_list = state.get_diff().unwrap();
            state.move_list = new_move_list;
            state.selected = 0;
        }
    }

    None
}

pub fn init_tui(depth: usize, fen: String) -> anyhow::Result<()> {
    initialize_panic_handler();

    // Startup
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(std::io::stderr(), crossterm::terminal::EnterAlternateScreen)?;

    let mut terminal = Terminal::new(CrosstermBackend::new(std::io::stderr()))?;
    let mut state = State::new(depth, fen);

    loop {
        // Render the current view
        terminal.draw(|f| {
            view(&mut state, f);
        })?;

        // Handle events and map to a Message
        let mut current_msg = handle_event(&state)?;

        // Process updates as long as they return a non-None message
        while current_msg.is_some() {
            current_msg = update(&mut state, current_msg.unwrap());
        }

        // Exit loop if quit flag is set
        if state.should_quit {
            break;
        }
    }

    // Shutdown
    crossterm::execute!(std::io::stderr(), crossterm::terminal::LeaveAlternateScreen)?;
    crossterm::terminal::disable_raw_mode()?;
    Ok(())
}
