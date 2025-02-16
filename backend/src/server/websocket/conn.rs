use std::ops::DerefMut;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::time::Duration;
use axum::extract::ws::{Utf8Bytes, WebSocket};
use dashmap::DashMap;
use futures::stream::{SplitSink, SplitStream};
use futures::{SinkExt, StreamExt};
use axum::extract::ws::Message;
use tokio::sync::{mpsc, Mutex};
use tokio::task::JoinHandle;
use crate::server::websocket::error::Error;

#[derive(Debug)]
pub struct WsClient {
    alive_cnt:  Arc<AtomicBool>,
    sender:     mpsc::Sender<Message>,
    rx_queue:   Arc<Mutex<mpsc::Receiver<Message>>>,
    ws_task:    Arc<Mutex<JoinHandle<i32>>>,
}


impl WsClient {
    pub fn new(tx: mpsc::Sender<Message>,
               rx: mpsc::Receiver<Message>,
               task: JoinHandle<i32>
    ) -> Self {
        WsClient {
            sender:     tx,
            alive_cnt:  Arc::new(AtomicBool::new(true)),
            rx_queue:   Arc::new(Mutex::new(rx)),
            ws_task:    Arc::new(Mutex::new(task)),
        }
    }
    
    pub fn get_sender(&self) -> mpsc::Sender<Message> {
        self.sender.clone()
    }

    pub fn rx_handler(text: Utf8Bytes) -> () {
        let text = text.to_string();
    }

    pub async fn task(&self, heartbeat_freq_s: u64) {
        let tx = self.sender.clone();
        let rx = self.rx_queue.clone();

        let ws_task = self.ws_task.clone();
        let mut ws_task = ws_task.lock().await;


        // Every ${heartbeat_freq_s} seconds, kill all not alive connections.
        let alive_cnt = self.alive_cnt.clone();
        let mut keep_alive_task: JoinHandle<Result<(), Error>> = tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(heartbeat_freq_s)).await;
                if !alive_cnt.load(Ordering::SeqCst) {
                    return Err(Error::HeartBeatTimeout)
                };
                alive_cnt.store(false, Ordering::SeqCst);
            }
        });

        let alive_cnt = self.alive_cnt.clone();
        let mut rx_task = tokio::spawn(async move {
            let mut rx = rx.lock().await;
            let rx = rx.deref_mut();
            while let Some(msg) = rx.recv().await {
                match msg {
                    Message::Close(_) => {
                        return Ok(());
                    },
                    Message::Ping(_) => {
                        // Reset alive_cnt to true, which marks the connection alive.
                        alive_cnt.store(true, Ordering::SeqCst);
                        if let Err(e) = tx.send(msg).await {
                            return Err(Error::SendError(e));
                        }
                    },
                    Message::Text(text) => {
                        Self::rx_handler(text);
                    }
                    _ => {
                        alive_cnt.store(true, Ordering::SeqCst);
                    }
                }
            }
            Ok(())
        });

        tokio::select! {
            _ = &mut keep_alive_task => {
                println!("Keep alive task finished.");
                rx_task.abort();
                ws_task.abort();
            },
            _ = &mut rx_task => {
                println!("Receive task finished.");
                keep_alive_task.abort();
                ws_task.abort();
            },
            _ = &mut ws_task.deref_mut() => {
                println!("WebSocket task finished.");
                keep_alive_task.abort();
                rx_task.abort();
            },
        }
    }

    pub async fn send<T: Into<Message>>(&self, msg: T) -> Result<(), Error> {
        let sender = self.get_sender();
        
        if let Err(e) = sender.send(msg.into()).await {
            Err(Error::SendError(e))
        } else {
            Ok(())
        }
    }
}

// pub struct WsConnections(DashMap<PkRoomId, DashMap<PkUserId, Connection>>);