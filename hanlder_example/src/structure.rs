use serde::{Deserialize, Serialize};


// struct for depth update
#[derive(Deserialize, Serialize, Debug)]
pub struct DiffDepth {
    pub s: String,
    pub u: i64,
    e: String,
    E: i64,
    U: i64,
    b: Vec<[String; 2]>,
    a: Vec<[String; 2]>
}

// struct for exchange symbols
#[derive(Deserialize, Serialize, Debug)]
pub struct ExchangeSymbols {
    pub symbol: String
}

// struct for exchange information
#[derive(Deserialize, Serialize, Debug)]
pub struct ExchangeInfo {
    pub symbols: Vec<ExchangeSymbols>
}