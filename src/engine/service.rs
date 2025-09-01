use chrono::Utc;
use serde_json::Value;

use crate::{global::{NEXT_ORDER_ID, ORDERBOOKS}, inputs::{CreateOrderInput, Order, OrderBook, OrderBookState, OrderType, ProcessOrderResult, Side}};

pub fn add_order(orderbook: &mut OrderBook, order: Order) {
    let book: &mut std::collections::BTreeMap<u64, Vec<Order>> = if order.is_buy {
        &mut orderbook.bids
    } else {
        &mut orderbook.asks
    };
    book.entry(order.price).or_insert_with(Vec::new).push(order);
    update_best_prices(orderbook);
}

pub fn match_order(orderbook: &mut OrderBook, order: &Order, order_type: OrderType) -> Vec<Order> {
    let mut trades = Vec::new();
    let mut qty_left = order.qty;

    if order.is_buy {
        let mut prices: Vec<_> = orderbook.asks.keys().cloned().collect();
        prices.sort();
        for price in prices {
            if order_type == OrderType::Limit && price > order.price {
                break;
            }
            let order = orderbook.asks.get_mut(&price).unwrap();
            for resting in order.iter_mut() {
                if qty_left == 0 {
                    break;
                }
                let trade_qty = qty_left.min(resting.qty);
                trades.push(Order {
                    id: resting.id,
                    price,
                    qty: trade_qty,
                    is_buy: false,
                    time: resting.time.to_string(),
                    order_type: OrderType::Limit,
                });
                resting.qty -= trade_qty;
                qty_left -= trade_qty;
                orderbook.last_trade_price = Some(price);
                orderbook.current_price = Some(price);
            }
            order.retain(|o| o.qty > 0);
            if qty_left == 0 {
                break;
            }
            if order.is_empty() {
                orderbook.asks.remove(&price);
            }
        }
    } else {
        let mut prices: Vec<_> = orderbook.bids.keys().cloned().collect();
        prices.sort_by(|a, b| b.cmp(a));

        for price in prices {
            if order_type == OrderType::Limit && price < order.price {
                break;
            }
            let orders = orderbook.bids.get_mut(&price).unwrap();
            for resting in orders.iter_mut() {
                if qty_left == 0 {
                    break;
                }
                let trade_qty = qty_left.min(resting.qty);
                trades.push(Order {
                    id: resting.id,
                    price,
                    qty: trade_qty,
                    is_buy: true,
                    time: resting.time.to_string(),
                    order_type: OrderType::Limit,
                });
                resting.qty -= trade_qty;
                qty_left -= trade_qty;
                orderbook.last_trade_price = Some(price);
                orderbook.current_price = Some(price);
            }
            orders.retain(|o| o.qty > 0);
            if qty_left == 0 {
                break;
            }
            if orders.is_empty() {
                orderbook.bids.remove(&price);
            }
        }
    }
    if qty_left > 0 && order_type == OrderType::Limit {
        let mut reminder = order.clone();
        reminder.qty = qty_left;
        orderbook.add_order(reminder);
    }
    update_best_prices(orderbook);
    trades
}

fn update_best_prices(orderbook: &mut OrderBook) {
    orderbook.current_best_bid = orderbook.bids.keys().max().copied();
    orderbook.current_best_ask = orderbook.asks.keys().min().copied();
}

pub fn process_order(order_data: &Value) -> Result<ProcessOrderResult, Box<dyn std::error::Error>> {
    let order_input: CreateOrderInput = serde_json::from_value(order_data.clone())?;

    let order_id = {
        let mut id = NEXT_ORDER_ID.lock().unwrap();
        let current_id = *id;
        *id += 1;
        current_id
    };
    let price_int = (order_input.price * 100.0) as u64;

    let order = Order {
        id: order_id,
        price: price_int,
        qty: order_input.quantity as u64,
        is_buy: order_input.side == Side::Buy,
        order_type: order_input.order_type,
        time: Utc::now().to_string(),
    };
    let mut orderbooks = ORDERBOOKS.lock().unwrap();
    let orderbook = orderbooks
        .get_mut(&order_input.symbol)
        .ok_or("Invalid Symbol")?;

    let trades = if order_input.order_type == OrderType::Market {
        let trades = orderbook.match_order(&order, OrderType::Market);
        trades
    } else {
        let trades = orderbook.match_order(&order, OrderType::Limit);
        trades
    };
    let filled_quantity: u64 = trades.iter().map(|t| t.qty).sum();
    let remaining_quantity = order.qty.saturating_sub(filled_quantity);

    let orderbook_state = OrderBookState {
        symbol: orderbook.symbol.clone(),
        current_price: orderbook.current_price,
        best_bid: orderbook.current_best_bid,
        best_ask: orderbook.current_best_ask,
        last_trade_price: orderbook.last_trade_price,
    };
    Ok(ProcessOrderResult {
        order_id,
        trades,
        remaining_quantity,
        orderbook_state,
    })
}
