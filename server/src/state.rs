use std::collections::HashMap;

use actix::{Actor, Addr, Context, Handler};

use chess::{Color, Game, GameStatus, Move, MoveGenerator, ThreefoldCounter};
use engine::{Engine, EngineCommand, EngineMove};

use crate::wsclient::WsClient;
use crate::messages::{ClientInfo, ClientMove, ClientRequestEngine, ClientRequestPlay, Connect, Disconnect};

// A player in the game: a client or the engine
enum Player {
    Client(Addr<WsClient>),
    Engine,
}

// The global state of the website
pub struct State {
    game: Game,
    counter: ThreefoldCounter,
    status: GameStatus,
    legals: HashMap<String, Move>,

    engine: Addr<Engine>,
    clients: HashMap<Addr<WsClient>, Option<Color>>,
    white: Option<Player>,
    black: Option<Player>,

    history: Vec<String>,
    legals_str: String,
    state_msg: ClientInfo,
}

impl State {
    // Return the color of that client, if he is playing in the game
    fn get_color_of(&self, addr: &Addr<WsClient>) -> Option<Color> {
        match &self.white {
            Some(Player::Client(a)) if *a == *addr => Some(Color::White),
            _ => match &self.black {
                Some(Player::Client(a)) if *a == *addr => Some(Color::Black),
                _ => None,
            },
        }
    }

    fn is_playing(&self, addr: &Addr<WsClient>) -> bool {
        matches!(self.get_color_of(addr), Some(c) if c == self.game.get_color())
    }

    fn update_state_msg(&mut self) {
        self.state_msg = ClientInfo {
            text: format!(
                "state {} {} {}",
                self.game.get_board(),
                self.history.iter()
                    .map(|s| s.clone())
                    .collect::<Vec<_>>()
                    .join(","),
                match self.status {
                    GameStatus::Playing {playing} => match playing {
                        Color::White => "w",
                        Color::Black => "b",
                    }
                    GameStatus::Drawn => "d",
                    GameStatus::Won {winner} => match winner {
                        Color::White => "wm",
                        Color::Black => "bm",
                    }
                },
            )
        }
    }

    fn get_info_msg(&self, addr: &Addr<WsClient>) -> ClientInfo {
        ClientInfo {
            text: format!(
                "info {} {}",
                self.get_color_of(addr).map_or("s".to_string(), |c| c.to_string()),
                if self.is_playing(&addr) {
                    &self.legals_str
                } else {
                    ""
                },
            )
        }
    }

    // Do a move from it's String
    fn do_move(&mut self, s: String) {
        if let Some(&mv) = self.legals.get(&s) {
            // Update game
            let (status, game, legals) = self.game.do_move_status(&mut self.counter, mv);
            self.game = game;
            self.history.push(s.clone());
            self.legals = legals;
            self.status = status;

            // Send last move to engine
            self.engine.do_send(EngineCommand::Move(s));

            // Update legals string
            self.legals_str = self.legals.keys()
                .map(|s| s.clone())
                .collect::<Vec<_>>()
                .join(",");

            // Send new state to every client
            self.update_state_msg();
            for addr in self.clients.keys() {
                addr.do_send(self.state_msg.clone());
            }

            // Send info to new player
            match self.game.get_color() {
                Color::White => self.white.as_ref(),
                Color::Black => self.black.as_ref(),
            }.map(|p| match p {
                Player::Client(a) => a.do_send(self.get_info_msg(&a)),
                Player::Engine => self.engine.do_send(EngineCommand::AskMove),
            });
        }
    }
}

impl Actor for State {
    type Context = Context<Self>;
}

impl Handler<Connect> for State {
    type Result = ();

    // When a client connects, give him any information he may need
    fn handle(&mut self, msg: Connect, _: &mut Self::Context) -> Self::Result {
        msg.addr.do_send(self.state_msg.clone());
        msg.addr.do_send(self.get_info_msg(&msg.addr));
        self.clients.insert(msg.addr, None);
    }
}

// Upon disconnection
impl Handler<Disconnect> for State {
    type Result = ();

    // When a client disconnect, remove him from any role he may have held
    fn handle(&mut self, msg: Disconnect, _: &mut Self::Context) -> Self::Result {
        self.clients.remove(&msg.addr);

        match self.get_color_of(&msg.addr) {
            Some(Color::White) => self.white = None,
            Some(Color::Black) => self.black = None,
            _ => (),
        }   
    }
}

impl Handler<ClientMove> for State {
    type Result = ();

    fn handle(&mut self, msg: ClientMove, _: &mut Self::Context) -> Self::Result {
        // Do the move if addr is playing
        if self.get_color_of(&msg.addr).map_or(false, |c| c == self.game.get_color()) {
            self.do_move(msg.text);
        }

        // Send info to old player
        msg.addr.do_send(self.get_info_msg(&msg.addr))
    }    
}

impl Handler<ClientRequestEngine> for State {
    type Result = ();

    // When a clients invites the engine to play
    fn handle(&mut self, _: ClientRequestEngine, _: &mut Self::Context) -> Self::Result {
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

impl Handler<ClientRequestPlay> for State {
    type Result = ();

    fn handle(&mut self, msg: ClientRequestPlay, _: &mut Self::Context) -> Self::Result {
        // If client is already playing, do nothing
        if self.get_color_of(&msg.addr).is_some() {
            return;
        }

        // See if there is any empty role the client could fill
        if self.white.is_none() {
            self.white = Some(Player::Client(msg.addr.clone()));
            *self.clients.get_mut(&msg.addr).unwrap() = Some(Color::White);
        } else if self.black.is_none() {
            self.black = Some(Player::Client(msg.addr.clone()));
            *self.clients.get_mut(&msg.addr).unwrap() = Some(Color::Black);
        }

        // Give info to the new player
        msg.addr.do_send(self.get_info_msg(&msg.addr));
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
        let game = Game::default();
        let legals = game.legals().to_map();

        State {
            game,
            counter: ThreefoldCounter::default(),
            status: GameStatus::default(),
            history: Vec::new(),
            legals,

            engine: Engine::default().start(),
            clients: HashMap::new(),
            white: None,
            black: None,

            legals_str: "b2b4,g2g3,d2d4,e2e4,f2f4,g1h3,b1a3,b2b3,c2c3,g1f3,e2e3,f2f3,h2h3,a2a3,d2d3,b1c3,g2g4,c2c4,a2a4,h2h4".to_string(),
            state_msg: ClientInfo {
                text: "state rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR  w".to_string(),
            },
        }
    }
}