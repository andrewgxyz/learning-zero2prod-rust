use std::{net::TcpListener, format};
use sqlx::{PgPool, PgConnection, Connection, Executor};
use zero2prod::configuration::{get_config, DatabaseSettings};
use zero2prod::startup::run;
use zero2prod::telemetry::{get_subscriber, init_subscriber};
use once_cell::sync::Lazy;

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "into".to_string();
    let sub_name = "test".to_string();

    if std::env::var("TEST_LOG").is_ok() {
        let sub = get_subscriber(sub_name, default_filter_level, std::io::stdout);
        init_subscriber(sub);
    } else {
        let sub = get_subscriber(sub_name, default_filter_level, std::io::sink);
        init_subscriber(sub);
    }
});

pub struct TestApp {
    pub addr: String,
    pub db_pool: PgPool
}

pub async fn config_db(config: &DatabaseSettings) -> PgPool {
    let mut db = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to Postgres");

    db.execute(format!("CREATE DATABASE {};", config.database_name).as_str()).await.expect("Failed to create database");

    let db_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to Postgres");

    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .expect("Failed to migrate the database");

    db_pool
}

async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let listen = TcpListener::bind("127.0.0.1:0").expect("Failed to find port.");
    let port = listen.local_addr().unwrap().port();
    let addr = format!("http://127.0.0.1:{}", port);

    let config = get_config().expect("Failed to read configuration");

    let db_pool = PgPool::connect_with(config.database.with_db())
        .await
        .expect("Failed to connect to Postgres");

    let server = run(listen, db_pool.clone()).expect("Failed to bind address");
    let _ = tokio::spawn(server);

    TestApp {
        addr,
        db_pool
    }
}

#[tokio::test]
async fn health_check_works() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let res = client
        .get(format!("{}/health-check", &app.addr))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(res.status().is_success());
    assert_eq!(Some(0), res.content_length())
}

#[tokio::test]
async fn subscribe_returns_a_400_when_fields_are_present_but_empty() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=&email=ursela_le_guin%40gmail.com", "empty name"),
        ("name=Andrew&email=", "empty email"),
        ("name=andrew&email=ursela_le_guin", "invalid email"),
    ];

    for (body, description) in test_cases {
        let res = client
            .post(format!("{}/subscribe", &app.addr))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(400, res.status().as_u16(), "The API did not return a 400 OK when the payloa was {}", description);
    }
}

#[tokio::test]
async fn subscribe_returns_a_200_form_data() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let body = "name=Andrew%20G&email=andrew.g%40gmail.com";
    let res = client
        .post(format!("{}/subscribe", &app.addr))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, res.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.name, "Andrew G");
    assert_eq!(saved.email, "andrew.g@gmail.com");
}

#[tokio::test]
async fn subscribe_returns_a_400_form_data() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=andrew%20g", "Missing email."),
        ("email=andrew%40andrewg.xyz", "Missing name."),
        ("", "Missing both email & name."),
    ];

    for (invalid, err_msg) in test_cases {
        let res = client
            .post(format!("{}/subscribe", &app.addr))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid)
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(400, res.status().as_u16(), "{}", err_msg);
    }
}
