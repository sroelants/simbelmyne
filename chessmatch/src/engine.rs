use std::time::Duration;
use std::{collections::HashMap, process::Stdio};
use chess::board::Board;
use tokio::process::{Command, ChildStdout};
use tokio::io::{BufReader,  AsyncWriteExt, AsyncBufReadExt};
use serde::Deserialize;

use shared::uci::{UciClientMessage, UciEngineMessage,  Info, TCType};

#[derive(Debug, Clone, Deserialize)]
pub struct EngineConfig {
    pub name: String,
    command: String,
    depth: Option<usize>,
    time: Option<usize>,
    nodes: Option<usize>,
    options: HashMap<String, String>
}

#[derive(Debug)]
pub struct Engine {
    stdin: tokio::process::ChildStdin,
    stdout: tokio::io::BufReader<ChildStdout>,
    pub config: EngineConfig,
    pub tc: TCType,
    pub search_info: Info,
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
            TCType::Depth(depth)
        } else if let Some(time) = config.time {
            TCType::FixedTime(Duration::from_millis(time as u64))
        } else if let Some(nodes) = config.nodes {
            TCType::Nodes(nodes)
        } else {
            TCType::Infinite
        };

        Self {
            stdin,
            stdout: BufReader::new(stdout),
            config: config.clone(),
            tc,
            search_info: Info::default()
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
            self.stdout.read_line(&mut buf).await.unwrap();

            if let Ok(msg) = buf.parse() {
                return msg
            }
        }
    }

    pub async fn init(&mut self) {
        let _ = self.send(UciClientMessage::Uci).await;
        let _ = self.send(UciClientMessage::IsReady).await;

        for (name, value) in self.config.options.clone().into_iter() {
            self.send(UciClientMessage::SetOption(name, value)).await.unwrap();
        }

        self.send(UciClientMessage::UciNewGame).await.unwrap();
    }

    pub async fn set_pos(&mut self, board: Board) {
        self.send(UciClientMessage::Position(board)).await.unwrap();
    }

    pub async fn go(&mut self) {
        self.send(UciClientMessage::Go(self.tc)).await.unwrap();
    }

    pub fn update_info(&mut self, search_info: Info) {
        if search_info.depth.is_some() {
            self.search_info.depth = search_info.depth;
        }

        if search_info.seldepth.is_some() {
            self.search_info.seldepth = search_info.seldepth;
        }

        if search_info.time.is_some() {
            self.search_info.time = search_info.time;
        }

        if search_info.nodes.is_some() {
            self.search_info.nodes = search_info.nodes;
        }

        if search_info.score.is_some() {
            self.search_info.score = search_info.score;
        }

        if search_info.currmove.is_some() {
            self.search_info.currmove = search_info.currmove;
        }

        if search_info.currmovenumber.is_some() {
            self.search_info.currmovenumber = search_info.currmovenumber;
        }

        if search_info.hashfull.is_some() {
            self.search_info.hashfull = search_info.hashfull;
        }

        if search_info.nps.is_some() {
            self.search_info.nps = search_info.nps;
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub white: EngineConfig,
    pub black: EngineConfig,
    pub positions: Option<Vec<String>>
}
