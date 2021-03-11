use actix::{Actor, Context, Handler, Message};

#[derive(Default)]
pub struct Engine;

/// A message the engine may send to ask to play a move
#[derive(Message)]
#[rtype(result = "()")]
pub struct EngineMove(pub String);

/// Ask the engine to compute a move and give back the result
#[derive(Message)]
#[rtype(result = "()")]
pub struct EngineAskMove;

/// Tell the engine a move has been played
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
