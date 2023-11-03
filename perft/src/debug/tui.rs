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
    let current_board = state.board_stack.last().unwrap();

    let move_table = DiffTable { 
        diffs: state.move_list.clone(), 
        selected: state.selected 
    };

    let board_view = BoardView {
        board: *current_board
    };

    let info_view = InfoView {
        starting_pos: state.initial_board.to_fen(),
        current_pos: current_board.to_fen(),
        search_depth: state.depth,
        current_depth: state.board_stack.len() - 1,
        total_found: state.move_list
            .iter()
            .map(|d| d.found.unwrap_or(0))
            .sum(),
        total_expected: state.move_list
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
    info: Rect
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
        .constraints([
            Constraint::Min(40),
            Constraint::Min(50)
        ])
        .split(centered_rect);

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
        let border = Block::default()
            .borders(Borders::ALL)
            .title("Moves")
            .title_alignment(Alignment::Left)
            .border_style(Style::new().dark_gray())
            .title_style(Style::new().white())
            .padding(Padding::new(3,3,2,2));

        let mut table_state = TableState::default().with_selected(Some(self.selected));
        let rows = self.diffs
            .iter()
            .map(|diff| diff
                .to_table_row()
                .style(if diff.found == diff.expected { 
                    Style::default().dark_gray() 
                } else {
                    Style::default().red() 
                })
            );

        let table = Table::new(rows)
            .header(Row::new(vec!["Move", "Found", "Expected"]).bold().blue())
            .block(Block::new().padding(Padding::new(2,2,2,2)))
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

        StatefulWidget::render(table, area, buf, &mut table_state);
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
        let width = 10 * CELL_WIDTH;
        let height = 10 * CELL_HEIGHT;

        let rect = Layout::new()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min((area.height - height as u16)/2),
                Constraint::Min(height as u16),
                Constraint::Min((area.height - height as u16)/2),
            ])
                .split(area)[1];

        let rect = Layout::new()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min((area.width - width as u16)/2),
                Constraint::Min(width as u16),
                Constraint::Min((area.width - width as u16)/2),
            ])
                .split(rect)[1];

        let file_labels = vec!["", "a", "b", "c", "d", "e", "f", "g", "h", ""]
            .into_iter()
            .map(|label| to_padded_cell(label.to_owned()))
            .collect_vec();

        let file_labels = Row::new(file_labels)
            .height(CELL_HEIGHT as u16)
            .dark_gray();

        let mut rows: Vec<Row> = Vec::new();
        // Push top heading
        rows.push(file_labels.clone());

        let mut current_rank: Vec<Cell> = Vec::new();
        let ranks = self.board.piece_list.into_iter().chunks(8);
        let ranks = ranks.into_iter().enumerate().collect_vec().into_iter().rev();

        for (rank, squares) in ranks {
            let rank_label = to_padded_cell((rank + 1).to_string()).dark_gray();
            current_rank.push(rank_label.clone());

            for (file, square) in squares.enumerate() {
                let cell = if (file + rank) % 2 == 0 {
                    square_to_cell(square)
                } else {
                    square_to_cell(square).on_dark_gray()
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

struct InfoView {
    starting_pos: String,
    current_pos: String,
    search_depth: usize,
    current_depth: usize,
    total_found: usize,
    total_expected: usize,
}

impl Widget for InfoView {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let starting_fen = Row::new(vec![
            Cell::from("Starting position").blue(),
            Cell::from(format!("{}", self.starting_pos))
        ]);

        let current_fen = Row::new(vec![
            Cell::from("Current position").blue(),
            Cell::from(format!("{}", self.current_pos))
        ]);

        let search_depth = Row::new(vec![
            Cell::from("Search depth").blue(),
            Cell::from(format!("{}", self.search_depth))
        ]);

        let current_depth = Row::new(vec![
            Cell::from("Current depth").blue(),
            Cell::from(format!("{}", self.current_depth))
        ]);

        let total_found = Row::new(vec![
            Cell::from("Total found").blue(),
            Cell::from(format!("{}", self.total_found))
        ]);

        let total_expected = Row::new(vec![
            Cell::from("Total expected").blue(),
            Cell::from(format!("{}", self.total_expected))
        ]);

        let table = Table::new(vec![
            starting_fen,
            current_fen,
            search_depth,
            current_depth,
            total_found,
            total_expected,
        ])
            .column_spacing(1)
            .block(Block::new()
                .title("Information")
                .borders(Borders::ALL)
                .title_style(Style::new().white())
                .border_style(Style::new().dark_gray())
                .padding(Padding::new(1,1,1,1))
            )
            .widths(&[
                Constraint::Length(20), 
                Constraint::Min(50), 
            ]);

        Widget::render(table, area, buf);
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
            state.board_stack.push(new_board);

            let new_move_list = state.get_diff().unwrap();
            state.move_list = new_move_list;
            state.selected = 0;
        },

        Message::Back => {
            let current_depth = state.board_stack.len();
            if current_depth == 1 { return None; }

            state.board_stack.pop();

            let new_move_list = state.get_diff().unwrap();
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

impl State {
    fn get_diff(&mut self) -> anyhow::Result<Vec<Diff>> {
        let current_board = self.board_stack.last().unwrap();
        let depth = self.depth - (self.board_stack.len() - 1);

        let our_results = self.simbelmyne.perft(*current_board, depth)?;
        let stockfish_results = self.stockfish.perft(*current_board, depth)?;

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

        move_list.sort_by(|diff1, diff2| {
            Ord::cmp(&diff1.mv.to_string(), &diff2.mv.to_string())
        });

        Ok(move_list)
    }
}
