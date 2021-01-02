// $ sudo ./target/debug/server
// Open http://82.65.218.243 in browser

// Communication protocol:

// semi-fen: <board> <color> <moves>

// Client commands:
// fen => get the semi-fen of the game
// play <move> => tries to play the move, get the resulting fen
// role => "w" if white, "b" if black, "s" if spectator

use actix_files as fs;
use actix_web::{App, HttpServer};

// Launch the server
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(fs::Files::new("/", "www/").index_file("index.html"))
    })
    .bind("192.168.0.24:80")?
    .run()
    .await
}
