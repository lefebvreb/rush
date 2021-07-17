use anyhow::{Error, Result};
use serde::Deserialize;
use warp::ws::Message;

//#################################################################################################
//
//                                         struct ClientMessage
//
//#################################################################################################

// A struct representing a parsed message from a client.
#[derive(Debug)]
pub enum ClientMessage {
    All,
    Play(String),
    Think(f32),
    Stop,
    Do,
    Undo,
    Redo,
}

// ================================ pub impl

impl ClientMessage {
    // Tries to parse a message from a client. Since we don't really care about
    // any error that might occur, the error type is simple the unit type.
    // A return value of Err(()) means that either the message was not text,
    // it wasn't json or that it doesn't follow the correct format.
    pub fn from_msg(msg: Message) -> Result<Self> {
        // The struct an incoming json message is supposed to conform to.
        #[derive(Deserialize, Debug)]
        struct ExpectedMessage {
            kind: String,
            #[serde(default)]
            seconds: f32,
            #[serde(default)]
            mv: String,
        }

        // Extract text data from the message.
        let data = msg.to_str().map_err(|err| Error::msg("Incoming message is not text."))?;

        // Parse the json message.
        let msg: ExpectedMessage = serde_json::from_str(data)?;

        // Converts to the correct enum variant.
        Ok(match msg.kind.as_str() {
            "all"   => Self::All,
            "play"  => Self::Play(msg.mv),
            "think" => Self::Think(msg.seconds),
            "stop" => Self::Stop,
            "do"    => Self::Do,
            "undo"  => Self::Undo,
            "redo"  => Self::Redo,
            _ => return Err(Error::msg("Invalid incoming message kind.")),
        })
    }
}