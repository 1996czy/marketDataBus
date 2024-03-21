use chrono::prelude::Utc;
use tokio::net::unix::{OwnedWriteHalf};
use tokio::io::AsyncWriteExt;
use tokio::task::spawn;
use std::sync::Arc;
use tokio::sync::{mpsc, watch};
use serde_json::{from_str, to_string, to_vec};
use futures::stream::StreamExt;
use futures::sink::SinkExt;
use futures::stream::{SplitStream, SplitSink};
use webpki_roots::TLS_SERVER_ROOTS;
use rustls::{RootCertStore, ClientConfig, KeyLogFile};
use tokio_tungstenite::{connect_async_tls_with_config, Connector};
use tungstenite::protocol::Message;
use std::collections::HashMap;
use tokio::time::{sleep, Duration};
use tokio;


use crate::structure::*;


pub struct MarketStream {
}

impl MarketStream {
    pub fn new() -> Self {
        MarketStream {
        }
    }

    async fn ws_connect() -> WS {
        let base_ws = "wss://stream.binance.com:443/stream";

        // set local tls config to write sslkeylog
        let mut root_cert_store = RootCertStore::empty();
        root_cert_store.extend(
            TLS_SERVER_ROOTS.iter().cloned()
        );
        let mut config = ClientConfig::builder()
            .with_root_certificates(root_cert_store)
            .with_no_client_auth();
        config.key_log = Arc::new(KeyLogFile::new());
        let connector = Connector::Rustls(Arc::new(config));

        // make ws connection to market stream, return handshaked stream
        connect_async_tls_with_config(base_ws.to_string(), None, false, Some(connector))
        .await
        .unwrap()
        .0
    }

    pub async fn start(port2write: Arh<u16, OwnedWriteHalf>, symbol2ports: Arh<String, Vec<u16>>) -> mpsc::Sender<String> {
        // connect market streams and join the callback
        let client = Self::ws_connect().await;
        // split client to stream and sink, distributed to different threads.
        let (mut sender, mut receiver) = client.split();
        // create threads channel
        let (tx, mut rx) = mpsc::channel(100);
        spawn(Self::ws_subscribe(sender, rx));
        spawn(Self::ws_callback(receiver, port2write, symbol2ports));
        tx
    }

    pub async fn ws_subscribe(mut sender: SplitSink<WS, Message>, mut rx: mpsc::Receiver<String>) {
        loop {
            let mut streams = Vec::new();
            let task1 = async {
                loop {
                    match rx.recv().await {
                        Some(msg) => {
                            streams.push(msg);
                        },
                        None => break
                    }
                }
            };

            // binance require client to submit message at most 5 times per second
            let task2 = sleep(Duration::from_millis(200));
        
            tokio::select! {
                () = task1 => println!("break of receiver"),
                () = task2 => {
                    if streams.len() > 0 {
                        // send message if there is new subscription
                        sender.send(
                            Message::Text(to_string(&Subscriber {
                                method: "SUBSCRIBE".to_string(),
                                params: streams.clone()
                            }).expect(""))
                        ).await;
                    }
                }
            }
        }
    }

    async fn ws_callback(mut receiver: SplitStream<WS>, port2write: Arh<u16, OwnedWriteHalf>, symbol2ports: Arh<String, Vec<u16>>) {
        // receive data from stream and handle it
        loop {
            match receiver.next().await{
                Some(msg) => {
                    // notedown the time that data arrival
                    let tmp_ts = Utc::now().timestamp_micros();

                    let text_msg = &(msg
                                    .unwrap()
                                    .into_text()
                                    .unwrap());

                    // get the right channel
                    if text_msg.contains("depthUpdate") {
                        let deser_msg = from_str::<DiffDepthStream>(text_msg)
                                                                    .unwrap()
                                                                    .data;
                        // find the port
                        let read_lock_map = symbol2ports
                                            .read()
                                            .await;

                        let port_vec = read_lock_map
                                        .get(&deser_msg.s)
                                        .unwrap();
                        
                        // create bytes data
                        let byte_msg = to_vec(&deser_msg).unwrap();

                        for port in port_vec {
                            let mut write_lock_map = port2write
                            .write()
                            .await;

                            // get relative sender for the port
                            let sender = write_lock_map
                            .get_mut(port)
                            .unwrap();

                            // send data to handler
                            sender.try_write(&byte_msg);
                        }
                        println!("receive ts: {:?}, data symbol: {:?}, data id: {:?}",
                        tmp_ts,
                        deser_msg.s,
                        deser_msg.u);
                    }
                },
                None => println!("None data received")
            }
        }
        println!("Market Module Disconnected");
    }
}
