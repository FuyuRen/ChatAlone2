use dashmap::DashMap;
use crate::uuid::UUID;

pub struct ChatRoom {
    room_id:    UUID,
    room_name:  String,
    users:      Vec<i32>,
}