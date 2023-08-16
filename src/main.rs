use dotenvy::dotenv;

use treatviewers_backend::start_web_server;

#[tokio::main]
async fn main() {
    // import .env file
    dotenv().ok();

    // start the web server
    start_web_server().await;
}
