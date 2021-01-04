#![allow(dead_code, unused_variables)]

// $ sudo ./target/debug/server
// Open http://82.65.218.243 in browser

use std::sync::Mutex;

use actix_files as fs;
use actix_web::{App, Error, HttpRequest, HttpResponse, HttpServer, web};
use actix_web_actors::ws;

mod game;
use game::OnlineGame;

async fn ws_index(request: HttpRequest, stream: web::Payload, game: web::Data<OnlineGame>) -> Result<HttpResponse, Error> {
    ws::start(game.lock().unwrap(), &request, stream)
}

// Launch the server
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        let game = web::Data::new(OnlineGame::default());

        App::new()
            .app_data(game)
            .service(web::resource("/ws/").route(web::get().to(ws_index)))
            .service(fs::Files::new("/", "www/").index_file("index.html"))
    })
    .bind("192.168.0.24:80")?
    .run()
    .await
}
