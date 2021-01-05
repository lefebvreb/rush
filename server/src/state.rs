use std::collections::HashMap;

use actix::{Actor, Addr, Context, Handler};
use chess::{Color, FullGame};

use crate::ws_client::WsClient;
use crate::messages::{Connect, Disconnect};

// The role of a client
enum Role {
    Playing(Color),
    Spectating,
}

// The global state of the website
#[derive(Default)]
pub struct State {
    connections: HashMap<Addr<WsClient>, Role>,
    game: FullGame,
}

impl Actor for State {
    type Context = Context<Self>;
}

// Upon a new connection
impl Handler<Connect> for State {
    type Result = ();

    fn handle(&mut self, msg: Connect, ctx: &mut Self::Context) -> Self::Result {
        self.connections.insert(msg.addr, Role::Spectating);
    }
}

// Upon disconnection
impl Handler<Disconnect> for State {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, ctx: &mut Self::Context) -> Self::Result {
        self.connections.remove(&msg.addr);
    }
}