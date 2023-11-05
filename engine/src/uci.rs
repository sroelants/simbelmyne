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
    // Combine the Id and UciOK commands into one message, so we can keep a 1-1 request<->response
    // model
    UciOk { name: &'static str, author: &'static str },
    ReadyOk,
    BestMove(Move),
    Info(BTreeMap<String, String>),
    Quiet, // In case the server doesn't want to send anything back
}

pub trait Uci {
    fn receive(&mut self, msg: ClientMessage) -> anyhow::Result<ServerMessage>;
}

impl Uci for Engine {
    fn receive(&mut self, msg: ClientMessage) -> anyhow::Result<ServerMessage> {
        match msg {
            ClientMessage::Uci => {
                    Ok(ServerMessage::UciOk { 
                    name: "Simbelmyne", 
                    author: "Sam Roelants" 
                })
            }

            ClientMessage::Debug(debug) => {
                self.debug = debug;
                Ok(ServerMessage::Quiet)
            }

            ClientMessage::IsReady => Ok(ServerMessage::ReadyOk),

            ClientMessage::SetOption(_) => Ok(ServerMessage::Quiet),

            ClientMessage::UciNewGame => Ok(ServerMessage::Quiet),

            ClientMessage::Position(fen) => {
                self.board = fen.parse()?;
                Ok(ServerMessage::Quiet)
            },

            ClientMessage::Go => {
                let mv = self.next_move();
                Ok(ServerMessage::BestMove(mv))
            },

            ClientMessage::PonderHit => Ok(ServerMessage::Quiet),
            
            ClientMessage::Stop => Ok(ServerMessage::Quiet),

            ClientMessage::Quit => Ok(ServerMessage::Quiet),

            ClientMessage::Poll => Ok(ServerMessage::Quiet), // Send over information
        }
    }
}
