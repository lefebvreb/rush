use actix::{Actor, Context, Handler, Message};

use chess::MoveGenerator;

use crate::{shared, threads};

/// Ask the engine to compute a move and give back the result
#[derive(Message)]
#[rtype(result = "String")]
pub struct EngineAskMove;

/// Tell the engine a move has been played
#[derive(Message)]
#[rtype(result = "()")]
pub struct EngineMakeMove {
    pub mv: String,
}

//#################################################################################################
//
//                                     struct Engine
//
//#################################################################################################

pub struct Engine;

// ================================ traits impl

impl Actor for Engine {
    type Context = Context<Self>;
}

impl Default for Engine {
    fn default() -> Engine {
        shared::initialize();
        threads::start_threads();
        Engine
    }
}

impl Handler<EngineAskMove> for Engine {
    type Result = String;

    fn handle(&mut self, msg: EngineAskMove, _: &mut Self::Context) -> String {
        threads::launch_search()
            .unwrap_or_else(|| shared::game().legals().next().unwrap())
            .to_string()
    }
}

impl Handler<EngineMakeMove> for Engine {
    type Result = ();

    fn handle(&mut self, msg: EngineMakeMove, _: &mut Self::Context) {
        shared::do_move(
            shared::game().parse_move(&msg.mv).unwrap()
        );
    }
}
