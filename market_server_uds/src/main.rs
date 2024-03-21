mod structure;
mod market;
use chrono::prelude::Utc;
use tokio::net::{UnixListener, UnixStream};
use tokio::task::spawn;
use tokio::time::{sleep, Duration};
use market::MarketStream;
use tokio::io::AsyncReadExt;
use std::str::from_utf8;
use tokio::sync::{mpsc, watch};
use tokio::net::unix::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::RwLock;
use std::sync::Arc;
use std::collections::HashMap;
use structure::{Amh, Arh, DiffDepth};


async fn handle_socket(
    mut socket: UnixStream,
    addr: u16,
    tx: mpsc::Sender<String>, 
    mut port2write: Arh<u16, OwnedWriteHalf>,
    mut symbol2ports: Arh<String, Vec<u16>>) {
        
    // split the tcp stream into 2 part, receiving part is running in this thread,
    // sending part is put into map to be called by other thread.
    let (mut read_half, mut write_half) = socket.into_split();

    // update hashmap
    port2write
    .write()
    .await
    .insert(addr, write_half);

    loop {
        const buf_len: usize = 32;
        let mut buf = [0; buf_len];
        match read_half.read(&mut buf).await {
            // receive 0 length data, maybe disconnected, break the loop
            Ok(0) => break,
            // receive n length data, making transmit
            Ok(n) if n <= buf_len.try_into().unwrap() => {
                let msg = from_utf8(&buf[..n])
                                .unwrap()
                                .to_string();
                // parse message and update hashmap
                let stream_list = msg
                                .split("/")
                                .collect::<Vec<&str>>();
                for stream in &stream_list {
                    let symbol = stream
                                .split_once("@")
                                .unwrap()
                                .0
                                .to_string()
                                .to_uppercase();

                    let mut symbol2ports_writer = symbol2ports
                                                .write()
                                                .await;
                    match symbol2ports_writer.get_mut(&symbol) {
                        Some(vector) => vector.push(addr),
                        None => {
                            symbol2ports_writer.insert(symbol.to_string().to_uppercase(), vec![addr]);
                            // send message to subscribe thread
                            tx.send(stream.to_string()).await;
                        }
                    }
                }
            },
            Ok(_) => println!("message too long!"),
            Err(e) => println!("Error in socket {:?}", e)
        }
    }
}


#[tokio::main]
async fn main() {
    // two maps are necessary here: one is to record the port according to symbols
    // the other is to record the port according to tcp writer
    let mut symbol2ports: Arh<String, Vec<u16>> = Arc::new(RwLock::new(HashMap::new()));
    let mut port2write: Arh<u16, OwnedWriteHalf> = Arc::new(RwLock::new(HashMap::new()));

    // initilize market connection to binance
    let mut tx = MarketStream::start(port2write.clone(), symbol2ports.clone()).await;

    // establish a listener, spawn a thread whenever receiving connecting request
    let listener = UnixListener::bind("/tmp/sock").unwrap();
    let mut port: u16 = 0;
    loop {
        match listener.accept().await {
            Ok((socket, addr)) => {
                spawn(
                    handle_socket(
                        socket,
                        port,
                        tx.clone(),
                        port2write.clone(),
                        symbol2ports.clone()
                    )
                );
                port = port + 1;
            },
            Err(e) => println!("listener error: {:?}", e)
        }
    }
}
