// $ sudo ./target/debug/server
// Open http://82.65.218.243 in browser

// Communication protocol:

// semi-fen: <board> <color> <moves>

// Client commands:
// fen => get the semi-fen of the game
// play <move> => tries to play the move, get the resulting fen
// role => "w" if white, "b" if black, "s" if spectator

use actix::{Actor, StreamHandler};
use actix_files as fs;
use actix_web::{App, Error, HttpRequest, HttpResponse, HttpServer, web};
use actix_web_actors::ws;

struct MyWebSocket;

async fn ws_index(r: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    ws::start(MyWebSocket, &r, stream)
}

impl Actor for MyWebSocket {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWebSocket {
    fn handle(
        &mut self,
        msg: Result<ws::Message, ws::ProtocolError>,
        ctx: &mut Self::Context,
    ) {
        match msg {
            Ok(ws::Message::Text(text)) => {
                println!("Got a message: {}", text);
                ctx.text("Hello from rust");
            },
            /*Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
            }*/
            _ => (),
        }
    }
}

// Launch the server
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(web::resource("/ws/").route(web::get().to(ws_index)))
            .service(fs::Files::new("/", "www/").index_file("index.html"))
    })
    .bind("192.168.0.24:80")?
    .run()
    .await
}
