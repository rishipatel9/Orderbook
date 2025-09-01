use orderbook::{config::APP_CONFIG, engine::service::process_order};
use rand::{ rngs::ThreadRng};
use redis::{AsyncCommands, Client};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::time::{Duration, sleep};

#[derive(Debug, Serialize, Deserialize)]
struct OrderResponse {
    result_id: u64,
    trades: Vec<serde_json::Value>,
    remaining_quantity: u64,
    current_price: Option<f64>,
    best_bid: Option<f64>,
    best_ask: Option<f64>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let redis_url = &APP_CONFIG.redis_url;
    let redis_client = Client::open(redis_url.clone())?;
    let mut conn = redis_client.get_async_connection().await?;
    let mut _rng: ThreadRng = rand::thread_rng();

    loop {
        let result: redis::RedisResult<Vec<String>> = conn.blpop("order", 0).await;
        match result {
            Ok(items) => {
                if items.len() >= 2 {
                    let order_data = &items[1];

                    let order_json: Value = serde_json::from_str(order_data)?;
                    let request_id = order_json["request_id"].as_str().unwrap_or("unknown");

                    match process_order(&order_json) {
                        Ok(result) => {
                            // let n: u32 = rng.gen_range(1..=100);
                            let response = OrderResponse {
                                result_id: result.order_id,
                                trades: result.trades.iter()
                                    .map(|t| serde_json::to_value(t).unwrap())
                                    .collect(),
                                remaining_quantity: result.remaining_quantity,
                                current_price: result.orderbook_state.current_price
                                    .map(|p| p as f64 / 100.0),
                                best_bid: result.orderbook_state.best_bid
                                    .map(|p| p as f64 / 100.0),
                                best_ask: result.orderbook_state.best_ask
                                    .map(|p| p as f64 / 100.0),
                            };
                            println!("{:?}",response);

                            let json = serde_json::to_string(&response)?;

                            let response_channel = format!("order_response:{}", request_id);
                            let _: () = conn.publish(&response_channel, json).await?;
                        }
                        Err(e)=>{
                            eprintln!("Error processing order :{}",e);
                            let error_response = serde_json::json!({
                                "error": e.to_string(),
                                "result_id": 0
                            });
                            let response_channel = format!("order_response:{}", request_id);
                            let _: () = conn.publish(&response_channel, error_response.to_string()).await?;
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Error reading from Redis queue: {}", e);
                sleep(Duration::from_secs(5)).await;
            }
        }
    }
}
