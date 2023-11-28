use std::time::Duration;
use std::{collections::HashMap, process::Stdio};
use chess::board::Board;
use tokio::process::{Command, ChildStdout};
use tokio::io::{BufReader,  AsyncWriteExt, AsyncBufReadExt};
use serde::Deserialize;

use crate::uci::{UciClientMessage, UciEngineMessage, TimeControl};

#[derive(Debug, Clone, Deserialize)]
pub struct EngineConfig {
    pub name: String,
    command: String,
    depth: Option<usize>,
    time: Option<usize>,
    nodes: Option<usize>,
    options: HashMap<String, String>
}

pub struct Engine {
    process: tokio::process::Child,
    stdin: tokio::process::ChildStdin,
    stdout: tokio::io::BufReader<ChildStdout>,
    pub config: EngineConfig,
    tc: TimeControl,
}

impl Engine {
    pub fn new(
        config: &EngineConfig
    ) -> Self {
        let mut command = Command::new(&config.command);

        let mut process = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        let stdin = process.stdin.take().unwrap();
        let stdout = process.stdout.take().unwrap();

        let tc = if let Some(depth) = config.depth {
            TimeControl::Depth(depth)
        } else if let Some(time) = config.time {
            TimeControl::Time(Duration::from_millis(time as u64))
        } else if let Some(nodes) = config.nodes {
            TimeControl::Nodes(nodes)
        } else {
            TimeControl::Infinite
        };

        Self {
            process,
            stdin,
            stdout: BufReader::new(stdout),
            config: config.clone(),
            tc
        }
    }

    // Send a UCI message to the engine over stdin, ignoring any errors
    pub async fn send(&mut self, msg: UciClientMessage) -> anyhow::Result<()>{
        let _ = self.stdin.write(format!("{msg}\n").as_bytes()).await;
        let _ = self.stdin.flush().await;

        Ok(())
    }

    /// Keep reading UCI messages from the engine stdout, discarding any that we
    /// fail to parse, and returning the first message that parsed correctly
    pub async fn read(&mut self) -> UciEngineMessage {
        loop {
            let mut buf = String::new();
            let _ = self.stdout.read_line(&mut buf).await;

            if let Ok(msg) = buf.parse() {
                return msg
            }
        }
    }

    pub async fn init(&mut self) {
        let _ = self.send(UciClientMessage::Uci).await;
        let _ = self.send(UciClientMessage::IsReady).await;

        for (name, value) in self.config.options.clone().into_iter() {
            let _ = self.send(UciClientMessage::SetOption(name, value)).await;
        }

        let _ = self.send(UciClientMessage::UciNewGame).await;
    }

    pub async fn set_pos(&mut self, board: Board) {
        let _ = self.send(UciClientMessage::Position(board.to_fen())).await;
    }

    pub async fn go(&mut self) {
        let _ = self.send(UciClientMessage::Go(self.tc)).await;
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub white: EngineConfig,
    pub black: EngineConfig
}
