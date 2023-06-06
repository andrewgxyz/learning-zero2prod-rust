use std::{net::TcpListener, format};
use sqlx::PgPool;
use zero2prod::configuration::get_config;

pub struct TestApp {
    pub addr: String,
    pub db_pool: PgPool
}

async fn spawn_app() -> TestApp {
    let listen = TcpListener::bind("127.0.0.1:0").expect("Failed to find port.");
    let port = listen.local_addr().unwrap().port();
    let addr = format!("http://127.0.0.1:{}", port);

    let config = get_config().expect("Failed to read configuration");
    let db_pool = PgPool::connect(&config.database.connection_string())
        .await
        .expect("Failed to connect to Postgres");

    let server = zero2prod::startup::run(listen, db_pool.clone()).expect("Failed to bind address");
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
async fn subscribe_returns_a_200_form_data() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let body = "name=Andrew%20Gristey&email=andrew.gristey%40gmail.com";
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

    assert_eq!(saved.name, "Andrew Gristey");
    assert_eq!(saved.email, "andrew.gristey@gmail.com");
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
