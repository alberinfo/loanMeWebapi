#![allow(non_snake_case)]
#![allow(clippy::needless_return)]

use axum::{http::StatusCode, response::IntoResponse};

pub async fn pageNotFound() -> impl IntoResponse {
    return (StatusCode::NOT_FOUND, String::from("Page not found!"));
}