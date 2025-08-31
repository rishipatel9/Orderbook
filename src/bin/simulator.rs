
use std::env;

use orderbook::sim::runner::{run_multi_symbol_simulation, SimulatorConfig};


#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    
    let server_url = args.get(1)
        .unwrap_or(&"http://127.0.0.1:8080".to_string())
        .clone();

    println!("🎯 Order Book Simulator");
    println!("📡 Server: {}", server_url);
    println!("⚡ Starting multi-symbol simulation...\n");

    let config = SimulatorConfig::default();
    run_multi_symbol_simulation(server_url, config).await;
}