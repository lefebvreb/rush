use std::collections::HashMap;

use actix::{Actor, Addr, Context, Handler};

use chess::{Color, Game, Move, MoveGenerator};
use engine::{Engine, EngineCommand, EngineMove};

use crate::wsclient::WsClient;
use crate::messages::{ClientDemand, Connect, Disconnect, ClientCommand};

enum Player {
    Client(Addr<WsClient>),
    Engine,
}

// The global state of the website
pub struct State {
    game: Game,
    legals: HashMap<String, Move>,
    history: String,
    engine: Addr<Engine>,
    clients: HashMap<Addr<WsClient>, Option<Color>>,
    white: Option<Player>,
    black: Option<Player>,
}

impl State {
    // Return the color of that client, if he is playing in the game
    fn get_client_of(&self, addr: &Addr<WsClient>) -> Option<Color> {
        match &self.white {
            Some(Player::Client(a)) if *a == *addr => Some(Color::White),
            _ => match &self.black {
                Some(Player::Client(a)) if *a == *addr => Some(Color::Black),
                _ => None,
            },
        }
    }

    // Compute the new state command to give to clients
    fn state(&self) -> ClientCommand {
        ClientCommand::State(format!(
            "state {} {} {}",
            self.game.get_board(),
            self.history,
            self.game.get_color(), // TODO: Implement Game::status in chess crate
        ))
    }

    // Compute the new info command to give to a specific client
    fn info(&self, addr: &Addr<WsClient>) -> ClientCommand {
        ClientCommand::Info(format!(
            "info {} {}",
            self.get_client_of(addr)
                .map_or("s".to_string(), |c| c.to_string()),
            match self.get_client_of(addr) {
                Some(c) if c == self.game.get_color() => self.legals
                    .keys()
                    .map(|s| s.clone())
                    .collect::<Vec<_>>()
                    .join(","),
                _ => String::new(),
            },
        ))
    }

    // Do a move from it's String
    fn do_move(&mut self, s: String) {
        if let Some(&mv) = self.legals.get(&s) {
            // Update game
            self.game = self.game.do_move(mv);
            self.legals = self.game.legals().to_map();
            self.history += &format!(",{}", s);

            // Send move to engine
            self.engine.do_send(EngineCommand::Move(s));

            // Send new state to every client
            let ans = self.state();
            for addr in self.clients.keys() {
                addr.do_send(ans.clone());
            }

            // Send info to new player
            match self.game.get_color() {
                Color::White => self.white.as_ref(),
                Color::Black => self.black.as_ref(),
            }.map(|p| match p {
                Player::Client(a) => a.do_send(self.info(&a)),
                Player::Engine => self.engine.do_send(EngineCommand::AskMove),
            });
        }
    }
}

impl Actor for State {
    type Context = Context<Self>;
}

// Upon a new connection
impl Handler<Connect> for State {
    type Result = ();

    // When a client connects, give him any information he may need
    fn handle(&mut self, msg: Connect, _: &mut Self::Context) -> Self::Result {
        msg.addr.do_send(self.state());
        msg.addr.do_send(self.info(&msg.addr));
        self.clients.insert(msg.addr, None);
    }
}

// Upon disconnection
impl Handler<Disconnect> for State {
    type Result = ();

    // When a client disconnect, remove him from any role he may have held
    fn handle(&mut self, msg: Disconnect, _: &mut Self::Context) -> Self::Result {
        self.clients.remove(&msg.addr);

        match self.get_client_of(&msg.addr) {
            Some(Color::White) => self.white = None,
            Some(Color::Black) => self.black = None,
            _ => (),
        }   
    }
}

impl Handler<ClientDemand> for State {
    type Result = ();

    // Upon receiving a command from a client
    fn handle(&mut self, msg: ClientDemand, _: &mut Self::Context) -> Self::Result {
        match msg {
            // When a client tries a move
            ClientDemand::Move {addr, s} => {
                // Do the move
                self.do_move(s);

                // Send info to old player
                addr.do_send(self.info(&addr))
            }
            // When a client requests to play
            ClientDemand::Play {addr} => {
                // If client is already playing, do nothing
                if self.get_client_of(&addr).is_some() {
                    return;
                }

                // See if there is any empty role the client could fill
                if self.white.is_none() {
                    self.white = Some(Player::Client(addr.clone()));
                    *self.clients.get_mut(&addr).unwrap() = Some(Color::White);
                } else if self.black.is_none() {
                    self.black = Some(Player::Client(addr.clone()));
                    *self.clients.get_mut(&addr).unwrap() = Some(Color::Black);
                }

                // Give info to the new player
                addr.do_send(self.info(&addr));
            }
            // When a clients invites the engine to play
            ClientDemand::Invite => {
                if self.white.is_none() {
                    self.white = Some(Player::Engine);
                    if self.game.get_color() == Color::White {
                        self.engine.do_send(EngineCommand::AskMove)
                    }
                } else if self.black.is_none() {
                    self.black = Some(Player::Engine);
                    if self.game.get_color() == Color::Black {
                        self.engine.do_send(EngineCommand::AskMove)
                    }
                }
            }
        }
    }
}

impl Handler<EngineMove> for State {
    type Result = ();

    fn handle(&mut self, msg: EngineMove, _: &mut Self::Context) -> Self::Result {
        // Do the move
        self.do_move(msg.0);
    }
}

impl Default for State {
    // Create a new State, in it's default state
    fn default() -> State {
        State {
            game: Game::default(),
            legals: HashMap::new(),
            history: String::new(),
            engine: Engine::default().start(),
            clients: HashMap::new(),
            white: None,
            black: None,
        }
    }
}