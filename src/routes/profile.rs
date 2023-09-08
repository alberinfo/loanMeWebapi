#![allow(non_snake_case)]
#![allow(clippy::needless_return)]

use std::collections::HashMap;

use axum::{http::{StatusCode, header}, response::{IntoResponse}, Json, extract::State};

use crate::{services::appState, models::{InputTypes::InputPerfilCrediticio, session::Session, usuario::Usuario, PerfilCrediticio::PerfilCrediticio}};

//Reads current user info
pub async fn getUserInfo(State(mut appState): State<appState::AppState>, headers: header::HeaderMap) -> impl IntoResponse {
    let dbPool = appState.dbState.getConnection().unwrap();
    let redisConn = appState.redisState.getConnection().unwrap();

    let sessionId = headers.get(axum::http::header::AUTHORIZATION).and_then(|header| header.to_str().ok()).unwrap().to_string(); //in auth.rs we already confirmed header is Some(value)
    let res = Session::getSessionUserById(&sessionId, redisConn).await;

    if let Err(err) = res {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{:?}: {}", err.kind(), err.detail().unwrap_or("No further detail provided"))))
    }

    let res = Usuario::buscarUsuario(&res.unwrap(), dbPool).await;
    if let Err(err) = res {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))
    }

    let user = res.unwrap();

    let res = PerfilCrediticio::get(user.id, dbPool).await;
    if let Err(err) = res {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string()));
    }

    let perfilCrediticio = res.unwrap();

    let response = InputPerfilCrediticio {
        Usuario: user,
        perfil: Some(perfilCrediticio),
        extra: HashMap::new()
    };

    return Ok(Json(response));
}

pub async fn changePwd(State(appState): State<appState::AppState>, Json(payload): Json<InputPerfilCrediticio>) -> impl IntoResponse {
    let dbPool = appState.dbState.getConnection().unwrap();

    let usuario = Usuario::buscarUsuario(&payload.Usuario.nombreusuario, dbPool).await;
    if let Err(err) = usuario {
        match err {
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

pub async fn changeCredit(State(appState): State<appState::AppState>, Json(payload): Json<InputPerfilCrediticio>) -> impl IntoResponse {
    let dbPool = appState.dbState.getConnection().unwrap();

    if payload.perfil.is_none() {
        return Err((StatusCode::BAD_REQUEST, String::from("New credit data has to be provided")));
    }
    let payloadPerfil = payload.perfil.unwrap();

    let credit = PerfilCrediticio::get(payloadPerfil.fkusuario, dbPool).await;
    if let Err(err) = credit {
        match err {
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
