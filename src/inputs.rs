use std::collections::{BTreeMap};

use actix_web::cookie::time::Time;
use serde::{Deserialize, Serialize};

#[derive(Deserialize,Serialize,Debug)]
pub struct CreateOrderInput{
    pub symbol :Symbol,
    pub price:f64,
    pub quantity:u32,
    pub user_id:u32,
    pub side:Side
}

#[derive(Deserialize,Serialize,Debug,Clone, Copy)]
pub enum Side{
    Buy,Sell
}
#[derive(Deserialize,Serialize,Debug, Clone, PartialEq, Eq, Hash)]
pub enum Symbol {
    BTCUSD,
    ETHUSD,
    SOLUSD
}
type OrderId = u64;
type Price = u64;
type Quantity = u64;

#[derive(Debug, Clone,Copy)]
pub struct Order {
    pub id: OrderId,
    pub price: Price,
    pub qty: Quantity,
    pub is_buy: bool,
    pub time:Time
}

#[derive(Debug)]
pub struct OrderBook {
    pub bids: BTreeMap<Price, Vec<Order>>, 
    pub asks: BTreeMap<Price, Vec<Order>>, 
    pub symbol:Symbol,
    pub last_trade_price: Option<Price>,
    pub current_best_bid: Option<Price>,
    pub current_best_ask: Option<Price>,
}
