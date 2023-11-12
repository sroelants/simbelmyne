use std::time::{Duration, Instant};

use chess::{board::{Board, Square}, movegen::moves::Move};
use crossterm::event::{KeyCode, self, KeyEvent};
use ratatui::{Frame, Terminal, prelude::{CrosstermBackend, Rect, Direction, Constraint, Layout}, widgets::Paragraph, style::{Color, Style}};
use tui_input::{self, backend::crossterm::EventHandler};

use crate::{search::{SearchResult, BoardState}, cli::components::board_view::BoardView};

use super::{input_view::InputView, info_view::InfoView};

pub struct State {
    us: chess::board::Color,
    play_state: PlayState,

    board_history: Vec<Board>,
    cursor: usize,

    error: Option<String>,

    search_depth: usize,
    search_result:  Option<SearchResult>,
    search_duration: Option<Duration>,

    input: tui_input::Input,
    input_mode: InputMode,

    should_quit: bool,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum InputMode {
    Normal,
    Insert
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PlayState {
    Idle,
    Selected(Square),
}

impl State {
    fn new() -> State {
        State {
            us: chess::board::Color::White,
            search_depth: 2,
            play_state: PlayState::Idle,
            board_history: vec![Board::new()],
            cursor: 0,
            error: None,
            search_result: None,
            search_duration: None,
            input: tui_input::Input::default(),
            input_mode: InputMode::Insert,
            should_quit: false
        }
    }

    fn current_board(&self) -> Board {
        self.board_history[self.cursor]
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
enum Message {
    NormalMode,
    InsertMode,
    Submit,
    Error(String),
    Select(Square),
    Input(KeyEvent),
    PlayMove(Move),
    PlayOpponentMove,
    GoBack,
    GoBackToStart,
    GoForward,
    GoForwardToEnd,
    Quit,
}

fn update(state: &mut State, message: Message) -> Option<Message> {
    match message {
        Message::Quit => {
            state.should_quit = true;
            None
        },

        Message::NormalMode => {
            state.input_mode = InputMode::Normal;
            None
        },

        Message::InsertMode => {
            state.input_mode = InputMode::Insert;
            None
        },

        Message::GoBack => {
            if state.cursor > 0 {
                state.cursor -= 1;
            }

            None
        },

        Message::GoBackToStart => {
            state.cursor = 0;

            None
        }


        Message::GoForward => {
            if state.cursor < state.board_history.len() - 1 {
                state.cursor += 1;
            }

            None
        },

        Message::GoForwardToEnd => {
            state.cursor = state.board_history.len() - 1;
            None
        }

        Message::Submit => {
            if state.us != state.current_board().current {
                return Some(Message::Error("It's not your turn!".to_string()));
            }

            let input = state.input.to_string();
            state.input.reset();
            let sq: anyhow::Result<Square> = input.parse();

            if let Err(err) = sq {
                return Some(Message::Error(err.to_string()));
            }

            let sq = sq.unwrap();

            match state.play_state {
                PlayState::Idle => {
                    Some(Message::Select(sq))
                },

                PlayState::Selected(src) => {
                    let legal_moves = state.current_board().legal_moves();
                    let mv = legal_moves.iter()
                        .find(|mv| mv.src() == src && mv.tgt() == sq);

                    if let Some(mv) = mv {
                        Some(Message::PlayMove(*mv))
                    } else {
                        Some(Message::Error(String::from("Not a valid move!")))
                    }
                }
            }
        },

        Message::Select(sq) => {
            if let Some(piece) = state.current_board().get_at(sq) {
                if piece.color() != state.current_board().current {
                    return Some(Message::Error("That's not one of your pieces!".to_string()))
                }

                state.play_state = PlayState::Selected(sq);
                state.error = None;

                None
            } else {
                Some(Message::Error(format!("There's no piece on {sq}")))
            }
        }

        Message::PlayMove(mv) => {
            // If we're in the past, blow away any future boards on the stack
            state.board_history.truncate(state.cursor + 1);
            let new_board = state.current_board().play_move(mv);
            state.board_history.push(new_board);
            state.cursor += 1;
            state.play_state = PlayState::Idle;
            state.error = None;

            Some(Message::PlayOpponentMove)
        },

        Message::PlayOpponentMove => {
            let search = BoardState::new(state.current_board());

            let start = Instant::now();
            let search_result = search.search(state.search_depth);
            let duration = start.elapsed();

            state.search_result = Some(search_result);
            state.search_duration = Some(duration);

            let new_board = state.current_board().play_move(search_result.best_move);
            state.board_history.push(new_board);
            state.cursor += 1;

            None
        },

        Message::Error(msg) => {
            state.error = Some(msg);
            None
        },

        Message::Input(key) => {
            state.input.handle_event(&event::Event::Key(key));
            None
        },

        _ => None
    }
}

fn view(state: &mut State, f: &mut Frame) {
    let term_rect = f.size();
    let layout_chunks = create_layout(term_rect);

    let current_board = state.current_board();

    let highlights: Vec<Square> = match state.play_state {
        PlayState::Idle => vec![],
        PlayState::Selected(sq) => {
            if let Some(piece) = current_board.get_at(sq) {
                let us = state.us;
                let ours = current_board.occupied_by(us);
                let theirs = current_board.occupied_by(us.opp());

                (piece.visible_squares(ours, theirs) & !ours).collect()
            } else {
                vec![]
            }
        }
    };
    let board_view = BoardView { 
        board: current_board,
        highlights,
    };

    let input_view = InputView { 
        input: state.input.value().to_string(), 
        input_mode: state.input_mode,
    };

    f.render_widget(board_view, layout_chunks.board);
    f.render_widget(input_view, layout_chunks.input);

    if let Some(msg) = &state.error {
        let error = Paragraph::new(format!("ERROR: {msg}"))
          .style(Style::default().fg(Color::Red));

        f.render_widget(error, layout_chunks.error);
    }

    
    let info_view = InfoView {
        depth: state.search_depth,
        duration: state.search_duration,
        nodes_visited: state.search_result.map(|res| res.nodes_visited),
        checkmates: state.search_result.map(|res| res.checkmates),
        score: state.search_result.map(|res| res.score),
        best_move: state.search_result.map(|res| res.best_move),
    };

    f.render_widget(info_view, layout_chunks.info);

    // Set the cursor depending on input mode
    if state.input_mode == InputMode::Insert {
        f.set_cursor(
            layout_chunks.input.x + state.input.visual_cursor() as u16 + 1,
            layout_chunks.input.y + 1
        );
    }
}

struct LayoutChunks {
    board: Rect,
    input: Rect,
    info: Rect,
    error: Rect,
}

fn create_layout(container: Rect) -> LayoutChunks {
    let app_width = 120;
    let app_height = 40;

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
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(39), Constraint::Min(1)])
        .split(centered_rect);

    let error_panel = sections[1];

    let sections = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(80), Constraint::Min(40)])
        .split(sections[0]);

    let info_panel = sections[1];

    let left_panel = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(app_height - 4), 
            Constraint::Min(3), 
        ])
        .split(sections[0]);

    let board_panel = left_panel[0];
    let input_panel = left_panel[1];

    LayoutChunks {
        board: board_panel,
        input: input_panel,
        info: info_panel,
        error: error_panel,
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

fn handle_event(state: &State) -> anyhow::Result<Option<Message>> {
    let message = if event::poll(Duration::from_millis(16))? {
        if let event::Event::Key(key) = event::read()? {
            match state.input_mode {
                InputMode::Normal => match key.code {
                    KeyCode::Char('h') => Message::GoBack,
                    KeyCode::Char('H') => Message::GoBackToStart,
                    KeyCode::Char('l') => Message::GoForward,
                    KeyCode::Char('L') => Message::GoForwardToEnd,
                    KeyCode::Char('i') => Message::InsertMode,
                    KeyCode::Char('q') => Message::Quit,
                    _ => return Ok(None),
                },

                InputMode::Insert => match key.code {
                    KeyCode::Esc => Message::NormalMode,
                    KeyCode::Enter => Message::Submit,

                    _ => Message::Input(key)
                }
            }
        } else {
            return Ok(None);
        }
    } else {
        return Ok(None);
    };

    Ok(Some(message))
}

pub fn init_tui() -> anyhow::Result<()> {
    initialize_panic_handler();

    // Startup
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(std::io::stderr(), crossterm::terminal::EnterAlternateScreen)?;

    let mut terminal = Terminal::new(CrosstermBackend::new(std::io::stderr()))?;
    let mut state = State::new();

    loop {
        // Render the current view
        terminal.draw(|f| {
            view(&mut state, f);
        })?;

        // Handle events and map to a Message
        let mut current_msg = handle_event(&mut state)?;

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
