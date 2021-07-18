use anyhow::{Error, Result};
use serde_json::Value;
use warp::ws::Message;

//#################################################################################################
//
//                                         enum Command
//
//#################################################################################################

// A struct representing a parsed message from a client.
#[derive(Debug)]
pub enum Command {
    Welcome(usize),
    Play(String),
    Think(f64),
    Stop,
    Do,
    Undo,
    Redo,
}

// ================================ pub impl

impl Command {
    // Tries to parse a command from a warp message.
    pub fn from_msg(msg: Message) -> Result<Self> {
        let data = msg.to_str().map_err(|_| Error::msg("Incoming message is not text."))?;
        let json: Value = serde_json::from_str(data)?;

        let obj = json.as_object().ok_or(Error::msg("Json value is not an object."))?;

        let kind = obj.get("kind").ok_or(Error::msg("No attribute kind in json value."))?
            .as_str().ok_or(Error::msg("kind attribute is not a string."))?;

        Ok(match kind {
            "play" => {
                let mv = obj.get("move").ok_or(Error::msg("No attribute move in json value."))?
                    .as_str().ok_or(Error::msg("move attribute is not a string."))?.to_string();
                Self::Play(mv)
            },
            "think" => {
                let seconds = obj.get("seconds").ok_or(Error::msg("No attribute move in json value."))?
                    .as_f64().ok_or(Error::msg("seconds attribute is not a string."))?;
                Self::Think(seconds)
            },
            "stop" => Self::Stop,
            "do" => Self::Do,
            "undo" => Self::Undo,
            "redo" => Self::Redo,
            _ => return Err(Error::msg("Invalid message kind")),
        })
    }
}

//#################################################################################################
//
//                                         enum Response
//
//#################################################################################################

// An enum representing the possible responses of the game state.
#[derive(Debug)]
pub enum Response {
    Broadcast(Message),
    Send {
        dest: usize,
        msg: Message,
    }
}