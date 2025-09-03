use actix_web::{post, web::{self, Data, Json}, HttpResponse, Responder};
use serde_json::Value;
use redis::AsyncCommands;
use futures_util::stream::StreamExt;
use uuid::Uuid;
use crate::{inputs::CreateOrderInput, output::{CreateOrderOutput, Success}};

type RedisPool = redis::Client;

#[post("/order")]
pub async fn create_order(body:Json<CreateOrderInput>,redis_client:Data<RedisPool>) ->impl Responder{

    let request_id = Uuid::new_v4().to_string();
    let response_channel = format!("order_response:{}",request_id);

    let mut order_with_id = serde_json::to_value(&body.0).unwrap();
    order_with_id["request_id"] = Value::String(request_id.clone());
    let serialized_order = order_with_id.to_string();

    let pubsub_conn = match redis_client.get_async_connection().await {
        Ok(c) =>c,
        Err(e)=>{
            eprintln!("Error Creating Pub Sub Connection : {:?}",e);
            return HttpResponse::InternalServerError().finish();
        }
    };
    let mut pubsub = pubsub_conn.into_pubsub();
    if let Err(e) = pubsub.subscribe(&response_channel).await {
        eprintln!("Failed to subscribe: {}", e);
        return HttpResponse::InternalServerError().finish();
    }

    let mut conn = match redis_client.get_async_connection().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Redis connection error: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };
    let res:redis::RedisResult<()> = conn.rpush("order", serialized_order).await;
    if let Err(e) = res{
         eprintln!("Failed to push to Queue: {}", e);
        return HttpResponse::InternalServerError().finish();
    };
    let msg = pubsub.on_message().next().await;

    match msg{
        Some(m) =>{
            let payload:redis::RedisResult<String> = m.get_payload();
            match payload {
                Ok(json)=>{
                    let v:Value = serde_json::from_str(&json).unwrap_or_default();
                    HttpResponse::Ok().json(CreateOrderOutput{
                        success:Success::True,
                        order_id:v["result_id"].as_u64().unwrap_or(0) as u32
                    })
                }
                Err(e)=>{
                    eprint!("Error Deserializing Message  :{:?}",e);
                    HttpResponse::InternalServerError().finish()
                }
            }
        }
        None => HttpResponse::InternalServerError().finish()
    }  
}
pub fn init(cfg:&mut web::ServiceConfig){
    cfg.service(create_order);
}

