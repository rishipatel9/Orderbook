use serde::{Deserialize, Serialize};

#[derive(Deserialize,Serialize,Debug)]
pub struct CreateOrderInput{
    pub price:f64,
    pub quantity:u32,
    pub user_id:u32,
    pub side:Side
}

#[derive(Deserialize,Serialize,Debug)]
pub enum Side{
    Buy,Sell
}