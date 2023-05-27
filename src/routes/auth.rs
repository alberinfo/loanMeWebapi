#![allow(non_snake_case)]
#![allow(clippy::needless_return)]

use axum::{http::{StatusCode, Request}, response::{IntoResponse, Response}, Json, extract::State, middleware::Next};

use crate::services::redisServer;

pub async fn validationLayer(State((_dbPool, mut redisPool)): State<(sqlx::PgPool, redis::aio::ConnectionManager)>, req: Request<axum::body::Body>, next: Next<axum::body::Body>) -> Response {
    let current_path = &req.uri().path().to_string();

    let skip_paths = vec!["/registrarUsuario", "/loginUsuario"]; //Añadir caminos a medida que sea necesario.
    for skip_path in skip_paths {
        if current_path.ends_with(skip_path) {
            return next.run(req).await;
        }
    }

    let auth_header = req.headers().get(axum::http::header::AUTHORIZATION).and_then(|header| header.to_str().ok());
    if auth_header.is_none() {
        return StatusCode::UNAUTHORIZED.into_response();
    } else if !(redisServer::verifyUserSession(&auth_header.unwrap().to_string(), &mut redisPool).await) {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    return next.run(req).await;
}