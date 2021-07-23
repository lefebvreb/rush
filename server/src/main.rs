use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;

use clap::{Arg, App};
use warp::Filter;

mod game;
mod messages;
mod sockets;

use crate::sockets::Sockets;

/// The default address the server listens on.
const DEFAULT_ADDRESS: &str = "127.0.0.1:5050";

#[tokio::main(flavor = "current_thread")]
async fn main() {
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
        .get_matches();

    // Parses the socket address.
    let addr_str = args.value_of("address").unwrap();
    let addr = match SocketAddr::from_str(addr_str) {
        Ok(addr) => addr,
        Err(_) => {
            eprintln!("Failed to parse address: {}.", addr_str);
            return;
        },
    };

    // Initializes the chess library.
    chess::init();

    // Creates our state object and converts it into a warp filter.
    let sockets = {
        let sockets = Sockets::new();
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
    println!("Launching server @ http://{}", addr_str);
    warp::serve(routes)
        .run(addr)
        .await;
}
