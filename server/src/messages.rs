use actix::{Addr, Message};

use crate::ws_client::WsClient;

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