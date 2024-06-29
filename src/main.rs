mod bot;
mod commands;
mod data;
mod util;

use dotenvy::dotenv;

#[tokio::main]
async fn main() {
    dotenv().expect(".env file not found");
    env_logger::init();
    bot::start().await;
}
