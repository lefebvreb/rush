use actix::{Actor, Context, Handler, Message};

use crate::engine::Engine;

#[derive(Message)]
#[rtype(result = "()")]
pub struct EngineMove(pub String);

#[derive(Message)]
#[rtype(result = "()")]
pub struct EngineAskMove;

#[derive(Message)]
#[rtype(result = "()")]
pub struct EngineMakeMove(pub String);

impl Actor for Engine {
    type Context = Context<Self>;
}

impl Handler<EngineAskMove> for Engine {
    type Result = ();

    fn handle(&mut self, _: EngineAskMove, _: &mut Self::Context) -> Self::Result {
        //TODO
    }
}

impl Handler<EngineMakeMove> for Engine {
    type Result = ();

    fn handle(&mut self, _: EngineMakeMove, _: &mut Self::Context) -> Self::Result {
        //TODO
    }
}
