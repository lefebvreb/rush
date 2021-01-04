#![allow(dead_code, unused_variables, unused_macros)]

use std::env::args;

use awc::Client;
use awc::ws::Message;

use futures_util::{sink::SinkExt, stream::StreamExt};

const DEFAULT_URI: &str = "ws://192.168.0.24/ws/";

#[actix_web::main]
async fn main() {
    let mut args = args();

    // Exectuable's path
    args.next().unwrap();

    let (_, mut ws) = Client::new()
        .ws(match args.next() {
            Some(uri) => uri,
            _ => DEFAULT_URI.to_owned(),
        })
        .connect()
        .await
        .expect("Cannot connect to server");

    ws.send(Message::Text("Hello from engine client".to_string()))
        .await
        .expect("Cannot send message to server");

    println!("{:?}", ws.next().await);
}