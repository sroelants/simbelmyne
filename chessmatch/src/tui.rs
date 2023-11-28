use std::path::Path;

use chess::{piece::Color, board::Board, movegen::moves::Move};
use crossterm::event::{EventStream, self, Event, KeyCode, KeyEvent};
use tokio::stream::*;
use ratatui::{Terminal, prelude::CrosstermBackend, Frame};
use tokio::select;
use tokio_stream::StreamExt;

use crate::{engine::{Config, Engine}, uci::{UciEngineMessage, Info}};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum GameResult {
    Win(Color),
    Draw
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineInfo {
    name: String,
    elo: Option<u32>,
    depth: Option<u8>,
    nodes: Option<u32>,
    score: Option<i32>,
    hashfull: Option<u32>,
    nps: Option<u32>,
}

impl EngineInfo {
    pub fn update(&mut self, info: Info) {
        self.depth = info.depth;
        self.nodes = info.nodes;
        self.score = info.score;
        self.hashfull = info.hashfull;
        self.nps = info.nps;
    }
}

impl EngineInfo {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            elo: None,
            depth: None,
            nodes: None,
            score: None,
            hashfull: None,
            nps: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct State {
    /// The current board state
    board: Board,

    /// Information on both engines, including name, elo, and current search information
    players: [EngineInfo; Color::COUNT],

    /// The game result. None when the game is still ongoing
    result: Option<GameResult>,

    /// If playing several games in a row, hold the result history so we can 
    /// show a summary
    history: Vec<GameResult>,

    /// Pause the play loop
    paused: bool,

    /// When true, exit on the next render loop iteration
    should_quit: bool,
}

impl State {
    pub fn new(board: Board, players: [EngineInfo; Color::COUNT]) -> Self{
        Self {
            board,
            players,
            result: None,
            history: Vec::new(),
            paused: false,
            should_quit: false,
        }
    }
    
    pub fn new_game(&mut self, board: Board) {
        if let Some(result) = self.result {
            self.history.push(result)
        }

        self.board = board;
        self.result = None;
        self.paused = false;
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Message {
    PlayMove(Move),
    UpdateInfo(Info),
    TogglePause,
    NewGame(Board),
    Quit,
}

fn update(state: &mut State, message: Message) -> Option<Message> {
    use Message::*;
    match message {
        PlayMove(mv) => {
            state.board = state.board.play_move(mv);
        },

        TogglePause => {
            state.paused = !state.paused;
        },

        Quit => {
            state.should_quit = true;
        },

        NewGame(board) => {
            state.new_game(board);
        }

        UpdateInfo(info) => {
            let current = state.board.current;
            state.players[current as usize].depth = info.depth;
            state.players[current as usize].depth = info.depth;
            state.players[current as usize].depth = info.depth;
            state.players[current as usize].depth = info.depth;
            state.players[current as usize].depth = info.depth;

        }
    }

    None
}

fn view(state: &mut State, f: &mut Frame) {
    todo!()
}

fn initialize_panic_handler() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        crossterm::execute!(std::io::stderr(), crossterm::terminal::LeaveAlternateScreen).unwrap();
        crossterm::terminal::disable_raw_mode().unwrap();
        original_hook(panic_info);
    }));
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

    let mut board = Board::new();
    let mut next_player = if board.current.is_white() { &mut white } else { &mut black };

    let players = [
        EngineInfo::new(&config.white.name), 
        EngineInfo::new(&config.black.name)
    ];

    let mut state = State::new(board, players);

    let mut event_stream = EventStream::new();

    // Start loop
    loop {
        let mut msg = None;

        // Listen for engine updates or user events, whichever comes in first
        select! {
            engine_msg = next_player.read_message() => msg = Some(engine_msg),
            event_msg = handle_event(&mut event_stream) => msg = Some(event_msg)
        }

        if let Some(msg) = msg {
            update(&mut state, msg);
        }

        // Render the current view
        terminal.draw(|f| {
            view(&mut state, f);
        })?;

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
