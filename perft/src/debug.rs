use crossterm::event::KeyCode;
use chess::{movegen::moves::Move, board::{Board, Square}};
use ratatui::{Frame, widgets::{Paragraph, Block, Borders, block::Position, Padding, Widget, Wrap}, Terminal, prelude::{CrosstermBackend, Alignment, Buffer, Direction, Layout, Constraint, Rect}, style::Stylize, text::{Line, Text, Span}};

use crate::perft::perft_divide;
const DEFAULT_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

#[derive(Debug, Clone)]
pub struct DivideResult {
    mv: Move,
    found: usize,
    expected: usize,
}

#[derive(Debug, Clone)]
struct State {
    move_list: Vec<DivideResult>,
    selected: usize,
    depth: usize,
    initial_board: Board,
    board_stack: Vec<Board>,
    should_quit: bool,
}

impl State {
    fn new(depth: usize, fen: String) -> State {
        let initial_board = fen.parse().unwrap();
        let move_list = get_perft_result(initial_board, depth);

        State {
            move_list,
            selected: 0,
            depth,
            initial_board,
            board_stack: vec![initial_board],
            should_quit: false,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Message {
    Up,
    Down,
    Select,
    Back, 
    Quit
}

fn view(state: &mut State, f: &mut Frame) {
    let term_rect = f.size();
    let app_rect = Rect::new(term_rect.x, term_rect.y, term_rect.width, 30);

    let sections = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(40),
            Constraint::Min(50)
        ])
        .split(app_rect);

    let left_panel = sections[0];
    let right_panel = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(70),
            Constraint::Percentage(70),
        ])
        .split(sections[1]);
    let board_panel = right_panel[0];
    let info_panel = right_panel[1];


    // Split up the frames
    f.render_widget(list_view(state), left_panel);
    f.render_widget(board_view(state), board_panel);
    f.render_widget(info_view(state), info_panel);

}

fn list_view(state: &State) -> impl Widget {
    let border = Block::default()
        .borders(Borders::ALL)
        .title("Moves")
        .title_alignment(Alignment::Left)
        .padding(Padding::new(3,3,1,1));

    let entries: Vec<Line> = state.move_list
        .iter()
        .enumerate()
        .map(|(idx, DivideResult { mv, found, expected })| {
            if idx == state.selected {
                format!("{mv:10}{found:10}{expected:10}").white().bold().into()
            } else if found != expected {
                format!("{mv:10}{found:10}{expected:10}").red().into()
            } else {
                format!("{mv:10}{found:10}{expected:10}").dark_gray().into()
            }
        })
        .collect();

    let text = Text::from(vec![
        vec![format!("{:8}{:8}{:8}", "Move", "Found", "Expected").bold().into(), "".into()],
        entries.clone()
    ].concat());

    Paragraph::new(text)
        .block(border)
}

fn board_view(state: &State) -> impl Widget {
    let current_board = state.board_stack.last().unwrap();
    let border = Block::default()
        .borders(Borders::ALL)
        .title("Board")
        .title_alignment(Alignment::Left)
        .padding(Padding::new(3,3,1,1));

    let board_str = current_board.to_string();

    let board_lines: Vec<Line> = board_str
        .split('\n')
        .map(|s| s.to_owned().white().bold().into())
        .collect::<Vec<_>>();

    let board_text = Text::from(board_lines);
    
    Paragraph::new(board_text)
        .block(border)
        .alignment(Alignment::Center)
}

fn info_view(state: &State) -> impl Widget {
    let border = Block::default()
        .borders(Borders::ALL)
        .title("Info")
        .title_alignment(Alignment::Left)
        .padding(Padding::new(3,3,1,1));

    Paragraph::new(vec![
        Span::raw(format!("Starting FEN: {}", DEFAULT_FEN)).into(),
        Span::raw(format!("Current FEN: {}", DEFAULT_FEN)).into(),
        Span::raw(format!("Current depth: {}", state.board_stack.len())).into()
    ])
      .block(border)
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
                KeyCode::Char('q') => Message::Quit,
                KeyCode::Char('h') => Message::Back,
                KeyCode::Enter => Message::Select,
                KeyCode::Esc => Message::Quit,
                _ => return Ok(None)
            }
        } else {
            return Ok(None)
        }
    } else {
        return Ok(None)
    };

    Ok(Some(message))
}

fn update(state: &State, message: Message) -> (State, Option<Message>) {
    let mut new_state = state.clone();

    match message {
        Message::Up => {
            if 0 < new_state.selected {
                new_state.selected -= 1
            }
        },

        Message::Down => {
            if new_state.selected < new_state.move_list.len() - 1 { 
                new_state.selected += 1
            }
        },
        Message::Quit => new_state.should_quit = true,
        Message::Select => {
            let current_depth = state.board_stack.len();
            if current_depth == state.depth { return (new_state, None); }
            let remaining_depth = state.depth - current_depth;


            let current_board = state.board_stack.last().unwrap();
            let selected_move = state.move_list[state.selected].mv;

            let new_board = current_board.play_move(selected_move);
            let new_moves = perft_divide::<true>(new_board, remaining_depth);
            let new_move_list = new_moves
                .into_iter()
                .map(|(mv, nodes)| DivideResult { mv, found: nodes, expected: nodes })
                .collect();

            new_state.move_list = new_move_list;
            new_state.board_stack.push(new_board);
            new_state.selected = 0;
        },
        Message::Back => {
            let current_depth = state.board_stack.len();
            if current_depth == 1 { return (new_state, None); }

            let remaining_depth = state.depth - current_depth;

            new_state.board_stack.pop();
            let previous_board = new_state.board_stack.last().unwrap();

            let new_moves = perft_divide::<true>(*previous_board, remaining_depth + 1);
            let new_move_list = new_moves
                .into_iter()
                .map(|(mv, nodes)| DivideResult { mv, found: nodes, expected: nodes })
                .collect();

            new_state.move_list = new_move_list;
            new_state.selected = 0;

        },
    }

    (new_state, None)
}

pub fn run_debug(depth: usize, fen: String) -> anyhow::Result<()> {
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
    while current_msg != None {
        let (new_state, new_message) = update(&mut state, current_msg.unwrap());
        state = new_state;
        current_msg = new_message;
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

fn get_perft_result(board: Board, depth: usize) -> Vec<DivideResult> {
    let results = perft_divide::<true>(board, depth);

    results
        .into_iter()
        .map(|(mv, nodes)| DivideResult { mv, found: nodes, expected: nodes })
        .collect()
}
