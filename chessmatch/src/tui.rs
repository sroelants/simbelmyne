use std::time::Duration;

use chess::{piece::Color, board::Board, movegen::moves::Move};
use crossterm::event::{EventStream, Event, KeyCode};
use ratatui::{Terminal, prelude::CrosstermBackend};
use tokio::select;
use tokio_stream::StreamExt;
use shared::uci::UciEngineMessage;
use shared::uci::Info;

use crate::{engine::{Config, Engine}, components::view};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum GameResult {
    Win(Color),
    Draw
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameMode {
    Suite(Vec<Board>),
    Infinite
}

#[derive(Debug)]
pub struct State {
    /// The mode we're running the gauntlet in
    pub mode: GameMode, 

    /// Count of how many games we've played so far
    pub game_count: usize,

    /// The current board state
    pub board: Board,

    /// The current game's starting position
    pub initial_board: Board,

    /// The game result. None when the game is still ongoing
    pub result: Option<GameResult>,

    /// If playing several games in a row, hold the result history so we can 
    /// show a summary
    pub history: Vec<GameResult>,

    /// Pause the play loop
    pub paused: bool,

    /// When true, exit on the next render loop iteration
    pub should_quit: bool,

    pub engines: [Engine; Color::COUNT],
}

impl State {
    pub fn new(mode: GameMode, white: Engine, black: Engine) -> Self{
        let board = match &mode {
            GameMode::Infinite => Board::new(),
            GameMode::Suite(boards) => boards[0],
        };

        Self {
            mode,
            game_count: 0,
            board,
            initial_board: board,
            result: None,
            history: Vec::new(),
            paused: false,
            should_quit: false,
            engines: [white, black]
        }
    }
    
    pub fn next_player(&mut self) -> &mut Engine {
        &mut self.engines[self.board.current as usize]
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Message {
    StartSearch,
    PlayMove(Move),
    UpdateInfo(Info),
    TogglePause,
    GameFinished(GameResult),
    NewGame,
    Quit,
}

async fn update(state: &mut State, message: Message) -> Option<Message> {
    use Message::*;

    match message {
        StartSearch => {
            let board = state.board;
            state.next_player().set_pos(board).await;
            state.next_player().go().await;
            None
        },

        PlayMove(mv) => {
            state.board = state.board.play_move(mv);

            if state.board.checkmate() {
                return Some(GameFinished(GameResult::Win(state.board.current.opp())));
            }

            if state.board.is_draw() {
                return Some(GameFinished(GameResult::Draw));
            }

            Some(StartSearch)
        },

        GameFinished(result) => {
            state.result = Some(result);
            state.history.push(result);

            match &state.mode {
                GameMode::Infinite => Some(NewGame),

                GameMode::Suite(boards) => {
                    if state.game_count + 1 < boards.len() {
                        Some(NewGame)
                    } else {
                        None
                    }
                }
            }
        }

        TogglePause => {
            state.paused = !state.paused;
            None
        },

        Quit => {
            state.should_quit = true;
            None
        },

        NewGame => {
            state.result = None;
            state.paused = false;
            state.game_count += 1;


            state.board = match &state.mode {
                GameMode::Infinite => Board::new(),

                // we checked that we're in bounds in GameFinished branch
                GameMode::Suite(boards) => boards[state.game_count],
            };

            Some(StartSearch)
        },

        UpdateInfo(info) => {
            state.engines[state.board.current as usize].update_info(info);
            None
        }
    }
}

async fn handle_event(stream: &mut EventStream) -> Message {
    loop {
        if let Some(Ok(Event::Key(key))) = stream.next().await {
            match key.code {
                KeyCode::Char('q') => return Message::Quit,
                KeyCode::Char(' ') => return Message::TogglePause,
                _ => {},
            }
        }
    }
}

impl Engine {
    async fn read_message(&mut self) -> Message {
        loop {
            let uci_msg = self.read().await;

            if let UciEngineMessage::BestMove(mv) = uci_msg {
                return Message::PlayMove(mv);
            } else if let UciEngineMessage::Info(info) = uci_msg {
                return Message::UpdateInfo(info);
            }
        }
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

pub async fn init_tui(config: Config) -> anyhow::Result<()> {
    initialize_panic_handler();

    // Startup
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(std::io::stderr(), crossterm::terminal::EnterAlternateScreen)?;

    let mut terminal = Terminal::new(CrosstermBackend::new(std::io::stderr()))?;

    // Set up state
    let mut white = Engine::new(&config.white);
    let mut black = Engine::new(&config.black);
    white.init().await;
    black.init().await;

    let mode = match config.positions {
        Some(fens) => {
            let boards: Vec<Board> = fens.iter()
                .map(|fen| fen.parse().unwrap())
                .collect();
            GameMode::Suite(boards)
        },
        None => GameMode::Infinite
    };

    let mut state = State::new(mode, white, black);

    let mut event_stream = EventStream::new();
    let board = state.board;

    state.next_player().set_pos(board).await;
    state.next_player().go().await;


    let mut render_interval = tokio::time::interval(Duration::from_millis(10));

    // Start loop
    loop {
        let mut msg = None;

        // Listen for engine updates or user events, whichever comes in first
        // TODO: How do I make this work with 'state.paused'?
        select! {
            engine_msg = state.next_player().read_message() => msg = Some(engine_msg),
            event_msg = handle_event(&mut event_stream) => msg = Some(event_msg),
            _ = render_interval.tick() => {
                // Render the current view
                terminal.draw(|f| {
                    view(&mut state, f);
                })?;
            }
        }

        while let Some(next_message) = msg {
            msg = update(&mut state, next_message).await;
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
