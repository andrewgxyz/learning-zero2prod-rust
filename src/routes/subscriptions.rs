use unicode_segmentation::UnicodeSegmentation;
use uuid::Uuid;
use chrono::Utc;
use actix_web::{web, HttpResponse};
use sqlx::PgPool;

use crate::domain::{SubscriberName, NewSubscriber, SubscriberEmail};

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    fn try_from(value: FormData) -> Result<Self, Self::Error> {
        Ok(Self {
            email: SubscriberEmail::parse(value.email)?,
            name: SubscriberName::parse(value.name)?
        })
    }
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, pool),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name,
    )
)]
pub async fn subcribe(form: web::Form<FormData>, pool: web::Data<PgPool>) -> HttpResponse {
    let new_sub = match form.0.try_into() {
        Ok(form) => form,
        Err(_) => return HttpResponse::BadRequest().finish()
    };

    match insert_subscriber(&new_sub, &pool).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            tracing::error!("Failed to execute query: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub fn is_valid_name(s: &str) -> bool {
    let is_empty_or_whatespace = s.trim().is_empty();
    let is_too_long = s.graphemes(true).count() > 256;
    let forbidden_chars = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
    let contains_forbidden_chars = s.chars().any(|g| forbidden_chars.contains(&g));

    !(is_empty_or_whatespace || is_too_long || contains_forbidden_chars)
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(new_sub, pool),
)]
pub async fn insert_subscriber(
    new_sub: &NewSubscriber, 
    pool: &PgPool
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO subscriptions (id, email, name, subscribed_at) VALUES ($1, $2, $3, $4)", 
        Uuid::new_v4(), new_sub.email.as_ref(), new_sub.name.as_ref(), Utc::now())
        .execute(pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute query: {:?}", e);
            e
        })?;

    Ok(())
}
