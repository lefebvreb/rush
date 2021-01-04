// $ sudo ./target/debug/server
// Open http://82.65.218.243 in browser

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
                ctx.text("Hello from server");
            },
            _ => ctx.close(None),
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
