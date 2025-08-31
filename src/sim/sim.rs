use rand::{Rng, thread_rng};
use reqwest::Client;
use std::{collections::VecDeque, thread::sleep, time::Duration};

use crate::inputs::{CreateOrderInput, Side, Symbol};
pub struct OrderSimulator {
    client: Client,
    server_url: String,
    base_price: f64,
    price_volatility: f64,
    order_frequency_ms: u64,
    min_quantity: u32,
    max_quantity: u32,
    symbol: Symbol,
    price_history: VecDeque<f64>,
    last_price: f64,
}

impl OrderSimulator {
    pub fn new(
        server_url: String,
        symbol: Symbol,
        base_price: f64,
        price_volatility: f64,
        order_frequency_ms: u64,
    ) -> Self {
        Self {
            client: Client::new(),
            server_url,
            base_price,
            symbol,
            price_volatility,
            order_frequency_ms,
            price_history: VecDeque::with_capacity(20),
            last_price: base_price,
            min_quantity: 1,
            max_quantity: 100,
        }
    }
    pub fn with_quantity_range(mut self, min: u32, max: u32) -> Self {
        self.min_quantity = min;
        self.max_quantity = max;
        self
    }
    pub async fn start_simulation(&mut self) {
        println!(
            "Starting order simulation for {:?} at base price ${:.2}",
            self.symbol, self.base_price
        );
        println!(
            "Volatility: {:.1}%, Frequency: {}ms",
            self.price_volatility * 100.0,
            self.order_frequency_ms
        );

        loop {
            let current_price = self.generate_realistic_price();

            let (side, price) = self.determine_order_side_and_price(current_price);

            let (order,side) = self.generate_order(side, price);
            match self.send_order(&order).await {
                Ok(response) => {
                    let side_icon = match side {
                        Side::Buy => "🟢",
                        Side::Sell => "🔴",
                    };
                    println!(
                        "{} {} {} {} @ ${:.2} - Response: {}",
                        side_icon,
                        chrono::Utc::now().format("%H:%M:%S"),
                        format!("{:?}", side).to_uppercase(),
                        order.quantity,
                        order.price,
                        response
                    );
                }
                Err(e) => {
                    eprintln!("❌ Failed to send order: {}", e);
                }
            }

            self.update_price_history(current_price);
            let jitter = thread_rng().gen_range(0..=200);
            sleep(Duration::from_millis(self.order_frequency_ms + jitter))
        }
    }
    fn generate_realistic_price(&mut self) -> f64 {
        let mut rng = thread_rng();
        let drift = (self.base_price - self.last_price) * 0.01;
        let volatility_adjustment = rng.gen_range(-self.price_volatility..=self.price_volatility);
        let trend_factor = self.calculate_trend_factor();

        let price_change = drift + (volatility_adjustment * self.last_price) + trend_factor;
        let new_price = self.last_price + price_change;
        self.last_price = new_price;
        new_price
    }
    fn calculate_trend_factor(&mut self) -> f64 {
        if self.price_history.len() < 5 {
            return 0.0;
        }
        let recent_prices: Vec<_> = self.price_history.iter().rev().take(5).cloned().collect();
        let trend = recent_prices.last().unwrap() - recent_prices.first().unwrap();

        trend * 0.1
    }
    fn determine_order_side_and_price(&self, current_price: f64) -> (Side, f64) {
        let mut rng = thread_rng();
        if rng.gen_bool(0.6) {
            let side = if rng.gen_bool(0.5) {
                Side::Buy
            } else {
                Side::Sell
            };
            let price_offset = rng.gen_range(-0.005..=0.005);
            let price = current_price * (1.0 + price_offset);
            (side, price)
        } else {
            if rng.gen_bool(0.5) {
                let discount = rng.gen_range(0.01..=0.05);
                (Side::Buy, current_price * (1.0 - discount))
            } else {
                let premium = rng.gen_range(0.01..=0.05);
                (Side::Sell, current_price * (1.0 + premium))
            }
        }
    }
    fn generate_order(&self, side: Side, price: f64) -> (CreateOrderInput, Side) {
        let mut rng = thread_rng();

        let quantity = if rng.gen_bool(0.7) {
            rng.gen_range(self.min_quantity..=(self.max_quantity / 4))
        } else if rng.gen_bool(0.8) {
            rng.gen_range((self.max_quantity / 4)..=(self.max_quantity / 2))
        } else {
            rng.gen_range((self.max_quantity / 2)..=self.max_quantity)
        };

        let order = CreateOrderInput {
            symbol:self.symbol.clone(),
            side,
            quantity,
            price: (price * 100.0).round() / 100.0,
            user_id: 1,
        };

        (order, side) 
    }
    async fn send_order(
        &self,
        order: &CreateOrderInput,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let response = self
            .client
            .post(&format!("{}/order", self.server_url))
            .json(order)
            .send()
            .await?;

        if response.status().is_success() {
            let body = response.text().await?;
            Ok(body)
        } else {
            Err(format!("HTTP {}: {}", response.status(), response.text().await?).into())
        }
    }
    fn update_price_history(&mut self, price: f64) {
        if self.price_history.len() >= 20 {
            self.price_history.pop_front();
        }
        self.price_history.push_back(price);
    }
}
