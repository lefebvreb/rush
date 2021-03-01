use actix::{Addr, Message};

use crate::wsclient::WsClient;

// A message used to signify a new connection 
#[derive(Message)]
#[rtype(result = "()")]
pub struct Connect {
    pub addr: Addr<WsClient>,
}

// A message used to signify a disconnection
#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub addr: Addr<WsClient>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ClientMove {
    pub addr: Addr<WsClient>,
    pub s: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ClientRequestEngine;

#[derive(Message)]
#[rtype(result = "()")]
pub struct ClientRequestPlay {
    pub addr: Addr<WsClient>,
}

// A message used to represent a server command
#[derive(Clone, Default, Message)]
#[rtype(result = "()")]
pub struct ClientInfo(pub String);