use clap::{Parser, Subcommand};
use play::run_play;
use uci::UciListener;
mod play;
mod uci;


#[derive(Parser)]
#[command(author = "Sam Roelants", version = "0.1", about = "A simple perft tool.", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    Play {
        ///Start from a FEN string
        #[arg(short, long, default_value = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")]
        fen: String,
    }
}

impl Command {
    fn run(&self) -> anyhow::Result<()> {
        match self {
            Command::Play { fen } => run_play(fen)?,
        };

        Ok(())
    }
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
