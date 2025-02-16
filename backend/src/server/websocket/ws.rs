use std::net::SocketAddr;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use anyhow::anyhow;
use axum::extract::{ConnectInfo, State, WebSocketUpgrade};
use axum::extract::ws::{Message, Utf8Bytes, WebSocket};
use axum::response::Response;
use axum::{Router, ServiceExt};
use axum::routing::get;
use futures::{SinkExt, StreamExt};

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::{mpsc, Mutex};
use tokio::task::JoinHandle;
use crate::id::{GeneralId, UserId};
use crate::jwt::Jwt;
use crate::server::AppState;
use crate::server::websocket::conn::WsClient;
use crate::server::websocket::error::Error;
use super::event::{Author, ChatContent, Event, MediaMeta, Payload, Scope};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsSignal {
    sn:         u32,
    timestamp:  u32,
    payload:    Payload,
}

impl Into<Message> for WsSignal {
    fn into(self) -> Message {
        let signal = serde_json::to_string(&self)
            .unwrap_or(
                json!({
                    "sn": -1,
                    "timestamp": -1,
                    "payload":{
                        "err": "Internal Server Error"
                    }
                }).to_string());
        Message::Text(signal.into())
    }
}
// 
// impl<'a> Into<Message> for &'a str {
//     fn into(self) -> Message {
//         todo!()
//     }
// }


pub(crate) fn route(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/ws", get(handler))
        .with_state(state)
}

async fn handler(
    jwt: Jwt,
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<AppState>,
) -> Response {
    println!("{} connected.", addr);
    let pk_uid = UserId::from_encoded(jwt.user_id() as u32).decode();
    ws.on_upgrade(move |socket| ws_handler(socket, pk_uid, state))
}

async fn ws_handler(mut socket: WebSocket, pk_uid: u32, state: AppState) -> () {
    println!("user_id: {}.", pk_uid);
    let (mut sender, mut receiver) = socket.split();

    if let Err(e) = sender.send(Message::Text("Hello!!!".into())).await {
        println!("send error: {}", e);
        return ;
    }

    // let tx = state.users.get(&pk_uid).unwrap().clone();
    let (send_tx, mut send_rx): (mpsc::Sender<Message>, mpsc::Receiver<Message>) = mpsc::channel(32);
    let (recv_tx, recv_rx): (mpsc::Sender<Message>, mpsc::Receiver<Message>) = mpsc::channel(32);

    let mut send_task = tokio::spawn(async move {
        loop {
            match send_rx.recv().await {
                Some(msg) => {
                    if let Err(e) = sender.send(msg).await {
                        return Err(anyhow!("channel closed"))
                    }
                }
                None => {
                    return Ok(());
                }
            }
        }
    });

    let mut recv_task = tokio::spawn(async move {
        loop {
            match receiver.next().await {
                Some(Ok(msg)) => {
                    if let Err(e) = recv_tx.send(msg).await {
                        return Err(anyhow!("channel closed"))
                    }
                },
                Some(Err(e)) => {
                    // Conn stopped due to error
                    return Err(anyhow!(e));
                }
                None => {
                    // Conn closed
                    return Ok(());
                }
            }
        }
    });

    let ws_task = tokio::spawn(async move {
        let res = tokio::select! {
            res = &mut recv_task => {
                println!("recv task finished: {:?}", res);
                send_task.abort();
                match res {
                    Ok(_) => {1}
                    Err(_) => {2}
                }
            },
            res = &mut send_task => {
                println!("send task finished: {:?}", res);
                recv_task.abort();
                match res {
                    Ok(_) => {3}
                    Err(_) => {4}
                }
            },
        };
        res
    });

    state.users.insert(pk_uid, WsClient::new(send_tx, recv_rx, ws_task));
    state.users.get(&pk_uid).unwrap().task(30).await;

    return ;

    // let mut send_task = tokio::spawn(async move {
    //     loop {
    //         match rx.recv().await {
    //             Ok(RoomEvents::Message(msg)) => {
    //                 if let Err(e) = sender.send(Message::Text(msg.into())).await {
    //                     return Err(e);
    //                 }
    //             }
    //             _ => return Ok(()),
    //         }
    //     }
    // });

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
    //             None    => sender.send(Message::Text("bye".into())).await,
    //             _       => sender.send(Message::Text("没懂".into())).await,
    //         } {
    //             println!("recv error: {}", e);
    //             return;
    //         }
    //     }
    // }.await;
}


#[tokio::test]
async fn test_main() -> Result<(), Error>{
    // start an axum server
    let state = AppState {
        db_conn: Default::default(),
        users: Arc::new(DashMap::new()),
    };
    let app = Router::new().merge(route(state.clone())).with_state(state.clone());
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    let serve = axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>());

    let mut server_task = tokio::spawn(async move {
        println!("server running on {}", addr);
        serve.await.expect("TODO: panic message");
    });

    let mut send_task = tokio::spawn(async move {
        let users = state.users.clone();
        loop {
            for res in users.iter() {
                println!("user_id: {}.", res.key());
                let user = res.value();
                if let Err(e) = user.send("Hello!!!".to_string()).await {
                    println!("Failed to send message: {}", e);
                    return;
                }
                // let sender = user.sender.clone();
                // if let Err(e) = sender.send(Message::Text("Hello!!!".into())).await {
                //     println!("Failed to send message: {}", e);
                //     return;
                // }
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    });

    tokio::select! {
        res = &mut server_task => {
            println!("server task finished: {:?}", res);
            send_task.abort();
        },
        res = &mut send_task => {
            println!("send task finished: {:?}", res);
            server_task.abort();
        },
    }
    Ok(())
}

#[tokio::test]
async fn test_event() {
    // use std::time::{SystemTime, UNIX_EPOCH};
    // let event = EventBase::new(114514, None, None, None);
    // let event = EventData::Text(event, Some(json!({ "test": "114514" })));
    // let event = Event::new(event, 1919810, SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as u32);
    let meta = MediaMeta::ImageJPEG { dimensions: (114, 514), size: 1919810 };
    let image = ChatContent::Image { file_id: 114, thumbnail_id: 514, meta };
    let payload = Payload {
        id: 114,
        author: Author::User { id: 114514 },
        scope: Scope::Room { lone_id: 1919, room_id: 810 },
        event: Event::Chat { event_id: 1111111, content: image, quote: Some(6666) },
    };
    let event = WsSignal {
        sn: 6,
        timestamp: 1919810,
        payload,
    };
    println!("{}", serde_json::to_string(&event).unwrap())
}

