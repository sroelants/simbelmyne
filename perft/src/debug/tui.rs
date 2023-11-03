use std::collections::HashSet;

use crossterm::event::KeyCode;
use chess::{movegen::moves::Move, board::{Board, Piece}};
use ratatui::{Frame, widgets::{Paragraph, Block, Borders, Padding, Widget, TableState, Row, Table, StatefulWidget, HighlightSpacing, Cell}, Terminal, prelude::{CrosstermBackend, Alignment, Buffer, Direction, Layout, Constraint, Rect}, style::{Stylize, Style}, text::{Line, Text, Span}};
use itertools::Itertools;

use crate::{perft::perft_divide, debug::engine::Engine};

use super::engine::{Stockfish, Simbelmyne};

const DEFAULT_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

#[derive(Debug, Clone)]
pub struct Diff {
    mv: Move,
    found: Option<usize>,
    expected: Option<usize>,
}

impl Diff {
    fn to_table_row(&self) -> Row {
        let mv = self.mv.to_string();
        let found = self.found
            .map(|found| found.to_string()).unwrap_or(String::from(""));
        let expected = self.expected
            .map(|found| found.to_string()).unwrap_or(String::from(""));

        Row::new(vec![mv, found, expected])
    }
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
        let move_list = get_perft_result(initial_board, depth);
        let stockfish = Stockfish::new().unwrap();
        let simbelmyne = Simbelmyne {};


        State {
            stockfish,
            simbelmyne,
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
    let layout = create_layout(term_rect);

    let move_table = DiffTable { 
        diffs: state.move_list.clone(), 
        selected: state.selected 
    };

    let board_view = BoardView {
        board: *state.board_stack.last().unwrap()
    };

    f.render_widget(move_table, layout.table);
    f.render_widget(board_view, layout.board);
    // f.render_widget(board_view(state), layout.board);
    f.render_widget(info_view(state), layout.info);

}

struct LayoutChunks {
    table: Rect,
    board: Rect,
    info: Rect
}

fn create_layout(container: Rect) -> LayoutChunks {
    let app_rect = Rect::new(container.x, container.y, container.width, 50);

    let sections = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(40),
            Constraint::Min(50)
        ])
        .split(app_rect);

    let left_panel = sections[0];
    let right_panel = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(70),
            Constraint::Percentage(30),
        ])
        .split(sections[1]);
    let board_panel = right_panel[0];
    let info_panel = right_panel[1];

    LayoutChunks {
        table: left_panel,
        board: board_panel,
        info: info_panel
    }
}

struct DiffTable {
    diffs: Vec<Diff>,
    selected: usize,
}

impl Widget for DiffTable {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::new()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(80), Constraint::Min(8)])
            .split(area);

        let top_panel = layout[0];
        let bottom_panel = layout[1];
        
        let border = Block::default()
            .borders(Borders::ALL)
            .title("Moves")
            .title_alignment(Alignment::Left)
            .padding(Padding::new(3,3,1,1));

        let mut table_state = TableState::default().with_selected(Some(self.selected));
        let rows = self.diffs
            .iter()
            .map(|diff| diff
                .to_table_row()
                .style(Style::default().dark_gray())
            );

        let table = Table::new(rows)
            .header(Row::new(vec!["Move", "Found", "Expected"]).bold().blue())
            .block(Block::new().padding(Padding::new(1,1,1,1)))
            .widths(&[
                Constraint::Length(5), 
                Constraint::Length(10), 
                Constraint::Length(10)
            ])
            .column_spacing(3)
            .highlight_style(Style::default().white())
            .highlight_spacing(HighlightSpacing::Always)
            .highlight_symbol("> ");

        border.render(area, buf);

        StatefulWidget::render(table, top_panel, buf, &mut table_state);

        Paragraph::new(vec![
            "Total found".dark_gray().into(),
            "Total expected".dark_gray().into()
        ])
            .block(Block::new().padding(Padding::new(2,2,2,2)))
            .render(bottom_panel, buf);
    }
}

struct BoardView {
    board: Board
}

fn square_to_cell(piece: Option<Piece>) -> Cell<'static> {
    match piece {
        Some(piece) => {
            to_padded_cell(piece.to_string())
        },
        None => {
            to_padded_cell(String::from(""))
        }
    }
}

const CELL_WIDTH: usize = 5;
const CELL_HEIGHT: usize = 3;

fn to_padded_cell(val: String) -> Cell<'static> {
    let lines = vec![
        vec![Line::from(""); CELL_HEIGHT/2],
        vec![Line::from(format!("{:^CELL_WIDTH$}", val))],
        vec![Line::from(""); CELL_HEIGHT/2]
    ].concat();

    Cell::from(lines)
}

impl Widget for BoardView {
    fn render(self, area: Rect, buf: &mut Buffer) {

        let file_labels = vec!["", "a", "b", "c", "d", "e", "f", "g", "h", ""]
            .into_iter()
            .map(|label| to_padded_cell(label.to_owned()))
            .collect_vec();

        let file_labels = Row::new(file_labels)
            .height(CELL_HEIGHT as u16)
            .blue();

        let mut rows: Vec<Row> = Vec::new();
        // Push top heading
        rows.push(file_labels.clone());

        let mut current_rank: Vec<Cell> = Vec::new();
        let ranks = self.board.piece_list.into_iter().chunks(8);
        let ranks = ranks.into_iter().enumerate().collect_vec().into_iter().rev();

        for (rank, squares) in ranks {
            current_rank.push(to_padded_cell((rank + 1).to_string()).blue());

            for (file, square) in squares.enumerate() {
                let cell = if (file + rank) % 2 == 0 {
                    square_to_cell(square)
                } else {
                    square_to_cell(square).on_dark_gray()
                };

                current_rank.push(cell);
            }

            current_rank.push(to_padded_cell((rank + 1).to_string()).blue());

            rows.push(Row::new(current_rank).height(CELL_HEIGHT as u16));
            current_rank = Vec::new();
        }

        // Push bottom heading
        rows.push(file_labels);

        let table = Table::new(rows)
            .widths(&[Constraint::Length(5); 10])
            .block(Block::new()
                .title("Board")
                .borders(Borders::ALL)
            );

        
        Widget::render(table, area, buf);
    }
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

fn update(state: &mut State, message: Message) -> Option<Message> {
    match message {
        Message::Up => {
            if 0 < state.selected {
                state.selected -= 1
            }
        },

        Message::Down => {
            if state.selected < state.move_list.len() - 1 { 
                state.selected += 1
            }
        },

        Message::Quit => state.should_quit = true,

        Message::Select => {
            let current_depth = state.board_stack.len();
            if current_depth == state.depth { return None; }

            let current_board = state.board_stack.last().unwrap();
            let selected_move = state.move_list[state.selected].mv;

            let new_board = current_board.play_move(selected_move);
            let new_move_list = get_diff(state).unwrap();
            state.move_list = new_move_list;
            state.board_stack.push(new_board);
            state.selected = 0;
        },

        Message::Back => {
            let current_depth = state.board_stack.len();
            if current_depth == 1 { return None; }

            state.board_stack.pop();

            let new_move_list = get_diff(state).unwrap();
            state.move_list = new_move_list;
            state.selected = 0;

        },
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

fn get_perft_result(board: Board, depth: usize) -> Vec<Diff> {
    let results = perft_divide::<true>(board, depth);

    results
        .into_iter()
        .map(|(mv, nodes)| Diff { mv, found: Some(nodes), expected: Some(nodes) })
        .collect()
}

fn get_diff(state: &mut State) -> anyhow::Result<Vec<Diff>> {
    let current_board = state.board_stack.last().unwrap();
    let depth = state.depth - state.board_stack.len();

    let our_results = state.simbelmyne.perft(*current_board, depth)?;
    let stockfish_results = state.stockfish.perft(*current_board, depth)?;

    let moves = our_results.keys()
        .chain(stockfish_results.keys())
        .collect::<HashSet<_>>();

    let mut move_list = Vec::new();
    for mv in moves {
        move_list.push(Diff { 
            mv: mv.parse().unwrap(), 
            found: our_results.get(mv).copied(), 
            expected: stockfish_results.get(mv).copied() 
        });
    }

    Ok(move_list)
}
