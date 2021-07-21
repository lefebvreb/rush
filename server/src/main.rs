use std::env::args;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::{Error, Result};
use warp::Filter;

mod game;
mod messages;
mod sockets;

use crate::sockets::Sockets;

// The default port the server listens on.
const DEFAULT_PORT: u16 = 5050;

// Tries to parse the port from the program's arguments.
fn parse_port() -> Result<u16> {
    // Get the program arguments.
    let mut args = args();
    // Extract the executable's path.
    args.next().ok_or(Error::msg("Cannot get executable path"))?;
    // Get and parse port.
    args.next().map_or(Ok(DEFAULT_PORT), |s| Ok(u16::from_str(s.as_str())?))
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // Parse the port.
    let port = match parse_port() {
        Ok(port) => port,
        Err(e) => {
            eprintln!("Error while parsing port: {}", e);
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
    println!("Launching server @ http://localhost:{}", port);
    warp::serve(routes)
        .run(([127, 0, 0, 1], port))
        .await;
}
