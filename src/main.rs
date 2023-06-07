use zero2prod::configuration::get_config;
use zero2prod::telemetry::{get_subscriber, init_subscriber};
use zero2prod::startup::run;
use std::net::TcpListener;
use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let subscriber = get_subscriber(
        "zero2prod".into(),
        "info".into(),
        std::io::stdout
    );
    init_subscriber(subscriber);

    let config = get_config().expect("Failed to read config");
    let con_pool = PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(config.database.with_db());

    let addr = format!("{}:{}", config.application.host, config.application.port);
    let listen = TcpListener::bind(addr).expect("Failed to find port.");
    run(listen, con_pool)?.await?;
    Ok(())
}
