use serde::{Deserialize, Serialize};

#[derive(Deserialize,Serialize)]
pub struct CreateOrderOutput{
    pub success:Success,
    pub order_id:u32
}
#[derive(Deserialize,Serialize)]
pub enum Success{
    True,False
}