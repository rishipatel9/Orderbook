use crate::{inputs::Symbol, sim::sim::OrderSimulator};

pub struct SimulatorConfig {
    pub symbols: Vec<Symbol>,
    pub base_prices: Vec<f64>,
    pub volatilities: Vec<f64>,
    pub frequencies: Vec<u64>,
}

impl Default for SimulatorConfig {
    fn default() -> Self {
        Self {
            symbols: vec![
                Symbol::BTCUSD,
            ],
            base_prices: vec![100000.0],
            volatilities: vec![0.0003],
            frequencies: vec![20],
        }
    }
}
// impl Default for SimulatorConfig {
//     fn default() -> Self {
//         Self {
//             symbols: vec![
//                 Symbol::BTCUSD,
//                 Symbol::ETHUSD,
//                 Symbol::SOLUSD,
//             ],
//             base_prices: vec![45000.0, 2500.0, 100.0],
//             volatilities: vec![0.02, 0.025, 0.03],
//             frequencies: vec![200, 300, 400],
//         }
//     }
// }

pub async fn run_multi_symbol_simulation(server_url: String, config: SimulatorConfig) {
    let mut handles = vec![];

    for (i, symbol) in config.symbols.iter().enumerate() {
        let mut simulator = OrderSimulator::new(
            server_url.clone(),
            symbol.clone(),
            config.base_prices[i],
            config.volatilities[i],
            config.frequencies[i],
        )
        .with_quantity_range(1, 50);

        let handle = tokio::spawn(async move {
            simulator.start_simulation().await;
        });

        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.await;
    }
}
