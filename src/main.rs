mod server;
mod client;
mod error;

use std::env::args;

#[tokio::main]
async fn main() {
    if args().nth(1).unwrap().as_str() == "server" {
        server::Server::spawn("test_id", 50000).await;
    } else {
        let ip = client::discover("test_id", 50000).await.unwrap();
        println!("{ip:?}");
    }
}
