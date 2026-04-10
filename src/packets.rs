use serde::{Deserialize, Serialize};
use warp::filters::ws::Message;

use crate::{error::Error, state::SocketType, SharedState};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Packet {
    #[serde(rename = "ping")]
    Ping { data: String },

    #[serde(rename = "reset")]
    Reset,

    #[serde(rename = "target")]
    SetTarget { value: f64 },

    #[serde(rename = "current")]
    SetCurrent { value: f64 },

    #[serde(rename = "drink")]
    SetDrink { value: f64 },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Response {
    #[serde(rename = "ok")]
    Okay,
    #[serde(rename = "type")]
    Pong { data: String },
    #[serde(rename = "status")]
    Status {
        current: f64,
        target: f64,
        update: bool,
    },
    #[serde(rename = "pump")]
    PumpUpdate { amount: f64 },
}

pub async fn handle_message(
    packet: Packet,
    state: SharedState,
    socket_type: SocketType,
) -> Result<(), Error> {
    let response = match packet {
        Packet::Ping { data: _ } => Response::Pong {
            data: "pong".to_string(),
        },
        Packet::Reset => todo!(),
        Packet::SetTarget { value } => {
            state
                .alcohol_update(|a| {
                    a.update_target(value);
                })
                .await;
            Response::Okay
        }
        Packet::SetCurrent { value } => {
            state
                .alcohol_update(|a| {
                    a.calibrate(value);
                })
                .await;
            Response::Okay
        }
        Packet::SetDrink { value } => {
            state
                .alcohol_update(|a|{
                    a.update_drink(value);
                }).await;
            Response::Okay
        },
    };

    if let Ok(json) = serde_json::to_string(&response) {
        state.send_message(socket_type, Message::text(json)).await?;
    }
    Ok(())
}
