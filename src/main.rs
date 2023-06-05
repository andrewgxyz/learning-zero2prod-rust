use std::net::TcpListener;

use zero2prod::run;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let listen = TcpListener::bind("127.0.0.1:8000").expect("Failed to find port.");
    run(listen)?.await
}
