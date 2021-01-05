use actix::{Actor, Addr, AsyncContext, Running, StreamHandler};
use actix_web_actors::ws;

use crate::messages::{Connect, Disconnect};
use crate::state::State;

// A connection to a client
pub struct WsClient {
    state_addr: Addr<State>,
    // client stuff
}

impl WsClient {
    pub fn new(state_addr: Addr<State>) -> WsClient {
        WsClient {state_addr}
    }
}

impl Actor for WsClient {
    type Context = ws::WebsocketContext<Self>;

    // When the connection is started
    fn started(&mut self, ctx: &mut Self::Context) {
        self.state_addr.do_send(Connect {
            addr: ctx.address(),
        });
    }

    // When the connection is stopped
    fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
        self.state_addr.do_send(Disconnect {
            addr: ctx.address(),
        });

        Running::Stop
    }
}

// Upon receiving a message from a client through the websocket
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsClient {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {        
        match msg {
            Ok(ws::Message::Text(text)) => {
                ctx.text("Hello from server");
            },
            _ => ctx.close(None),
        }
    }
}