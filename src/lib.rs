use actix_web::{web, App, HttpServer, HttpResponse};
use actix_web::dev::Server;
use std::net::TcpListener;

#[derive(serde::Deserialize)]
struct FormData {
    email: String,
    name: String
}

async fn health_check() -> HttpResponse {
    HttpResponse::Ok().finish()
}

async fn subcribe(_form: web::Form<FormData>) -> HttpResponse {
    HttpResponse::Ok().finish()
}

pub fn run(listen: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| {
        App::new()
            .route("/health-check", web::get().to(health_check))
            .route("/subscribe", web::post().to(subcribe))
    })
    .listen(listen)?
    .run();

    Ok(server)
}
