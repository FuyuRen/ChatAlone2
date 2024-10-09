use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use axum::{
    extract::ws::{Message, WebSocket},
    extract::{ConnectInfo, State, WebSocketUpgrade},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};

use axum_extra::extract::Query;
use dashmap::DashMap;
use futures::{SinkExt, StreamExt};
use serde::Deserialize;
use tokio::time::sleep;

use crate::jwt::{Jwt, JwtError};
use crate::room::{ChatRoom, RoomEvents};
use crate::server::AppState;
use crate::id::{GeneralId, Id, RoomId, UserId};
use crate::uuid::UUID;

#[derive(Clone)]
struct WsSharedState {
    rooms: Arc<DashMap<RoomId, ChatRoom>>, // key: room_id, value: ChatRoom
    users: Arc<DashMap<UserId, RoomId>>,     // key: user_id, value: room_id
}

impl WsSharedState {
    fn new() -> Self {
        Self {
            rooms: Arc::new(DashMap::new()),
            users: Arc::new(DashMap::new()),
        }
    }
}

impl Default for WsSharedState {
    fn default() -> Self {
        let room_id = Id::from_decoded(1919810);
        let user_1 = Id::from_decoded(114);
        let user_2 = Id::from_decoded(514);

        println!(
            "room_id: {}, user_1: {}, user_2: {}",
            room_id, user_1, user_2
        );

        let rooms = DashMap::new();
        rooms.insert(room_id.clone(), ChatRoom::default());
        let users = DashMap::new();
        users.insert(user_1, room_id.clone());
        users.insert(user_2, room_id);
        WsSharedState {
            rooms: Arc::new(rooms),
            users: Arc::new(users),
        }
    }
}

pub(crate) fn route(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/ws", get(handler))
        .with_state(state)
}

#[derive(Debug, Deserialize, Clone)]
struct WsConnQuery {
    token: Option<String>,
    session_id: Option<String>,
}

impl WsConnQuery {
    fn authorize(&self) -> Result<UserId, impl IntoResponse> {
        if let Some(token) = &self.token {
            match Jwt::parse_and_verify(token) {
                Err(e) => Err(e),
                Ok(jwt) => Ok(UserId::from_decoded(jwt.payload().user_id as u32)),
            }
        } else {
            Err(JwtError::MissingToken)
        }
    }
}

async fn handler(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Query(query): Query<WsConnQuery>,
    State(state): State<AppState>,
) -> Response {
    println!("{} connected.", addr);

    match query.authorize() {
        Err(e) => e.into_response(),
        Ok(uid) => ws.on_upgrade(move |socket| ws_handler(socket, uid, state)),
    }
}

async fn ws_handler(mut socket: WebSocket, user_id: UserId, state: AppState) -> () {
    // println!("user_id: {}.", user_id);
    // let (mut sender, mut receiver) = socket.split();
    // 
    // if let Err(e) = sender.send(Message::Text("Hello!!!".to_string())).await {
    //     println!("send error: {}", e);
    //     return;
    // }
    // 
    // let room_id = state.users.get(&user_id).unwrap().clone();
    // let room = state.rooms.get(&room_id).unwrap();
    // 
    // let tx = room.get_sender();
    // let mut rx = room.subscribe();
    // 
    // let mut recv_task = tokio::spawn(async move {
    //     loop {
    //         match receiver.next().await {
    //             Some(Ok(Message::Text(msg))) => {
    //                 if let Err(e) = tx.send(RoomEvents::Message(msg)) {
    //                     return Err(e);
    //                 }
    //             }
    //             _ => return Ok(()),
    //         }
    //     }
    // });
    // 
    // let mut send_task = tokio::spawn(async move {
    //     loop {
    //         match rx.recv().await {
    //             Ok(RoomEvents::Message(msg)) => {
    //                 if let Err(e) = sender.send(Message::Text(msg)).await {
    //                     return Err(e);
    //                 }
    //             }
    //             _ => return Ok(()),
    //         }
    //     }
    // });
    // 
    // tokio::select! {
    //     res = &mut recv_task => {
    //         println!("recv task finished: {:?}", res);
    //         send_task.abort()
    //     },
    //     res = &mut send_task => {
    //         println!("send task finished: {:?}", res);
    //         recv_task.abort()
    //     },
    // }

    // async move {
    //     loop {
    //         if let Err(e) = match receiver.next().await {
    //             Some(Ok(Message::Text(msg)))
    //                     => sender.send(Message::Text(msg)).await,
    //             Some(Ok(Message::Ping(msg)))
    //                     => sender.send(Message::Pong(msg)).await,
    //             None    => sender.send(Message::Text("bye".to_string())).await,
    //             _       => sender.send(Message::Text("没懂".to_string())).await,
    //         } {
    //             println!("recv error: {}", e);
    //             return;
    //         }
    //     }
    // }.await;
}
