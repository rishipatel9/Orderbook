use std::collections::BTreeMap;

use crate::{
    engine::service::{add_order, match_order}, global::{ORDERBOOKS}, inputs::{ Order, OrderBook, OrderBookDepth, OrderBookState, OrderType, Price, Symbol}
};

impl OrderBook {
    pub fn new(symbol: Symbol, price: Price) -> Self {
        Self {
            symbol,
            asks: BTreeMap::new(),
            bids: BTreeMap::new(),
            current_price: Some(price),
            current_best_ask: None,
            current_best_bid: None,
            last_trade_price: None,
        }
    }
    pub fn add_order(&mut self, order: Order) {
        add_order(self, order);
    }
    pub fn match_order(&mut self, incoming_order: &Order, order_type: OrderType) -> Vec<Order> {
        match_order(self, incoming_order, order_type)
    }
    pub fn get_depth(&self, levels: usize) -> OrderBookDepth {
        let mut bids: Vec<(u64, u64)> = self
            .bids
            .iter()
            .map(|(price, orders)| (*price, orders.iter().map(|o| o.qty).sum()))
            .collect();
        bids.sort_by(|a, b| b.0.cmp(&a.0)); // Sort by price descending
        bids.truncate(levels);

        let mut asks: Vec<(u64, u64)> = self
            .asks
            .iter()
            .map(|(price, orders)| (*price, orders.iter().map(|o| o.qty).sum()))
            .collect();
        asks.sort_by(|a, b| a.0.cmp(&b.0)); // Sort by price ascending
        asks.truncate(levels);

        OrderBookDepth { bids, asks }
    }
    
    pub fn get_orderbook_snapshot(
        symbol: Symbol,
    ) -> Result<OrderBookState, Box<dyn std::error::Error>> {
        let orderbooks = ORDERBOOKS.lock().unwrap();
        let orderbook = orderbooks.get(&symbol).ok_or("Symbol not found")?;

        Ok(OrderBookState {
            symbol: orderbook.symbol.clone(),
            current_price: orderbook.current_price,
            best_bid: orderbook.current_best_bid,
            best_ask: orderbook.current_best_ask,
            last_trade_price: orderbook.last_trade_price,
        })
    }
}
