use std::collections::HashMap;

use actix::{Actor, StreamHandler};
use actix_web_actors::ws;

use chess::*;

enum Role {
    Playing(Color),
    Spectating,
}

#[derive(Default)]
pub struct OnlineGame {
    game: Box<FullGame>,
    clients: HashMap<String, Role>,
}

impl Actor for OnlineGame {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for OnlineGame {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        println!("{:?}", msg);
        
        match msg {
            Ok(ws::Message::Text(text)) => {
                println!("Got a message: {}", text);
                ctx.text("Hello from server");
            },
            _ => ctx.close(None),
        }
    }
}