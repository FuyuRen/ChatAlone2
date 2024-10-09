use dashmap::DashMap;
use tokio::sync::broadcast;
use tokio::sync::broadcast::{Receiver, Sender};
use crate::id::{GeneralId, RoomId, UserId};

#[derive(Debug, Clone)]
pub enum RoomEvents {
    Message(String),
    UserJoined(UserId),
    UserLeft(UserId),
}

pub struct ChatRoom {
    room_id:        RoomId,
    room_title:     String,
    notifier:       Receiver<RoomEvents>,
    sender:         Sender<RoomEvents>,
}

impl ChatRoom {
    fn start(id: RoomId, name: String) -> Self {
        let (tx, rx) = broadcast::channel(32);
        Self {
            room_id: id,
            room_title: name,
            notifier: rx,
            sender: tx,
        }
    }

    fn add_user(&mut self, user_id: UserId) -> anyhow::Result<()> {
        Ok(())
    }

    pub fn subscribe(&self) -> Receiver<RoomEvents> {
        self.sender.subscribe()
    }

    pub fn get_sender(&self) -> Sender<RoomEvents> {
        self.sender.clone()
    }
}

impl Default for ChatRoom {
    fn default() -> Self {
        Self::start(RoomId::from_decoded(1919810), "test".to_string())
    }
}
