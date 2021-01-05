use actix::{Actor, Addr, AsyncContext, Handler, Running, StreamHandler};
use actix_web_actors::ws;

use crate::messages::{ClientCommand, Connect, Disconnect, ServerCommand};
use crate::state::State;

// A connection to a client
pub struct WsClient {
    state: Addr<State>,
}

impl WsClient {
    // Create a new WsClient struct
    pub fn new(state: Addr<State>) -> WsClient {
        WsClient {state}
    }
}

impl Actor for WsClient {
    type Context = ws::WebsocketContext<Self>;

    // When the connection is started
    fn started(&mut self, ctx: &mut Self::Context) {
        self.state.do_send(Connect {
            addr: ctx.address(),
        });
    }

    // When the connection is stopped
    fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
        self.state.do_send(Disconnect {
            addr: ctx.address(),
        });

        Running::Stop
    }
}

// Upon receiving a message from a client through the websocket
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsClient {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {        
        match msg {
            Ok(ws::Message::Text(s)) => {
                let mut split = s.split(" ");

                match split.next().unwrap_or("") {
                    "legals" => self.state.do_send(ClientCommand::Legals {
                        addr: ctx.address(),
                    }),
                    "move" => self.state.do_send(ClientCommand::Move {
                        addr: ctx.address(),
                        s: split.next().unwrap_or("").to_string(),
                    }),
                    "play" => self.state.do_send(ClientCommand::Play {
                        addr: ctx.address(),
                    }),
                    err => eprintln!("Erroneous message \"{}\"", err),
                }
            },
            _ => ctx.close(None),
        }
    }
}

impl Handler<ServerCommand> for WsClient {
    type Result = ();

    // Upon receiving a command from the server: format it and send it via the websockets
    fn handle(&mut self, msg: ServerCommand, ctx: &mut Self::Context) -> Self::Result {
        match msg {
            ServerCommand::Role(s)   => ctx.text(format!("role {}", s)),
            ServerCommand::Legals(s) => ctx.text(format!("legals {}", s)),
            ServerCommand::Fen(s)    => ctx.text(format!("fen {}", s)),
        }
    }
}