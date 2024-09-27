use std::net::SocketAddr;
use std::time::Duration;

use axum::{
    Router,
    extract::{
        ConnectInfo,
        State,
        WebSocketUpgrade
    },
    extract::ws::{
        Message,
        WebSocket
    },
    response::{
        IntoResponse,
        Response
    },
    routing::get,
};

use axum_extra::extract::Query;
use dashmap::DashMap;
use futures::{SinkExt, StreamExt};
use serde::Deserialize;
use tokio::time::sleep;

use crate::jwt::{Jwt, JwtError};
use crate::room::{ChatRoom, RoomEvents};
use crate::server::AppState;
use crate::uuid::UUID;


struct WsSharedState {
    rooms: DashMap<UUID, ChatRoom>, // key: room_id, value: ChatRoom
    users: DashMap<UUID, UUID>, // key: user_id, value: room_id
}

impl WsSharedState {
    fn new() -> Self {
        Self {
            rooms: DashMap::new(),
            users: DashMap::new(),
        }
    }
}

impl Default for WsSharedState {
    fn default() -> Self {
        Self::new()
    }
}

pub(crate) fn route(app_state: AppState) -> Router<AppState> {
    Router::new()
        .route("/ws", get(handler))
        .with_state(app_state)
}

#[derive(Debug, Deserialize, Clone)]
struct WsConnQuery {
    token:      Option<String>,
    session_id: Option<String>
}

impl WsConnQuery {
    fn authorize(&self) -> Result<UUID, impl IntoResponse> {
        if let Some(token) = &self.token {
            match Jwt::parse_and_verify(token) {
                Err(e) => Err(e),
                Ok(jwt) => Ok(jwt.payload().uuid()),
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
    State(ws_state): State<WsSharedState>,
) -> Response {

    println!("{} connected", addr);

    let res = query.authorize();
    if let Err(e) = res { return e.into_response() }

    ws.on_upgrade(move |socket| ws_handler(socket, res.unwrap(), ws_state))
}

async fn ws_handler(mut socket: WebSocket, user_id: UUID, state: WsSharedState) -> () {
    let (mut sender, mut receiver) = socket.split();

    if let Err(e) = sender.send(Message::Text("Hello!!!".to_string())).await {
        println!("send error: {}", e);
        return;
    }

    let mut vec = vec![];

    let room_id = state.users.get(&user_id).unwrap().clone();
    let room = state.rooms.get(&room_id).unwrap();

    let mut rx = room.subscribe();


    let recv_task = async move {
        loop {
            match receiver.next().await {
                Some(Ok(Message::Text(msg))) => vec.push(msg),
                _ => return
            }
        }
    };

    let send_task = async move {
        loop {
            if let RoomEvents::Message(msg) = rx.recv().await {
                sender.send(Message::Text(msg)).await.unwrap();
            }
        }
    };

    tokio::select! {
        _ = recv_task => send_task.abort(),
        _ = send_task => {},
    }

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
