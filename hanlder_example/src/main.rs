mod structure;

use tokio::net::UnixStream;
use tokio::task::spawn;
use tokio::io::AsyncReadExt;
use std::str::from_utf8;
use chrono::prelude::Utc;
use serde_json::from_str;
use reqwest;
use structure::*;
use tokio::time::{sleep, Duration};


async fn create_handler(symbol: String, port: &str) {
    let mut client = UnixStream::connect(port).await.unwrap();

    // execute one send and wait for data stream
    client.writable().await.expect("");
    client.try_write(symbol.as_bytes());
    
    let mut heap_msg = Vec::new();
    const buf_size: usize = 1024;
    let buf_len = 1024;
    loop {
        // read until there is an end
        let mut buf = [0; buf_size];

        match client.read(&mut buf).await {
            // receive 0 length data, maybe disconnected, break the loop
            Ok(0) => break,
            // receive n length data, print receive timestamp
            Ok(n) if (n < buf_len || (n == buf_len && buf[n - 1] == 125)) => {
                // notedown the time that data arrival
                let tmp_ts = Utc::now().timestamp_micros();
                let mut msg: DiffDepth;

                match heap_msg.len() {
                    // handle small message
                    0 => {
                        msg = from_str::<DiffDepth>(
                            from_utf8(&buf[..n]).unwrap()
                        ).unwrap();
                    },
                    // handle large message
                    _ => {
                        heap_msg.append(&mut buf[..n].to_vec());
                        
                        msg = from_str::<DiffDepth>(
                            from_utf8(&heap_msg).unwrap()
                        ).unwrap();
                        heap_msg.clear();
                    }
                }
                
                println!("receive ts: {:?}, data length: {:?}, data symbol: {:?}, data id: {:?}",
                            tmp_ts, n, msg.s, msg.u)
            },
            // receive buf_size data, append to heap and concat it with new data later
            Ok(m) => heap_msg.append(&mut buf.to_vec()),
            Err(e) => println!("Error: {:?}", e)
        }
    }
}


#[tokio::main]
async fn main() {
    // get exchange info
    let exchange_info = reqwest::get("https://api.binance.com/api/v3/exchangeInfo")
        .await
        .unwrap()
        .json::<ExchangeInfo>()
        .await
        .unwrap();
    
    // count the used symbols
    let mut count = 0;
    for symbol in &exchange_info.symbols{
        // let channel: String = symbol.symbol.to_lowercase() + "@depth@100ms";
        let channel: String = "bnbusdt@depth@100ms".to_string();
        spawn(create_handler(channel, "/tmp/sock"));
        count = count + 1;
        if count > 290 { break }
        sleep(Duration::from_millis(10)).await;
    }

    loop {
        sleep(Duration::from_millis(10000)).await;
    }
}
