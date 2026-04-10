use futures_util::stream::SplitSink;
use futures_util::StreamExt;
use tokio::sync::RwLock;
use warp::filters::ws::Message;
use warp::ws::{WebSocket, Ws};
use warp::Filter;

use crate::alcohol::Alcohol;
use crate::error::Error;
use crate::packets::{handle_message, Packet};
use crate::state::{Connection, SharedState, State, SocketType};

pub mod alcohol;
pub mod error;
pub mod packets;
pub mod state;
pub mod worker;

#[tokio::main]
async fn main() {
    let state = State {
        connection: RwLock::new(Connection {
            client: None,
            pump: None,
        }),
        alcohol: RwLock::new(Alcohol::default()),
    };

    let mut _state: &'static mut State = Box::leak(Box::new(state));

    let client_ws = warp::path("ws")
        .and(warp::path("client"))
        .and(warp::ws())
        .map(|ws: Ws| ws.on_upgrade(|socket| handle_socket(socket, _state, SocketType::Client)))
        .with(warp::cors().allow_any_origin());

    let pump_ws = warp::path("ws")
        .and(warp::path("pump"))
        .and(warp::ws())
        .map(|ws: Ws| ws.on_upgrade(|socket| handle_socket(socket, _state, SocketType::Pump)))
        .with(warp::cors().allow_any_origin());

    let routes = client_ws.or(pump_ws);

    tokio::spawn(async {
        worker::worker(_state).await;
    });

    println!("Server running on http://127.0.0.1:3030");
    println!("  - Client endpoint: ws://127.0.0.1:3030/ws/client");
    println!("  - Pump endpoint: ws://127.0.0.1:3030/ws/pump");
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

async fn handle_socket(ws: WebSocket, state: SharedState, socket_type: SocketType) {
    let (sender, mut receiver) = ws.split();

    state.set_connection(socket_type, Box::new(sender)).await;

    while let Some(result) = receiver.next().await {
        if let Ok(msg) = result {
            if msg.is_text() || msg.is_binary() {
                let text = msg.to_str().unwrap_or("{}");
                match serde_json::from_str::<Packet>(text) {
                    Ok(packet) => {
                        let _ = handle_message(packet, state, socket_type).await;
                    }
                    Err(err) => {
                        let _ = state.send_message(socket_type, Message::text(err.to_string())).await;
                    }
                }
            }
        }
    }

    state.disconnect(socket_type).await;
}
