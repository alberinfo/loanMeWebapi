#![allow(non_snake_case)]

use axum::{http::{StatusCode, Request}, response::{IntoResponse, Response}, Json, extract::State, middleware::Next, Error};
//use crate::{db, logic::{self, generatePwdPHC}};
use crate::{db::db, models::{usuario::{Usuario, TipoUsuario}, session}};
use argon2::password_hash::rand_core::OsRng;

pub async fn pageNotFound() -> impl IntoResponse {
    return (StatusCode::NOT_FOUND, "Page not found!");
}

pub async fn registrarUsuario(State((dbPool, redisPool)): State<(sqlx::PgPool, redis::aio::ConnectionManager)>, Json(mut payload): Json<Usuario>) -> Result<String, (StatusCode, String)> {
    payload.hashcontrasenna = payload.generatePwd().await;
    let res = db::insertarUsuario(payload, &dbPool).await;
    
    let nuevaSession = session::session::new(&mut OsRng);
    
    return match res {
        Ok(r) => match r.rows_affected() {
            0 => Err((StatusCode::BAD_REQUEST, "There was an error while creating the user".to_string())),
            1 => Ok("Done".to_string()),
            _ => Err((StatusCode::INTERNAL_SERVER_ERROR, "This should not have happened.".to_string()))
        },
        Err(r) => Err((StatusCode::INTERNAL_SERVER_ERROR, r.to_string()))
    };
}

//TODO: Return session id
pub async fn loginUsuario(State((dbPool, redisPool)): State<(sqlx::PgPool, redis::aio::ConnectionManager)>, Json(mut payload): Json<Usuario>) -> Result<String, (StatusCode, String)> {
    let usuario = db::buscarUsuario(&payload.nombreusuario, &dbPool).await;

    if usuario.is_err() == true {
        match usuario.unwrap_err() {
            //no se encontro el usuario
            sqlx::Error::RowNotFound => return Err((StatusCode::BAD_REQUEST, "User does not exist".to_string())),
            x => return Err((StatusCode::INTERNAL_SERVER_ERROR, x.to_string()))
        }
    }

    //usuario.hashContrasenna currently contains the PHC
    let valid = payload.validatePwd(usuario.unwrap().hashcontrasenna).await;
    return match valid {
        false => Err((StatusCode::UNAUTHORIZED, "Wrong password".to_string())),
        //True path should return session id
        true => Ok("Ok".to_string())
    };
}