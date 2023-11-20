use clap::Parser;
use cli::Command;
use uci::UciListener;

mod cli;
mod uci;
mod search;
mod evaluate;
mod square_piece_tables;
mod zobrist;
mod position;
mod transpositions;
mod move_picker;
mod time_control;
mod tests;

#[derive(Parser)]
#[command(author = "Sam Roelants", version = "0.1", about = "A simple perft tool.", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    if let Some(command) = cli.command {
        command.run()?;
    }  else {
        UciListener::new().run()?;
    }

    Ok(())
}
