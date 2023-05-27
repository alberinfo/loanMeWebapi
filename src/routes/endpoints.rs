#![allow(non_snake_case)]
#![allow(clippy::needless_return)]

use axum::{http::{StatusCode, Request}, response::{IntoResponse, Response}, Json, extract::State};
use crate::{services::{db, redisServer::insertUserSession}, models::{usuario::Usuario, session}};

pub async fn pageNotFound() -> impl IntoResponse {
    return (StatusCode::NOT_FOUND, "Page not found!");
}

pub async fn registrarUsuario(State((dbPool, _redisPool)): State<(sqlx::PgPool, redis::aio::ConnectionManager)>, Json(mut payload): Json<Usuario>) -> impl IntoResponse {
    payload.hashcontrasenna = payload.generatePwd().await;
    let res = db::insertarUsuario(payload, &dbPool).await;

    return match res {
        Ok(r) => match r.rows_affected() {
            0 => Err((StatusCode::BAD_REQUEST, "There was an error while creating the user".to_string())),
            1 => Ok("Done".to_string()),
            _ => Err((StatusCode::INTERNAL_SERVER_ERROR, "This should not have happened.".to_string()))
        },

        Err(r) => Err((StatusCode::INTERNAL_SERVER_ERROR, r.to_string()))
    };
}

pub async fn loginUsuario(State((dbPool, mut redisPool)): State<(sqlx::PgPool, redis::aio::ConnectionManager)>, Json(payload): Json<Usuario>) -> impl IntoResponse {
    let usuario = db::buscarUsuario(&payload.nombreusuario, &dbPool).await;

    if usuario.is_err() {
        match usuario.unwrap_err() {
            //no se encontro el usuario
            sqlx::Error::RowNotFound => return Err((StatusCode::BAD_REQUEST, "User does not exist".to_string())),
            x => return Err((StatusCode::INTERNAL_SERVER_ERROR, x.to_string()))
        }
    }

    //usuario.hashContrasenna currently contains the PHC
    let validPwd = payload.validatePwd(usuario.unwrap().hashcontrasenna).await;

    if !validPwd {
        return Err((StatusCode::UNAUTHORIZED, "Wrong password".to_string()));
    }


    let nuevaSession = session::Session::new().await;
    let res = insertUserSession(&nuevaSession, &mut redisPool).await;
    return match res {
        Ok(_) => Ok(nuevaSession.id),
        Err(err) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{:?}: {}", err.kind(), err.detail().unwrap())))
    };
}