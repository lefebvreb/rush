#![allow(dead_code, unused_variables)]

// $ sudo ./target/debug/server
// Open http://82.65.218.243 in browser

use actix::{Actor, Addr};
use actix_files as fs;
use actix_web::{App, Error, HttpRequest, HttpResponse, HttpServer, web};
use actix_web_actors::ws;

mod wsclient;
mod messages;
mod state;

use wsclient::WsClient;
use state::State;

// The default IP address
const IP: &str = "192.168.0.24:80";

// Fired when a new ws connection is requested
async fn ws_index(request: HttpRequest, stream: web::Payload, state: web::Data<Addr<State>>) -> Result<HttpResponse, Error> {
    let connection = WsClient::new(state.get_ref().clone());
    ws::start(connection, &request, stream)
}

// Launch the server
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // The global state
    let state = State::default().start();

    // Starts the HTTP server and starts listening
    HttpServer::new(move || {
        App::new()
            .data(state.clone())
            .service(web::resource("/ws/").route(web::get().to(ws_index)))
            .service(fs::Files::new("/", "www/").index_file("index.html"))
    })
    .bind(IP)?
    .run()
    .await
}
