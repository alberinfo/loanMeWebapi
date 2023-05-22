#![allow(non_snake_case)]

use axum::{http::{StatusCode, Request}, response::{IntoResponse, Response}, Json, extract::State, middleware::Next, Error};
//use crate::{db, logic::{self, generatePwdPHC}};
use crate::db::redisServer;

//Bad name. Checks whether endpoint needs for the user to be logged in, and if so then checks whether or not the user _is_ logged in.
pub async fn validateCredentialsLayer(State((dbPool, redisPool)): State<(sqlx::PgPool, redis::aio::ConnectionManager)>, req: Request<axum::body::Body>, next: Next<axum::body::Body>) -> Response {
    //Camino actual
    let path = &req.uri().path().to_string();
    
    //Endpoints que no requieren validar al usuario.
    let skip_paths = vec!["/registrarUsuario", "/loginUsuario"]; //Añadir caminos a medida que sea necesario.
    for skip_path in skip_paths {
        if path.ends_with(skip_path) {
            return next.run(req).await;
        }
    }

    
    return next.run(req).await;
}