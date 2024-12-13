use crate::config::AppConfig;
use dotenvy::dotenv;

mod config;
mod homeassistant;
mod mailpit;
mod sensor;
mod web;

#[tokio::main]
async fn main() {
    dotenv().expect(".env file not found");
    let config = AppConfig::from_env();

    web::run(config).await;
}
