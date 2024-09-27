use dashmap::DashMap;
use tokio::sync::broadcast;
use tokio::sync::broadcast::Receiver;
use crate::uuid::UUID;

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
    sender:         broadcast::Sender<RoomEvents>,
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
}