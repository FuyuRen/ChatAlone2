use dashmap::DashMap;
use tokio::sync::broadcast;
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
    notifier:       broadcast::Receiver<RoomEvents>,
    sender:         broadcast::Sender<RoomEvents>,
    subscribers:    DashMap<UUID, broadcast::Sender<RoomEvents>>
}

impl ChatRoom {
    fn start(id: UUID, name: String) -> Self {
        let (tx, rx) = broadcast::channel(32);
        Self {
            room_id: id,
            room_title: name,
            notifier: rx,
            sender: tx,
            subscribers: DashMap::new()
        }
    }

    fn add_user(&mut self, user_id: UUID) -> anyhow::Result<()> {

    }
}