use std::{net::TcpListener, format};

fn spawn_app() -> String {
    let listen = TcpListener::bind("127.0.0.1:0").expect("Failed to find port.");
    let port = listen.local_addr().unwrap().port();

    let server = zero2prod::run(listen).expect("Failed to bind address");
    let _ = tokio::spawn(server);

    format!("http://127.0.0.1:{}", port)
}

#[tokio::test]
async fn health_check_works() {
    let addr = spawn_app();
    let client = reqwest::Client::new();

    let res = client
        .get(format!("{}/health-check", &addr))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(res.status().is_success());
    assert_eq!(Some(0), res.content_length())
}

#[tokio::test]
async fn subscribe_returns_a_200_form_data() {
    let addr = spawn_app();
    let client = reqwest::Client::new();

    let body = "name=Andrew%20Gristey&email=andrew.gristey%40gmail.com";
    let res = client
        .post(format!("{}/subscribe", &addr))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, res.status().as_u16());
}

#[tokio::test]
async fn subscribe_returns_a_400_form_data() {
    let addr = spawn_app();
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=andrew%20g", "Missing email."),
        ("email=andrew%40andrewg.xyz", "Missing name."),
        ("", "Missing both email & name."),
    ];

    for (invalid, err_msg) in test_cases {
        let res = client
            .post(format!("{}/subscribe", &addr))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid)
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(400, res.status().as_u16(), "{}", err_msg);
    }
}
