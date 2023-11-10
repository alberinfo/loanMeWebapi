#![allow(non_snake_case)]
#![allow(clippy::needless_return)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

use axum::{http::StatusCode, response::IntoResponse};

pub async fn pageNotFound() -> impl IntoResponse {
    return (StatusCode::NOT_FOUND, String::from("Page not found!"));
}