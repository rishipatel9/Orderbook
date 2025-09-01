use std::collections::{BTreeMap};
use serde::{Deserialize, Serialize};

#[derive(Deserialize,Serialize,Debug)]
pub struct CreateOrderInput{
    pub symbol :Symbol,
    pub price:f64,
    pub quantity:u32,
    pub user_id:u32,
    pub side:Side,
    pub order_type:OrderType
}

#[derive(Deserialize,Serialize,Debug,Clone, Copy,PartialEq)]
pub enum Side{
    Buy,Sell
}
#[derive(Serialize,Deserialize,Debug,Clone,Copy,PartialEq)]
pub enum OrderType{
    Limit,Market
}
#[derive(Deserialize,Serialize,Debug, Clone, PartialEq, Eq, Hash)]
pub enum Symbol {
    BTCUSD,
    ETHUSD,
    SOLUSD
}
type OrderId = u64;
pub type Price = u64;
type Quantity = u64;

#[derive(Debug, Clone,Serialize,Deserialize)]
pub struct Order {
    pub id: OrderId,
    pub price: Price,
    pub qty: Quantity,
    pub is_buy: bool,
    pub order_type:OrderType,
    pub time:String,
}

#[derive(Debug)]
pub struct OrderBook {
    pub bids: BTreeMap<Price, Vec<Order>>, 
    pub asks: BTreeMap<Price, Vec<Order>>, 
    pub symbol:Symbol,
    pub current_price:Option<Price>,
    pub last_trade_price: Option<Price>,
    pub current_best_bid: Option<Price>,
    pub current_best_ask: Option<Price>,
}

#[derive(Debug)]
pub struct OrderBookDepth {
    pub bids: Vec<(u64, u64)>, 
    pub asks: Vec<(u64, u64)>, 
}

#[derive(Debug)]
pub struct ProcessOrderResult {
    pub order_id: u64,
    pub trades: Vec<Order>,
    pub remaining_quantity: u64,
    pub orderbook_state: OrderBookState,
}
#[derive(Debug)]
pub struct OrderBookState {
    pub symbol: Symbol,
    pub current_price: Option<u64>,
    pub best_bid: Option<u64>,
    pub best_ask: Option<u64>,
    pub last_trade_price: Option<u64>,
}
