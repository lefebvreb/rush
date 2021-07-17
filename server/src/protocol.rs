use anyhow::{Error, Result};
use serde::{Serialize, Deserialize};
use warp::ws::Message;

//#################################################################################################
//
//                                         struct ClientMessage
//
//#################################################################################################

// A struct representing a parsed message from a client.
#[derive(Debug)]
pub enum ClientMessage {
    Play(String),
    Think(f32),
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
    pub fn from_msg(msg: Message) -> Result<ClientMessage> {
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
            "play"  => ClientMessage::Play(msg.mv),
            "think" => ClientMessage::Think(msg.seconds),
            "do"    => ClientMessage::Do,
            "undo"  => ClientMessage::Undo,
            "redo"  => ClientMessage::Redo,
            _ => return Err(Error::msg("Invalid incoming message kind.")),
        })
    }
}

//#################################################################################################
//
//                                         struct ServerMessage
//
//#################################################################################################

// A struct that represents a server message to be sent by a websocket.
#[allow(non_snake_case)]
#[derive(Serialize, Debug)]
pub struct ServerMessage {
    pub fen: String,
    pub draw: bool,
    pub lastMove: String,
    
    pub thinking: bool,
    pub engineMove: String,
    pub engineDepth: u8,
}

// ================================ pub impl

impl ServerMessage {
    // Consumes the ServerMessage struct and converts it into a warp message.
    pub fn to_warp_msg(&self) -> Message {
        let json = serde_json::to_string(&self).expect("Could not convert server message to json.");
        Message::text(json)
    }
}