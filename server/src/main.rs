use std::net::SocketAddr;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::{Error, Result};
use clap::{Arg, App};
use warp::Filter;

use chess::board::Board;
use chess::book::Book;
use engine::Engine;

mod game;
mod messages;
mod sockets;

use crate::sockets::Sockets;

/// The default address the server listens on.
const DEFAULT_ADDRESS: &str = "127.0.0.1:5050";

/// The fen used for the default position.
const DEFAULT_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    // Gets the arguments.
    let args = App::new("Rush chess engine server")
        .version(engine::VERSION)
        .author("Benjamin Lefebvre")
        .about("A web server backend for playing againt the Rush chess engine.")
        .arg(Arg::with_name("address")
            .index(1)
            .value_name("ADDRESS")
            .default_value(DEFAULT_ADDRESS)
            .help("Sets the address to bind the http server to, uses localhost by default.")
            .takes_value(true))
        .arg(Arg::with_name("fen")
            .short("f")
            .long("fen")
            .value_name("FEN")
            .default_value(DEFAULT_FEN)
            .help("Sets the fen string to use as the starting position, use double quotes to give everything in a single argument.")
            .takes_value(true))
        .arg(Arg::with_name("book")
            .short("b")
            .long("book")
            .value_name("BOOK")
            .help("Gives the path to a polyglot book (.bin), that the engine will use whenever it can.")
            .takes_value(true))
        .get_matches();

    // Parses the socket address.
    let addr_str = args.value_of("address").unwrap();
    let addr = match SocketAddr::from_str(addr_str) {
        Ok(addr) => addr,
        Err(_) => return Err(Error::msg(format!("Failed to parse address: {}.", addr_str))),
    };

    // The book that may be used to lookup moves.
    let book = if let Some(book_path) = args.value_of("book") {
        Some(Book::open(Path::new(book_path))?)
    } else {
        None
    };

    // Initializes the chess library.
    chess::init();

    // Creates our state object and converts it into a warp filter.
    let sockets = {
        let board = Board::new(args.value_of("fen").unwrap())?;
        let engine = Engine::new(board, book);

        let sockets = Sockets::new(engine);
        warp::any().map(move || sockets.clone())
    };

    // Creates the routing of our app.
    let routes = {
        // For getting the websocket resource.
        let ws = warp::path("ws")
            .and(warp::ws())
            .and(sockets)
            .map(|ws: warp::ws::Ws, state: Arc<Sockets>| {
                ws.on_upgrade(move |socket| {
                    state.handle_connection(socket)
                })
            });

        // For files.
        let public = warp::get()
            .and(warp::fs::dir("www/public"))
            .with(warp::compression::gzip());

        public.or(ws)
    };

    // Launches the server, printing the used port.
    log::info!("Launching server @ http://{}", addr_str);
    warp::serve(routes)
        .run(addr)
        .await;

    Ok(())
}
