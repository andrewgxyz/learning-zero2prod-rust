use zero2prod::configuration::get_config;
use zero2prod::telemetry::{get_subscriber, init_subscriber};
use zero2prod::startup::run;
use std::net::TcpListener;
use sqlx::PgPool;


#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let subscriber = get_subscriber(
        "zero2prod".into(),
        "info".into()
    );
    init_subscriber(subscriber);

    let config = get_config().expect("Failed to read config");
    let con_pool = PgPool::connect(&config.database.connection_string())
        .await
        .expect("Failed to connect");

    let addr = format!("127.0.0.1:{}", config.application_port);
    let listen = TcpListener::bind(addr).expect("Failed to find port.");
    run(listen, con_pool)?.await?;
    Ok(())
}
