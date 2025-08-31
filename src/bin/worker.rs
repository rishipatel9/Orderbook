use rand::{rngs::ThreadRng, Rng};
use redis::{AsyncCommands, Client};
use serde_json::Value;
use tokio::time::{sleep, Duration};
use serde::{Serialize, Deserialize};
use orderbook::{config::APP_CONFIG, engine::engine::process_order};

#[derive(Debug, Serialize, Deserialize)]
struct OrderResponse {
    order: String,   
    result_id: u32,  
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let redis_url = &APP_CONFIG.redis_url;
    let redis_client = Client::open(redis_url.clone())?;
    let mut conn = redis_client.get_async_connection().await?;
    let mut rng: ThreadRng = rand::thread_rng();
    
    loop {
        let result: redis::RedisResult<Vec<String>> = conn.blpop("order", 0).await;
        match result {
            Ok(items) => {
                 if items.len() >= 2 {
                    let order_data = &items[1];
                    
                    let order_json: Value = serde_json::from_str(order_data)?;
                    process_order(&order_json);
                    let request_id = order_json["request_id"].as_str()
                        .unwrap_or("unknown");
                    
                    let n: u32 = rng.gen_range(1..=100);
                    let response = OrderResponse {
                        order: order_data.clone(),
                        result_id: n,
                    };

                    let json = serde_json::to_string(&response)?;
                    
                    let response_channel = format!("order_response:{}", request_id);
                    let _: () = conn.publish(&response_channel, json).await?;
                }
            } 
            Err(e) => {
                eprintln!("Error reading from Redis queue: {}", e);
                sleep(Duration::from_secs(5)).await;
            }
        }
    }
}

