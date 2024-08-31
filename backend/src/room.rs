use dashmap::DashMap;
pub struct ChatRoom {
    name:  String,
    users: Vec<String>,
}