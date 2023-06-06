use uuid::Uuid;
use chrono::Utc;
use actix_web::{web, HttpResponse};
use sqlx::PgPool;

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String
}

pub async fn subcribe(form: web::Form<FormData>, pool: web::Data<PgPool>) -> HttpResponse {
    let uuid = Uuid::new_v4();
    let timestamp = Utc::now();

    match sqlx::query!("INSERT INTO subscriptions (id, email, name, subscribed_at) VALUES ($1, $2, $3, $4)", uuid, form.email, form.name, timestamp)
    .execute(pool.get_ref())
    .await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            println!("Failed to execute query: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    };

    HttpResponse::Ok().finish()
}
