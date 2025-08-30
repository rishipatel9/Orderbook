use actix_web::{self, get, web, App, HttpServer, Responder};
use redis::Client;

pub mod router;
pub mod inputs;
pub mod output;
pub mod config;

use config::APP_CONFIG;
#[actix_web::main]

async fn main() -> Result<(),std::io::Error>{
     let addrs = &APP_CONFIG.server_addr;
    let redis_url = &APP_CONFIG.redis_url;

    println!("Server is listening on http://{addrs}");
    let redis_client = Client::open(redis_url.clone()).expect("Something went wrong with redis");
    HttpServer::new(move || {
        App::new()
        .service(base)
        .app_data(web::Data::new(redis_client.clone()))
        .configure(router::init)
    })
    .bind(addrs)
    ?.run()
    .await
    
}
#[get("/")]
async fn base() ->impl Responder{
    return "Hello world";
}