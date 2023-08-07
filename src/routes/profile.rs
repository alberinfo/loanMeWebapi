#![allow(non_snake_case)]
#![allow(clippy::needless_return)]

use axum::{http::{StatusCode}, response::{IntoResponse}, Json, extract::State};

use crate::{services::appState, models::userInput::UserInput};

pub async fn changePwd(State(appState): State<appState::AppState>, Json(payload): Json<UserInput>) -> impl IntoResponse {
    let dbPool = appState.dbState.getConnection().unwrap();

    let usuario = payload.Usuario.buscarUsuario(dbPool).await;
    if usuario.is_err() {
        match usuario.unwrap_err() {
            //no se encontro el usuario
            sqlx::Error::RowNotFound => return Err((StatusCode::BAD_REQUEST, String::from("User does not exist"))),
            x => return Err((StatusCode::INTERNAL_SERVER_ERROR, x.to_string()))
        }
    }
    let mut usuario = usuario.unwrap();

    let validPwd = payload.Usuario.validatePwd(usuario.contrasenna.clone()).await;

    if !validPwd {
        return Err((StatusCode::UNAUTHORIZED, String::from("Wrong password")));
    }

    if !payload.extra.contains_key("newPwd") {
        return Err((StatusCode::BAD_REQUEST, String::from("new password has to be provided")));
    }

    usuario.contrasenna = payload.extra.get("newPwd").unwrap().as_str().unwrap().to_string();

    let _ = usuario.generatePwd().await;

    let res = usuario.actualizarUsuario(dbPool).await;

    return match res {
        Ok(_r) => Ok(String::from("DONE")),
        Err(err) => Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))
    };
}

pub async fn changeCredit(State(appState): State<appState::AppState>, Json(payload): Json<UserInput>) -> impl IntoResponse {
    let dbPool = appState.dbState.getConnection().unwrap();

    if payload.perfil.is_none() {
        return Err((StatusCode::BAD_REQUEST, String::from("New credit data has to be provided")));
    }
    let payloadPerfil = payload.perfil.unwrap();

    let credit = payloadPerfil.get(dbPool).await;
    if credit.is_err() {
        match credit.unwrap_err() {
            //no se encontro el usuario
            sqlx::Error::RowNotFound => return Err((StatusCode::BAD_REQUEST, String::from("Credit history does not exist"))),
            x => return Err((StatusCode::INTERNAL_SERVER_ERROR, x.to_string()))
        }
    }
    let mut credit = credit.unwrap();

    if payloadPerfil.historialcrediticio.is_some() { credit.historialcrediticio = payloadPerfil.historialcrediticio; }
    if payloadPerfil.extractobancario.is_some() { credit.extractobancario = payloadPerfil.extractobancario; }
    if payloadPerfil.comprobantedeingreso.is_some() { credit.comprobantedeingreso = payloadPerfil.comprobantedeingreso; }
    if payloadPerfil.descripcionfinanciera.is_some() { credit.descripcionfinanciera = payloadPerfil.descripcionfinanciera; }

    let res = credit.update(dbPool).await;
    return match res {
        Ok(r) => match r.rows_affected() {
            0 => Err((StatusCode::BAD_REQUEST, String::from("There was an error while updating the credit data"))),
            1 => Ok(String::from("Done")),
            _ => Err((StatusCode::INTERNAL_SERVER_ERROR, String::from("This should not have happened.")))
        }

        Err(r) => Err((StatusCode::INTERNAL_SERVER_ERROR, r.to_string()))
    };
}