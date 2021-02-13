use std::env::args;

use actix::{Actor, Addr};
use actix_files::Files;
use actix_web::{App, Error, HttpRequest, HttpResponse, HttpServer, web};
use actix_web::http::ContentEncoding;
use actix_web::middleware::Compress;
use actix_web_actors::ws;

mod wsclient;
mod messages;
mod state;

use wsclient::WsClient;
use state::State;

// The default address to which the server is bound
const DEFAULT_ADDR: &str = "127.0.0.1:8080";

// Fired when a new ws connection is requested
async fn ws_index(request: HttpRequest, stream: web::Payload, state: web::Data<Addr<State>>) -> Result<HttpResponse, Error> {
    let client = WsClient::new(state.get_ref().clone());
    ws::start(client, &request, stream)
}

// Launch the server
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut args = args();

    // Executable path
    args.next().unwrap();

    // IP address
    let address = args.next().unwrap_or(DEFAULT_ADDR.to_string());

    // The global state
    let state = State::default().start();

    // Start the HTTP server and start listening
    HttpServer::new(move || {
        App::new()
            .data(state.clone())
            .wrap(Compress::new(ContentEncoding::Gzip))
            .service(web::resource("/ws/").route(web::get().to(ws_index)))
            .service(Files::new("/", "www/dist/").index_file("index.html"))
    })
    .bind(address)?
    .run()
    .await

    // CLI
}
