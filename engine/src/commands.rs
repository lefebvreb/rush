use actix::{Actor, Context, Handler, Message};

use crate::engine::Engine;

#[derive(Message)]
#[rtype(result = "()")]
pub struct EngineMove(pub String);

#[derive(Message)]
#[rtype(result = "()")]
pub enum EngineCommand {
    AskMove,
    Move(String),
}

impl Actor for Engine {
    type Context = Context<Self>;
}

impl Handler<EngineCommand> for Engine {
    type Result = ();

    fn handle(&mut self, _: EngineCommand, _: &mut Self::Context) -> Self::Result {
        //TODO
    }
}
