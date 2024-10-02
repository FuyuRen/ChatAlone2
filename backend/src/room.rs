use crate::uuid::UUID;
use dashmap::DashMap;
use tokio::sync::broadcast;
use tokio::sync::broadcast::{Receiver, Sender};

#[derive(Debug, Clone)]
pub enum RoomEvents {
    Message(String),
    UserJoined(UUID),
    UserLeft(UUID),
}

pub struct ChatRoom {
    room_id:        UUID,
    room_title:     String,
    notifier:       Receiver<RoomEvents>,
    sender:         Sender<RoomEvents>,
}

impl ChatRoom {
    fn start(id: UUID, name: String) -> Self {
        let (tx, rx) = broadcast::channel(32);
        Self {
            room_id: id,
            room_title: name,
            notifier: rx,
            sender: tx,
        }
    }

    fn add_user(&mut self, user_id: UUID) -> anyhow::Result<()> {
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
        Self::start(UUID::from(1919810_i64), "test".to_string())
    }
}
