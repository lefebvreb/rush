use std::collections::HashMap;

use actix::{Actor, Addr, Context, Handler};

use chess::{Color, Game, GameStatus, Move, MoveGenerator, ThreefoldCounter};
use engine::{Engine, EngineAskMove, EngineMakeMove, EngineMove};

use crate::wsclient::WsClient;
use crate::messages::{ClientInfo, ClientMove, ClientRequestEngine, ClientRequestPlay, Connect, Disconnect};

//#################################################################################################
//
//                                      struct Player
//
//#################################################################################################

// Represent a player: either a client or the engine
#[derive(Clone)]
enum Player {
    Client(Addr<WsClient>),
    Engine,
}

// ================================ impl

impl Player {
    // Return true if the player is that client
    fn is(maybe_player: &Option<Player>, addr: &Addr<WsClient>) -> bool {
        matches!(maybe_player, Some(Player::Client(a)) if *a == *addr)
    }

    // Return true if that Option<Player> could be set to Some(new_player)
    fn try_set(maybe_player: &mut Option<Player>, new_player: &Player) -> bool {
        if maybe_player.is_none() {
            *maybe_player = Some(new_player.clone());
            true
        } else {
            false
        }
    }
}

//#################################################################################################
//
//                                    struct Addresses
//
//#################################################################################################

// A struct to hold the addresses of the different actors
struct Addresses {
    engine: Addr<Engine>,
    clients: HashMap<Addr<WsClient>, Option<Color>>,
    white: Option<Player>,
    black: Option<Player>,
}

// ================================ impl

impl Addresses {
    // Initialize the address repository
    fn new(engine: Addr<Engine>) -> Addresses {
        Addresses {
            engine,
            clients: HashMap::default(),
            white: None,
            black: None,
        }
    }

    // Return the (maybe?) color of that client
    fn get_color_of(&self, addr: &Addr<WsClient>) -> Option<Color> {
        if Player::is(&self.white, addr) {
            Some(Color::White)
        } else if Player::is(&self.black, addr) {
            Some(Color::Black)
        } else {
            None
        }
    }

    // Add a new client to the addresses
    fn new_client(&mut self, addr: &Addr<WsClient>) {
        self.clients.insert(addr.clone(), None);
    }

    // Try to give a new role to that client, return true on success
    fn try_give_role(&mut self, addr: &Addr<WsClient>) -> bool {
        let new_player = Player::Client(addr.clone());

        self.get_color_of(addr).is_none() && (
            Player::try_set(&mut self.white, &new_player) ||
            Player::try_set(&mut self.black, &new_player)
        )
    }

    // Try to give the engine a new role
    fn try_place_engine(&mut self) -> bool {
        let new_player = Player::Engine;

        Player::try_set(&mut self.white, &new_player) || 
        Player::try_set(&mut self.black, &new_player)
    }

    // Remove a client from all addresses
    fn remove_client(&mut self, addr: &Addr<WsClient>) {
        self.clients.remove(addr);

        match self.get_color_of(addr) {
            Some(Color::White) => self.white = None,
            Some(Color::Black) => self.black = None,
            _ => (),
        }
    }

    // Return an iterators to the clients
    fn clients(&self) -> impl Iterator<Item = &Addr<WsClient>> {
        self.clients.keys()
    }

    // Return the address of the engine
    fn engine(&self) -> &Addr<Engine> {
        &self.engine
    }

    // Return the (maybe?) player corresponding to that color
    fn get_player(&self, color: Color) -> &Option<Player> {
        match color {
            Color::White => &self.white,
            Color::Black => &self.black,
        }
    }
}

//#################################################################################################
//
//                                    struct GameState
//
//#################################################################################################

// A struct to hold the Game and it's auxilliary types
#[derive(Debug)]
struct GameState {
    counter: ThreefoldCounter,
    game: Game,
    history: Vec<Move>,
    legals: HashMap<String, Move>,
    status: GameStatus,
}

// ================================ impl

impl GameState {
    // Return true if that color is the one playing
    fn is_playing(&self, maybe_color: impl Into<Option<Color>>) -> bool {
        matches!(maybe_color.into(), Some(c) if c == self.game.get_color())
    }

    // Try to do a move and return true if it was successful
    fn try_move(&mut self, s: String) -> bool {
        if let Some(&mv) = self.legals.get(&s) {
            let (status, game, legals) = self.game.do_move_status(&mut self.counter, mv);
            self.game = game;
            self.history.push(mv);
            self.legals = legals;
            self.status = status;
            true
        } else {
            false
        }
    }

    // Return the currently playing color
    fn playing_color(&self) -> Color {
        self.game.get_color()
    }
}

// ================================ traits impl

impl Default for GameState {
    // Initialize a GameState
    fn default() -> GameState {
        let game = Game::default();
        let legals = game.legals().to_map();

        GameState {
            counter: ThreefoldCounter::default(),
            game,
            history: Vec::new(),
            legals,
            status: GameStatus::default(),
        }
    }
}

//#################################################################################################
//
//                                      struct View
//
//#################################################################################################

// A struct to hold the necessary informations for clients
struct View {
    legals: String,
    state: ClientInfo,
}

// ================================ impl

impl View {
    // Creates a View from the Game State
    fn new(game_state: &GameState) -> View {
        View {
            legals: game_state.legals.keys()
                .map(|s| s.clone())
                .collect::<Vec<_>>()
                .join(","),
            state: ClientInfo(
                format!(
                    "state {} {} {}",
                    game_state.game.get_board().to_string(),
                    game_state.history.iter()
                        .map(|mv| mv.to_string())
                        .collect::<Vec<_>>()
                        .join(","),
                    match game_state.status {
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
            ),
        }
    }

    // Return a proper message for informing clients of the state of the game
    fn state(&self) -> ClientInfo {
        self.state.clone()
    }

    // Return a proper message for informing a client of his role and capabilities
    fn info(&self, addresses: &Addresses, game_state: &GameState, addr: &Addr<WsClient>) -> ClientInfo {
        let maybe_color = addresses.get_color_of(addr);

        maybe_color.map_or(
            ClientInfo("info s ".to_string()),
            |color| ClientInfo(
                format!(
                    "info {} {}", 
                    color, 
                    if game_state.is_playing(color) {&self.legals} else {""}
                )
            ),
        )
    }
}

//#################################################################################################
//
//                                      struct State
//
//#################################################################################################

// A struct to hold every necessary pieces of the server
pub struct State {
    addresses: Addresses,
    game_state: GameState,
    view: View,
}

// ================================ pub impl

impl State {
    // Initialize the State of the server
    pub fn new(engine: &Addr<Engine>) -> State {
        let addresses = Addresses::new(engine.clone());
        let game_state = GameState::default();
        let view = View::new(&game_state);

        State {
            addresses,
            game_state,
            view,
        }
    }
}

// ================================ impl

impl State {
    // Send the state to the given client
    fn send_state(&self, addr: &Addr<WsClient>) {
        addr.do_send(self.view.state());
    }

    // Send infos to the given clients
    fn send_info(&self, addr: &Addr<WsClient>) {
        addr.do_send(self.view.info(&self.addresses, &self.game_state, addr));
    }

    // Ask whoever's turn it is to play
    fn ask_for_move(&self) {
        match self.addresses.get_player(self.game_state.playing_color()) {
            Some(Player::Client(a)) => self.send_info(&a),
            Some(Player::Engine) => self.addresses.engine().do_send(EngineAskMove),
            _ => (),
        }
    }

    // Try to play the move given by it's string representation
    fn try_move(&mut self, s: String) {
        // If the move is valid
        if self.game_state.try_move(s.clone()) {
            // Send the move to the engine
            self.addresses.engine().do_send(EngineMakeMove(s));

            // Update the view
            self.view = View::new(&self.game_state);

            // Send new state to every clients
            for addr in self.addresses.clients() {
                self.send_state(addr);
            }

            // Send new infos to playing
            self.ask_for_move();
        }
    }

    // Return true if the engine is currently playing
    fn is_engine_playing(&self) -> bool {
        matches!(
            self.addresses.get_player(self.game_state.playing_color()),
            Some(Player::Engine),
        )
    }
}

// ================================ traits impl

impl Actor for State {
    type Context = Context<Self>;
}

impl Handler<Connect> for State {
    type Result = ();

    // On a new connection
    fn handle(&mut self, msg: Connect, _: &mut Self::Context) -> Self::Result {
        let addr = &msg.addr;

        self.addresses.new_client(addr);
        self.send_info(addr);
        self.send_state(addr);
    }
}

impl Handler<Disconnect> for State {
    type Result = ();

    // On a client's disconnection
    fn handle(&mut self, msg: Disconnect, _: &mut Self::Context) -> Self::Result {
        self.addresses.remove_client(&msg.addr);
    }
}

impl Handler<ClientMove> for State {
    type Result = ();

    // When a clients plays a move
    fn handle(&mut self, msg: ClientMove, _: &mut Self::Context) -> Self::Result {
        let addr = &msg.addr;

        // If it's that client's turn
        if self.game_state.is_playing(self.addresses.get_color_of(addr)) {
            // Try to do the move
            self.try_move(msg.s);
        }

        // Send new infos to old player
        self.send_info(addr);
    }    
}

impl Handler<ClientRequestEngine> for State {
    type Result = ();

    // When a clients invites the engine to play
    fn handle(&mut self, _: ClientRequestEngine, _: &mut Self::Context) -> Self::Result {
        // If the engine is playing
        if self.is_engine_playing() {
            // Try to give it another role
            self.addresses.try_place_engine();
        } else {
            // Try to give it another role and then ask it to play
            self.addresses.try_place_engine();
            self.ask_for_move();
        }
    }    
}

impl Handler<ClientRequestPlay> for State {
    type Result = ();

    // When a client requests to play
    fn handle(&mut self, msg: ClientRequestPlay, _: &mut Self::Context) -> Self::Result {
        let addr = &msg.addr;

        // If he got a new role, send him the new infos
        if self.addresses.try_give_role(addr) {
            self.send_info(addr);
        }
    }    
}

impl Handler<EngineMove> for State {
    type Result = ();

    // When the engine requests a move
    fn handle(&mut self, msg: EngineMove, _: &mut Self::Context) -> Self::Result {
        self.try_move(msg.0);
    }
}