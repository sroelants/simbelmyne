use clap::Parser;
use cli::Command;
use uci::SearchController;

mod cli;
mod spsa;
mod uci;

#[derive(Parser)]
#[command(author = "Sam Roelants", version = "0.1", about = "A simple perft tool.", long_about = None)]
struct Cli {
  /// Load the engine with a praticular board position
  #[arg(
    short,
    long,
    value_name = "FEN",
    default_value = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
  )]
  fen: String,

  #[command(subcommand)]
  command: Option<Command>,
}

fn main() -> anyhow::Result<()> {
  let cli = Cli::parse();

  if let Some(command) = cli.command {
    command.run()?;
  } else {
    let board = cli.fen.parse().unwrap();
    SearchController::new(board).run()?;
  }

  Ok(())
}
