use std::sync::Mutex;
use std::{collections::HashMap, sync::Arc};

use redis::Connection;

use crate::inputs::{OrderBook, Symbol};

lazy_static::lazy_static! {
    pub static ref ORDERBOOKS: Arc<Mutex<HashMap<Symbol, OrderBook>>> = {
        let mut books = HashMap::new();
        books.insert(Symbol::BTCUSD, OrderBook::new(Symbol::BTCUSD));
        books.insert(Symbol::ETHUSD, OrderBook::new(Symbol::ETHUSD));
        books.insert(Symbol::SOLUSD, OrderBook::new(Symbol::SOLUSD));
        Arc::new(Mutex::new(books))
    };
    pub static ref NEXT_ORDER_ID: Arc<Mutex<u64>> = Arc::new(Mutex::new(1));
}

