use futures_util::{stream::SplitSink, SinkExt};
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use warp::filters::ws::{Message, WebSocket};

use crate::{alcohol::Alcohol, error::Error};

pub type SharedState = &'static State;
pub type MessageSender = SplitSink<WebSocket, Message>;

#[derive(Debug, Clone, Copy)]
pub enum SocketType {
    Client,
    Pump,
}

pub struct Connection {
    pub client: Option<Box<MessageSender>>,
    pub pump: Option<Box<MessageSender>>,
}

pub struct State {
    pub connection: RwLock<Connection>,
    pub alcohol: RwLock<Alcohol>,
}

impl State {
    pub async fn send_message(&self, socket_type: SocketType, msg: Message) -> Result<(), Error> {
        let mut conn = self.connection.write().await;
        let sender = match socket_type {
            SocketType::Client => &mut conn.client,
            SocketType::Pump => &mut conn.pump,
        };

        if let Some(tx) = sender {
            tx.send(msg).await.map_err(|_| Error::NotConnected)
        } else {
            Err(Error::NotConnected)
        }
    }

    pub async fn set_connection(&self, socket_type: SocketType, sender: Box<MessageSender>) {
        let mut conn = self.connection.write().await;
        match socket_type {
            SocketType::Client => conn.client = Some(sender),
            SocketType::Pump => conn.pump = Some(sender),
        }
    }

    pub async fn disconnect(&self, socket_type: SocketType) {
        let mut conn = self.connection.write().await;
        match socket_type {
            SocketType::Client => conn.client = None,
            SocketType::Pump => conn.pump = None,
        }
    }

    pub async fn alcohol_update<T>(&self, fun: T)
    where
        T: FnOnce(&mut RwLockWriteGuard<'_, Alcohol>) -> (),
    {
        let mut lock = self.alcohol.write().await;
        fun(&mut lock);
        drop(lock);
    }

    pub async fn alcohol_read<T>(&self, fun: T)
    where
        T: FnOnce(&RwLockReadGuard<'_, Alcohol>) -> (),
    {
        let mut lock = self.alcohol.read().await;
        fun(&mut lock);
        drop(lock);
    }
}
