use std::{collections::HashMap, process::Stdio, io::Write};
use std::io::{BufReader, BufRead};
use std::process::{Command, ChildStdin};
use chess::piece::Color;
use serde::Deserialize;

use crate::uci::{UciClientMessage, UciEngineMessage, TimeControl};

#[derive(Debug, Clone, Deserialize)]
pub struct EngineConfig {
    name: String,
    command: String,
    depth: Option<usize>,
    time: Option<usize>,
    nodes: Option<usize>,
    options: HashMap<String, String>
}

pub struct Engine {
    process: std::process::Child,
    stdin: ChildStdin,
    config: EngineConfig,
    tc: TimeControl,
}

impl Engine {
    pub fn new(
        side: Color, 
        sender: crossbeam::channel::Sender<LabeledMessage>, 
        config: &EngineConfig
    ) -> Self {
        let mut command = Command::new(&config.command);

        let mut process = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        let mut stdin = process.stdin.take().unwrap();
        let stdout = process.stdout.take().unwrap();

        std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines() {
                let line = line.unwrap();
                println!("{line}");

                if let Ok(msg) = line.parse() {
                    let _ = sender.send((side, msg));
                }
            }
        });

        // Set up time control
        let tc =  if let Some(n) = config.depth {
            TimeControl::Depth(n)
        } else if let Some(t) = config.time {
            TimeControl::Time(t)
        } else if let Some(n) = config.nodes {
            TimeControl::Nodes(n)
        } else {
            TimeControl::Infinite
        };

        // Set up engine
        let _ = writeln!(stdin, "{}", UciClientMessage::Uci);
        let _ = stdin.flush();
        let _ = writeln!(stdin, "{}", UciClientMessage::IsReady);
        let _ = stdin.flush();
        for (name, value) in config.options.clone().into_iter() {
            let _ = writeln!(stdin, "{}", UciClientMessage::SetOption(name, value));
        }

        let _ = stdin.flush();
        let _ = writeln!(stdin, "{}", UciClientMessage::UciNewGame);

        Self {
            process,
            stdin,
            config: config.clone(),
            tc
        }
    }

    pub fn send(&mut self, msg: UciClientMessage) -> anyhow::Result<()>{
        write!(self.stdin, "{msg}")?;
        self.stdin.flush()?;

        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub white: EngineConfig,
    pub black: EngineConfig
}

pub type LabeledMessage = (Color, UciEngineMessage);
