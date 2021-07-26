use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use anyhow::Result;
use futures::{FutureExt, StreamExt};
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::{RwLock, mpsc};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::Message;

use crate::game::Game;
use crate::messages::{Command, Response};

//#################################################################################################
//
//                                         struct State
//
//#################################################################################################

/// Manages the different connections, as well as the state of the server.
#[derive(Debug)]
pub struct Sockets {
    // The atomic counter keeping trace of wich ids have been attributed already.
    next_uid: AtomicUsize,
    // The shared hashmap containing all of our sinks.
    senders: RwLock<HashMap<usize, UnboundedSender<Result<Message, warp::Error>>>>,
    // The game state, which the state communcates with through this channel.
    game_tx: UnboundedSender<Command>,
}

// ================================ pub impl

impl Sockets {
    /// Creates a new Socket object, managing all connections.
    pub fn new() -> Arc<Self> {
        // Create channels to communicate with the game state.
        let (tx, mut game_rx) = mpsc::unbounded_channel();
        let game_tx = Game::new(tx);

        // Construct the state object.
        let state = Arc::new(Self {
            next_uid: AtomicUsize::new(0),
            senders: RwLock::new(HashMap::new()),
            game_tx,
        });

        // Copy a reference to this state object and create a task forwarding
        // messages from the game state to the web.
        let state_cpy = state.clone();
        tokio::spawn(async move {
            let state = state_cpy;

            // While receiving messages from the game state, forward them
            // according to it's demands.
            while let Some(res) = game_rx.recv().await {
                match res {
                    Ok(Response::Broadcast(msg)) => state.broadcast(msg).await,
                    Ok(Response::Send{dest, msg}) => state.send(dest, msg).await,
                    _ => (), // Invalid action in context, simply ignore.
                }
            }
        });

        state
    }

    /// Handle a new connections through it's life cycle.
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
            tokio::spawn(mpsc_rx.forward(tx).map(|res| {
                if let Err(e) = res {
                    eprintln!("WebSocket send error: {}", e);
                }
            }));
        }

        // Request the game state to send the welcome message to the new client.
        self.game_tx.send(Command::Welcome(uid)).ok();

        // Listen for incoming messages from the web.
        while let Some(res) = rx.next().await {
            match res {
                // On correct message.
                Ok(msg) => {
                    // Message indicates that the client disconnected.
                    if msg.is_close() {
                        break;
                    }

                    // If the message was incorrect, print the error to the terminal.
                    if let Err(e) = self.on_message(msg) {
                        eprintln!("Erroneous order: {}", e);
                    }
                },
                // On error, prints it and breaks out of the event loop.
                Err(e) => {
                    eprintln!("WebSocket receive error: {}", e);
                    break;
                },
            }
        }

        // On disconnection, remove the client from our list.
        self.senders.write().await.remove(&uid);
    }
}

// ================================ impl

impl Sockets {
    /// Sends a message to a specified client, if it is still connected.
    async fn send(&self, uid: usize, msg: Message) {
        if let Some(tx) = self.senders.read().await.get(&uid) {
            tx.send(Ok(msg)).ok();
        }
    }

    /// Broadcasts a message to all connected clients.
    async fn broadcast(&self, msg: Message) {
        for tx in self.senders.read().await.values() {
            tx.send(Ok(msg.clone())).ok();
        }
    }

    /// Upon receiving a message from a client, parses it and forwards it to the game state.
    fn on_message(&self, msg: Message) -> Result<()> {
        let command = Command::from_msg(msg)?;
        self.game_tx.send(command)?;
        Ok(())
    }
}