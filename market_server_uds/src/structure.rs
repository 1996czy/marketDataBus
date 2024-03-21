use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::RwLock;
use tokio::net::TcpStream;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use tokio_tungstenite::{WebSocketStream, MaybeTlsStream};

// Self Type
pub type Arh<A, T> = Arc<RwLock<HashMap<A, T>>>;
pub type Amh<A, T> = Arc<Mutex<HashMap<A, T>>>;
pub type WS = WebSocketStream<MaybeTlsStream<TcpStream>>;

// struct for depth update
#[derive(Deserialize, Serialize)]
pub struct DiffDepth {
    pub s: String,
    pub u: i64,
    e: String,
    E: i64,
    U: i64,
    b: Vec<[String; 2]>,
    a: Vec<[String; 2]>
}

// struct for wraped depth update stream
#[derive(Deserialize, Serialize)]
pub struct DiffDepthStream {
    pub data: DiffDepth,
    stream: String,
}

// struct for subscribe or unsubscribe
#[derive(Deserialize, Serialize, Debug)]
pub struct Subscriber {
    pub method: String,
    pub params: Vec<String>,
}