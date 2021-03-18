use actix::{Actor, Addr, AsyncContext, Handler, Running, StreamHandler};
use actix_web_actors::ws;

use crate::messages::{ClientInfo, ClientMove, ClientRequestEngine, ClientRequestPlay, Connect, Disconnect};
use crate::state::State;

//#################################################################################################
//
//                                      struct WsClient
//
//#################################################################################################


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

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsClient {
    // Upon receiving a message from a client through the websocket
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {        
        match msg {
            Ok(ws::Message::Text(s)) => {
                let mut split = s.split(" ");

                match split.next().unwrap_or("") {
                    "move" => self.state.do_send(ClientMove {
                        addr: ctx.address(),
                        mv: split.next().unwrap_or("").to_string(),
                    }),
                    "play" => self.state.do_send(ClientRequestPlay {
                        addr: ctx.address(),
                    }),
                    "invite" => self.state.do_send(ClientRequestEngine),
                    err => eprintln!("Erroneous message \"{}\"", err),
                }
            },
            _ => ctx.close(None),
        }
    }
}

impl Handler<ClientInfo> for WsClient {
    type Result = ();

    // Upon receiving a command from the server: format it and send it via the websockets
    fn handle(&mut self, msg: ClientInfo, ctx: &mut Self::Context) {
        ctx.text(msg.0)
    }
}