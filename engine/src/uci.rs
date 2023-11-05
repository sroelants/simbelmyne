use std::collections::BTreeMap;

use chess::movegen::moves::Move;

use crate::Engine;

/// Messages that the Client can send to the Server
pub enum ClientMessage {
    Uci,
    Debug(bool),
    IsReady,
    SetOption(BTreeMap<String, Vec<String>>),
    UciNewGame,
    Position(String), // TODO: Accept both FEN and startpos+moves
    Go, // Don't bother with any go options for now
    Stop,
    PonderHit,
    Quit,
    Poll, //NOTE Not par of UCI, just a way for me to poll for additional infarmation
}

/// Messages that the Client can send to the Server
pub enum ServerMessage {
    Id { name: &'static str, author: &'static str },
    UciOk,
    ReadyOk,
    BestMove(Move),
    Info(BTreeMap<String, String>),
}

pub trait Uci {
    fn receive(&mut self, msg: ClientMessage) -> anyhow::Result<Vec<ServerMessage>>;
}

impl Uci for Engine {
    fn receive(&mut self, msg: ClientMessage) -> anyhow::Result<Vec<ServerMessage>> {
        match msg {
            ClientMessage::Uci => {
                Ok(vec![
                    ServerMessage::Id { name: "Simbelmyne", author: "Sam Roelants" },
                    ServerMessage::UciOk
                ])
            }

            ClientMessage::Debug(debug) => {
                self.debug = debug;
                Ok(vec![])
            }

            ClientMessage::IsReady => Ok(vec![ServerMessage::ReadyOk]),

            ClientMessage::SetOption(_) => Ok(vec![]),

            ClientMessage::UciNewGame => Ok(vec![]),

            ClientMessage::Position(fen) => {
                self.board = fen.parse()?;
                Ok(vec![])
            },

            ClientMessage::Go => {
                let mv = self.next_move();
                Ok(vec![ServerMessage::BestMove(mv)])
            },
            
            ClientMessage::Stop => Ok(vec![]),

            ClientMessage::Quit => Ok(vec![]),

            ClientMessage::Poll => Ok(vec![]), // Send over information

            _ => unreachable!()

        }
    }
}
