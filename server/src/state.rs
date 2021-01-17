use std::collections::HashMap;

use actix::{Actor, Addr, Context, Handler};

use chess::{Color, Game, Move, MoveGenerator};

use crate::wsclient::WsClient;
use crate::messages::{ClientCommand, Connect, Disconnect, ServerCommand};

// The role of a client
enum Role {
    Playing(Color),
    Spectating,
}

// The global state of the website
#[derive(Default)]
pub struct State {
    clients: HashMap<Addr<WsClient>, Role>,
    game: Game,
    moves: HashMap<String, Move>,
    white: Option<Addr<WsClient>>,
    black: Option<Addr<WsClient>>,
}

impl State {
    // Return true if that client is playing
    fn is_playing(&self, addr: &Addr<WsClient>) -> bool {
        match match self.game.get_color() {
            Color::White => &self.white,
            Color::Black => &self.black,
        } {
            Some(a) => *a == *addr,
            _ => false,
        }
    }
}

impl Actor for State {
    type Context = Context<Self>;
}

// Upon a new connection
impl Handler<Connect> for State {
    type Result = ();

    fn handle(&mut self, msg: Connect, _: &mut Self::Context) -> Self::Result {
        self.clients.insert(msg.addr, Role::Spectating);
    }
}

// Upon disconnection
impl Handler<Disconnect> for State {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Self::Context) -> Self::Result {
        self.clients.remove(&msg.addr);

        match &self.white {
            Some(a) if *a == msg.addr => self.white = None,
            _ => (),
        }
        match &self.black {
            Some(a) if *a == msg.addr => self.black = None,
            _ => (),
        }
    }
}

impl Handler<ClientCommand> for State {
    type Result = ();

    // Upon receiving a command from a client
    fn handle(&mut self, msg: ClientCommand, _: &mut Self::Context) -> Self::Result {
        match msg {
            // A demand to see the legal moves of that position
            ClientCommand::Legals {addr} =>
                addr.do_send(ServerCommand::Legals(
                    self.moves
                        .keys()
                        .fold(String::new(), |acc, s| acc + s + " ")
                )),
            // A demand to play a move
            ClientCommand::Move {addr, s} => if self.is_playing(&addr) {
                if let Some(&mv) = self.moves.get(&s) {
                    self.game.do_move(mv);
                    self.moves = self.game.legals().to_map();

                    let ans = ServerCommand::Fen(self.game.to_string());
                    for addr in self.clients.keys() {
                        addr.do_send(ans.clone());
                    }
                }
            }
            // A demand to get the authorization to play
            ClientCommand::Play {addr} => {
                if self.is_playing(&addr) {
                    return;
                }

                match &self.white {
                    None => {
                        if let Some(r) = self.clients.get_mut(&addr) {
                            *r = Role::Playing(Color::White);
                        }
                        addr.do_send(ServerCommand::Role("w".to_string()));
                    }
                    _ => match &self.black {
                        None => {
                            if let Some(r) = self.clients.get_mut(&addr) {
                                *r = Role::Playing(Color::Black);
                            }
                            addr.do_send(ServerCommand::Role("b".to_string()));
                        }
                        _ => (),
                    },
                }
            }
        }
    }
}