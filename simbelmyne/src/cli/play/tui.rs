use std::{time::Duration, sync::{Mutex, Arc}, collections::VecDeque};

use chess::square::Square;
use chess::piece::Color;
use chess::board::Board;
use chess::movegen::moves::Move;
use crossterm::event::{KeyCode, self, KeyEvent};
use ratatui::Frame;
use ratatui::Terminal;
use ratatui::prelude::CrosstermBackend;
use ratatui::prelude::Rect;
use ratatui::prelude::Direction;
use ratatui::prelude::Constraint;
use ratatui::prelude::Layout;
use ratatui::widgets::Paragraph;
use ratatui::style::Style;
use ratatui::style;
use tui_input::{self, backend::crossterm::EventHandler};

use crate::{search::{Search, DEFAULT_OPTS}, transpositions::TTable, time_control::TimeControl};
use crate::position::Position;

use shared::components::{board_view::BoardView, centered};
use super::{input_view::InputView, info_view::InfoView};

pub struct State {
    us: Color,
    play_state: PlayState,

    tt_occupancy: usize,
    tt_inserts: usize,
    tt_overwrites: usize,

    board_history: Vec<Board>,
    cursor: usize,

    error: Option<String>,

    search_depth: usize,
    search:  Option<Search>,

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
    fn new(fen: String, depth: usize) -> State {
        let board: Board = fen.parse().unwrap();

        State {
            us: board.current,
            search_depth: depth,
            play_state: PlayState::Idle,
            board_history: vec![board],
            cursor: 0,
            error: None,
            search: None,
            input: tui_input::Input::default(),
            input_mode: InputMode::Insert,
            should_quit: false,

            tt_occupancy: 0,
            tt_inserts: 0,
            tt_overwrites: 0,
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
    SearchOpponentMove,
    ReturnSearch(Search, usize, usize, usize),
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

            Some(Message::SearchOpponentMove)
        },

        Message::ReturnSearch(search, occupancy, inserts, overwrites) => {
            state.search = Some(search);
            state.tt_occupancy = occupancy;
            state.tt_inserts = inserts;
            state.tt_overwrites = overwrites;

            let new_board = state.current_board().play_move(search.pv.pv_move());
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
            if current_board.get_at(sq).is_some() {
                state.current_board()
                    .legal_moves()
                    .iter()
                    .filter(|mv| mv.src() == sq)
                    .map(|mv| mv.tgt())
                    .collect()
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
          .style(Style::default().fg(style::Color::Red));

        f.render_widget(error, layout_chunks.error);
    }


    let best_move = state.search.map_or(Move::NULL, |search| search.pv.pv_move());
    let score = state.search.map_or(0, |search| search.score);
    let nodes_visited = state.search.map_or(0, |search| search.nodes_visited);
    let duration = state.search.map_or(Duration::default(), |search| search.duration);

    let info_view = InfoView {
        depth: state.search_depth,
        duration,
        nodes_visited,
        score,
        best_move,
        tt_occupancy: state.tt_occupancy,
        tt_inserts: state.tt_inserts,
        tt_overwrites: state.tt_overwrites,
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

    let centered_rect = centered(container, app_width, app_height);

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

fn handle_event(state: &State, queue: Arc<Mutex<VecDeque<Message>>>) -> anyhow::Result<()> {

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
                    _ => return Ok(()),
                },

                InputMode::Insert => match key.code {
                    KeyCode::Esc => Message::NormalMode,
                    KeyCode::Enter => Message::Submit,

                    _ => Message::Input(key)
                }
            }
        } else {
            return Ok(());
        }
    } else {
        return Ok(());
    };

    queue.lock().unwrap().push_back(message);

    Ok(())
}

pub fn init_tui(fen: String, depth: usize) -> anyhow::Result<()> {
    initialize_panic_handler();

    // Startup
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(std::io::stderr(), crossterm::terminal::EnterAlternateScreen)?;

    let mut terminal = Terminal::new(CrosstermBackend::new(std::io::stderr()))?;
    let mut state = State::new(fen, depth);
    let message_queue: Arc<Mutex<VecDeque<Message>>> = Arc::new(Mutex::new(VecDeque::new()));
    let tt: Arc<Mutex<TTable>> = Arc::new(Mutex::new(TTable::with_capacity(64)));

    loop {
        // Render the current view
        terminal.draw(|f| {
            view(&mut state, f);
        })?;

        // Handle events and map to a Message
        handle_event(&mut state, message_queue.clone())?;

        while message_queue.lock().unwrap().len() > 0 {
            let current_msg = message_queue.lock().unwrap().pop_front();

            // Handle searches by pushing the work onto a separate thread
            if let Some(Message::SearchOpponentMove) = current_msg {
                let board = state.current_board().clone();
                let queue = message_queue.clone();

                // TODO: Make this a longer-lived thread, so I can have a transposition table that
                // is shared across searches.
                let thread_tt = tt.clone();
                std::thread::spawn(move || {
                    let mut tt = thread_tt.lock().unwrap();
                    let (tc, _handle) = TimeControl::fixed_depth(depth);
                    let search = Position::new(board).search(&mut tt, DEFAULT_OPTS, tc);

                    queue.lock().unwrap()
                        .push_back(Message::ReturnSearch(
                            search,
                            tt.occupancy(), 
                            tt.inserts(), 
                            tt.overwrites()
                        ));
                });
            } else {
                // Process updates as long as they return a non-None message
                let additional_msg = update(&mut state, current_msg.unwrap());
                if let Some(msg) = additional_msg {
                    message_queue.lock().unwrap().push_back(msg);
                }
            };
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
