use std::env;
use orderbook::{global::ORDERBOOKS, inputs::Symbol};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
use redis::Client;
use serde_json::json;
use chrono::Utc;


async fn handle_connection(stream: TcpStream) {
    println!("New WebSocket connection");
    
    let ws_stream = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            eprintln!("WebSocket connection error: {}", e);
            return;
        }
    };

    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    let mut subscriptions: Vec<Symbol> = Vec::new();
    
    let redis_client = match Client::open("redis://127.0.0.1:6379/".to_string()) {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Redis connection error: {}", e);
            return;
        }
    };

    let pubsub_conn = match redis_client.get_async_connection().await {
        Ok(conn) => conn,
        Err(e) => {
            eprintln!("Redis pubsub connection error: {}", e);
            return;
        }
    };

    let mut pubsub = pubsub_conn.into_pubsub();
    if let Err(e) = pubsub.subscribe("market_updates").await {
        eprintln!("Failed to subscribe to market updates: {}", e);
        return;
    }

    let welcome_msg = json!({
        "type": "welcome",
        "message": "Connected to OrderBook WebSocket",
        "available_symbols": ["BTCUSD", "ETHUSD", "SOLUSD"]
    });
    
    if let Err(e) = ws_sender.send(Message::Text(welcome_msg.to_string())).await {
        eprintln!("Failed to send welcome message: {}", e);
        return;
    }

    let mut on_message = pubsub.on_message();
    loop {
        tokio::select! {
            // Handle incoming WebSocket messages
            msg = ws_receiver.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        if let Err(e) = handle_client_message(&text, &mut ws_sender, &mut subscriptions).await {
                            eprintln!("Error handling client message: {}", e);
                            break;
                        }
                    }
                    Some(Ok(Message::Close(_))) => {
                        println!("WebSocket connection closed by client");
                        break;
                    }
                    Some(Err(e)) => {
                        eprintln!("WebSocket error: {}", e);
                        break;
                    }
                    None => break,
                    _ => {} 
                }
            }
            
            update = on_message.next() => {
                if let Some(msg) = update {
                    println!("{:?}",msg);
                    if let Ok(payload) = msg.get_payload::<String>() {
                        if let Ok(update_data) = serde_json::from_str::<serde_json::Value>(&payload) {
                            if let Some(symbol_str) = update_data["symbol"].as_str() {
                                let symbol = match symbol_str {
                                    "BTCUSD" => Symbol::BTCUSD,
                                    "ETHUSD" => Symbol::ETHUSD, 
                                    "SOLUSD" => Symbol::SOLUSD,
                                    _ => continue,
                                };
                                
                                if subscriptions.contains(&symbol) {
                                    if let Ok(orderbook_data) = get_orderbook_with_trades(&symbol, &update_data) {
                                        let market_update = json!({
                                            "type": "orderbook_update",
                                            "symbol": symbol_str,
                                            "data": orderbook_data
                                        });
                                        
                                        if let Err(e) = ws_sender.send(Message::Text(market_update.to_string())).await {
                                            eprintln!("Failed to send market update: {}", e);
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    println!("WebSocket connection ended");
}

async fn handle_client_message(
    text: &str, 
    ws_sender: &mut futures_util::stream::SplitSink<tokio_tungstenite::WebSocketStream<TcpStream>, Message>,
    subscriptions: &mut Vec<Symbol>
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let msg: serde_json::Value = serde_json::from_str(text)?;
    
    match msg["type"].as_str() {
        Some("subscribe") => {
            let symbol_str = msg["symbol"].as_str().unwrap_or("BTCUSD");
            let symbol = match symbol_str {
                "BTCUSD" => Symbol::BTCUSD,
                "ETHUSD" => Symbol::ETHUSD,
                "SOLUSD" => Symbol::SOLUSD,
                _ => {
                    let error = json!({
                        "type": "error",
                        "message": "Invalid symbol. Available: BTCUSD, ETHUSD, SOLUSD"
                    });
                    ws_sender.send(Message::Text(error.to_string())).await?;
                    return Ok(());
                }
            };

            if !subscriptions.contains(&symbol) {
                subscriptions.push(symbol.clone());
            }
            
            let orderbook_snapshot = get_full_orderbook_snapshot(&symbol)?;
            let response = json!({
                "type": "subscription_confirmed",
                "symbol": symbol_str,
                "message": format!("Subscribed to {} orderbook", symbol_str),
                "orderbook": orderbook_snapshot
            });
            ws_sender.send(Message::Text(response.to_string())).await?;
        }
        Some("unsubscribe") => {
            let symbol_str = msg["symbol"].as_str().unwrap_or("BTCUSD");
            let symbol = match symbol_str {
                "BTCUSD" => Symbol::BTCUSD,
                "ETHUSD" => Symbol::ETHUSD,
                "SOLUSD" => Symbol::SOLUSD,
                _ => return Ok(()),
            };
            
            subscriptions.retain(|s| s != &symbol);
            
            let response = json!({
                "type": "unsubscribe_confirmed",
                "symbol": symbol_str,
                "message": format!("Unsubscribed from {}", symbol_str)
            });
            ws_sender.send(Message::Text(response.to_string())).await?;
        }
        Some("get_orderbook") => {
            let symbol_str = msg["symbol"].as_str().unwrap_or("BTCUSD");
            let symbol = match symbol_str {
                "BTCUSD" => Symbol::BTCUSD,
                "ETHUSD" => Symbol::ETHUSD,
                "SOLUSD" => Symbol::SOLUSD,
                _ => {
                    let error = json!({
                        "type": "error",
                        "message": "Invalid symbol"
                    });
                    ws_sender.send(Message::Text(error.to_string())).await?;
                    return Ok(());
                }
            };
            
            let orderbook_snapshot = get_full_orderbook_snapshot(&symbol)?;
            let response = json!({
                "type": "orderbook_snapshot",
                "symbol": symbol_str,
                "orderbook": orderbook_snapshot
            });
            ws_sender.send(Message::Text(response.to_string())).await?;
        }
        Some("price") =>{
            let symbol_str= msg["symbol"].as_str().unwrap_or("BTCUSD");
             let symbol = match symbol_str {
                "BTCUSD" => Symbol::BTCUSD,
                "ETHUSD" => Symbol::ETHUSD,
                "SOLUSD" => Symbol::SOLUSD,
                _ => {
                    let error = json!({
                        "type": "error",
                        "message": "Invalid symbol"
                    });
                    ws_sender.send(Message::Text(error.to_string())).await?;
                    return Ok(());
                }
            };
            ws_sender.send(Message::Binary("BTCUSD".into()))   .await?;
        }
        Some("ping") => {
            let pong = json!({
                "type": "pong",
                "timestamp": Utc::now().timestamp()
            });
            ws_sender.send(Message::Text(pong.to_string())).await?;
        }
        _ => {
            let error = json!({
                "type": "error",
                "message": "Unknown message type. Available: subscribe, unsubscribe, get_orderbook, ping"
            });
            ws_sender.send(Message::Text(error.to_string())).await?;
        }
    }
    
    Ok(())
}

fn get_full_orderbook_snapshot(symbol: &Symbol) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
    let orderbooks = ORDERBOOKS.lock().unwrap();
    let orderbook = orderbooks.get(symbol).ok_or("Symbol not found")?;
    
    let depth = orderbook.get_depth(10);
    
    let orderbook_data = json!({
        "symbol": format!("{:?}", symbol),
        "current_price": orderbook.current_price.map(|p| p as f64 / 100.0),
        "last_trade_price": orderbook.last_trade_price.map(|p| p as f64 / 100.0),
        "best_bid": orderbook.current_best_bid.map(|p| p as f64 / 100.0),
        "best_ask": orderbook.current_best_ask.map(|p| p as f64 / 100.0),
        "bids": depth.bids.iter().map(|(price, qty)| json!({
            "price": *price as f64 / 100.0,
            "quantity": qty,
            "total": (*price as f64 / 100.0) * (*qty as f64)
        })).collect::<Vec<_>>(),
        "asks": depth.asks.iter().map(|(price, qty)| json!({
            "price": *price as f64 / 100.0,
            "quantity": qty,
            "total": (*price as f64 / 100.0) * (*qty as f64)
        })).collect::<Vec<_>>(),
        "timestamp": Utc::now().timestamp()
    });
    
    Ok(orderbook_data)
}

fn get_orderbook_with_trades(symbol: &Symbol, trade_data: &serde_json::Value) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
    let orderbook_snapshot = get_full_orderbook_snapshot(symbol)?;
    
    let combined_data = json!({
        "orderbook": orderbook_snapshot,
        "recent_trades": trade_data["trades"],
        "trade_summary": {
            "trades_count": trade_data["trades"].as_array().map(|t| t.len()).unwrap_or(0),
            "last_price": trade_data["current_price"]
        }
    });
    
    Ok(combined_data)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    let addr = env::args().nth(1).unwrap_or_else(|| "127.0.0.1:4000".to_string());
    let addr: std::net::SocketAddr = addr.parse().expect("Invalid address");
    
    println!("WebSocket server starting on: {}", addr);
    let listener = TcpListener::bind(&addr).await.expect("Failed to bind");
    
    println!("WebSocket server listening on ws://{}", addr);
    println!("Available commands:");
    println!("  - subscribe: {{\"type\": \"subscribe\", \"symbol\": \"BTCUSD\"}}");
    println!("  - unsubscribe: {{\"type\": \"unsubscribe\", \"symbol\": \"BTCUSD\"}}");
    println!("  - get_orderbook: {{\"type\": \"get_orderbook\", \"symbol\": \"BTCUSD\"}}");
    println!("  - ping: {{\"type\": \"ping\"}}");
    
    while let Ok((stream, addr)) = listener.accept().await {
        println!("New connection from: {}", addr);
        tokio::spawn(handle_connection(stream));
    }
    
    Ok(())
}