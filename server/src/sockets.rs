use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use futures::{FutureExt, StreamExt};
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::{RwLock, mpsc};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::Message;

use crate::game::Game;

//#################################################################################################
//
//                                         struct State
//
//#################################################################################################

// Manages the different connections, as well as the state of the server.
#[derive(Debug)]
pub struct State {
    // The atomic counter keeping trace of wich ids have been attributed already.
    next_uid: AtomicUsize,
    // The shared hashmap containing all of our sinks.
    senders: RwLock<HashMap<usize, UnboundedSender<Result<Message, warp::Error>>>>,
    // The game state.
    game: RwLock<Game>,
}

// ================================ pub impl

impl State {
    // Creates a new Socket object, managing all connections.
    pub fn new() -> State {
        State {
            next_uid: AtomicUsize::new(0),
            senders: RwLock::new(HashMap::new()),
            game: RwLock::new(Game::new()),
        }
    }

    // Handle a new connections through it's life cycle.
    pub async fn handle_connection(self: Arc<Self>, ws: warp::ws::WebSocket) {
        // Get the next valid unique id.
        let uid = self.next_uid.fetch_add(1, Ordering::Relaxed);

        // Split the socket into it's sink and sender components.
        let (tx, mut rx) = ws.split();

        {
            // Construct a new mpsc channel and add the sender end to
            // our shared hashmap.
            let (mpsc_tx, mpsc_rx) = mpsc::unbounded_channel();
            self.senders.write().await.insert(uid, mpsc_tx);

            // React to a message coming from the program by
            // forwarding it through the socket.
            let mpsc_rx = UnboundedReceiverStream::new(mpsc_rx);
            tokio::task::spawn(mpsc_rx.forward(tx).map(|res| {
                if let Err(err) = res {
                    eprintln!("WebSocket send error: {}", err);
                }
            }));
        }

        // Listen for incoming messages from the web.
        while let Some(res) = rx.next().await {
            match res {
                // On correct message.
                Ok(msg) => self.on_message(uid, msg).await,
                // On error, prints it and breaks out of the event loop.
                Err(err) => {
                    eprintln!("WebSocket receive error: {}", err);
                    break;
                },
            }
        }

        // On disconnection, remove the client from our list.
        self.senders.write().await.remove(&uid);
    }
}

// ================================ impl

impl State {
    // Send a message to a single client, if it is still connected.
    async fn send(&self, uid: usize, msg: Message) {
        if let Some(tx) = self.senders.read().await.get(&uid) {
            tx.send(Ok(msg)).ok();
        }
    }

    // Broadcasts a string message to all connected clients.
    async fn broadcast(&self, msg: Message) {
        for tx in self.senders.read().await.values() {
            tx.send(Ok(msg.clone())).ok();
        }
    }

    // Called when a new message is received.
    async fn on_message(&self, uid: usize, msg: Message) {
        // TODO: decode message and act.
        todo!()
    }
}