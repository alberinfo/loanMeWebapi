#![allow(non_snake_case)]
#![allow(clippy::needless_return)]

use std::collections::HashMap;

use axum::{http::{StatusCode, header}, response::{IntoResponse}, Json, extract::{State, Path}};

use crate::{services::appState, models::{InputTypes::InputPerfilCrediticio, session::Session, usuario::{Usuario, UserError}, PerfilCrediticio::PerfilCrediticio, mail::{Mail, MailError}}};

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

pub async fn changePwd(State(mut appState): State<appState::AppState>, headers: header::HeaderMap, Json(newPwd): Json<String>) -> impl IntoResponse {
    let dbPool = appState.dbState.getConnection().unwrap();
    let redisConn = appState.redisState.getConnection().unwrap();

    let sessionId = headers.get(axum::http::header::AUTHORIZATION).and_then(|header| header.to_str().ok()).unwrap().to_string(); //in auth.rs we already confirmed header is Some(value)
    let username = Session::getSessionUserById(&sessionId, redisConn).await;

    if let Err(err) = username {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{:?}: {}", err.kind(), err.detail().unwrap_or("No further detail provided"))))
    }

    let username = username.unwrap();

    let usuario = Usuario::buscarUsuario(&username, dbPool).await;
    if let Err(r) = usuario {
        match r {
            UserError::MultithreadError(_) => return Err((StatusCode::INTERNAL_SERVER_ERROR, String::from("There was an error while processing your request"))),
            UserError::DbError(err) => match err {
                sqlx::Error::RowNotFound => return Err((StatusCode::BAD_REQUEST, String::from("User does not exist"))),
                x => return Err((StatusCode::INTERNAL_SERVER_ERROR, x.to_string()))
            }
        }
    }
    let mut usuario = usuario.unwrap();

    let validPwd = usuario.validatePwd(usuario.contrasenna.clone()).await;

    if let Err(r) = validPwd {
        match r {
            UserError::MultithreadError(_) => return Err((StatusCode::INTERNAL_SERVER_ERROR, String::from("There was an error while processing your request"))),
            UserError::DbError(err) => return Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))
        }
    }

    let validPwd = validPwd.unwrap();

    if !validPwd {
        return Err((StatusCode::UNAUTHORIZED, String::from("Wrong password")));
    }

    usuario.contrasenna = newPwd.clone();

    let _ = usuario.generatePwd().await;

    let res = usuario.actualizarUsuario(dbPool).await;

    return match res {
        Ok(_r) => Ok(String::from("DONE")),
        Err(err) => Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))
    };
}

pub async fn changeCredit(State(mut appState): State<appState::AppState>, headers: header::HeaderMap, Json(payload): Json<InputPerfilCrediticio>) -> impl IntoResponse {
    let dbPool = appState.dbState.getConnection().unwrap();
    let redisConn = appState.redisState.getConnection().unwrap();

    if payload.perfil.is_none() {
        return Err((StatusCode::BAD_REQUEST, String::from("New credit data has to be provided")));
    }
    let payloadPerfil = payload.perfil.unwrap();

    let sessionId = headers.get(axum::http::header::AUTHORIZATION).and_then(|header| header.to_str().ok()).unwrap().to_string(); //in auth.rs we already confirmed header is Some(value)
    let username = Session::getSessionUserById(&sessionId, redisConn).await;

    if let Err(err) = username {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{:?}: {}", err.kind(), err.detail().unwrap_or("No further detail provided"))))
    }

    let username = username.unwrap();

    let usuario = Usuario::buscarUsuario(&username, dbPool).await;
    if let Err(r) = usuario {
        match r {
            UserError::MultithreadError(_) => return Err((StatusCode::INTERNAL_SERVER_ERROR, String::from("There was an error while processing your request"))),
            UserError::DbError(err) => match err {
                sqlx::Error::RowNotFound => return Err((StatusCode::BAD_REQUEST, String::from("User does not exist"))),
                x => return Err((StatusCode::INTERNAL_SERVER_ERROR, x.to_string()))
            }
        }
    }
    let usuario = usuario.unwrap();

    let credit = PerfilCrediticio::get(usuario.id, dbPool).await;
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

pub async fn requestRestorePwd(State(mut appState): State<appState::AppState>, Json(username): Json<String>) -> impl IntoResponse {
    let dbPool = appState.dbState.getConnection().unwrap();
    let redisConn = appState.redisState.getConnection().unwrap();
    let mailingPool = appState.mailingState.getConnection().unwrap();

    let user = Usuario::buscarUsuario(&username, dbPool).await;
    if let Err(r) = user {
        match r {
            UserError::MultithreadError(_) => return Err((StatusCode::INTERNAL_SERVER_ERROR, String::from("There was an error while processing your request"))),
            UserError::DbError(err) => match err {
                sqlx::Error::RowNotFound => return Err((StatusCode::BAD_REQUEST, String::from("User does not exist"))),
                x => return Err((StatusCode::INTERNAL_SERVER_ERROR, x.to_string()))
            }
        }
    }

    let user = user.unwrap();
    let mut mail = Mail::PwdRestore(user, String::from(""));

    let res = mail.save(redisConn).await;

    if let Err(err) = res {
        return Err(((StatusCode::INTERNAL_SERVER_ERROR), format!("{:?}: {}", err.kind(), err.detail().unwrap_or("No further detail provided"))));
    }

    let res = mail.send(mailingPool).await;
    
    if let Err(err) = res {
        let errorContent = match err {
            MailError::Test => (StatusCode::NO_CONTENT, String::from("test")),
            MailError::AddressError(r) => (StatusCode::INTERNAL_SERVER_ERROR, r.to_string()),
            MailError::EmailError(r) => (StatusCode::INTERNAL_SERVER_ERROR, r.to_string()),
            MailError::SendError(r) => (StatusCode::INTERNAL_SERVER_ERROR, r.to_string())
        };
        return Err(errorContent);
    }

    return Ok("Done");
}

pub async fn restorePwd(State(mut appState): State<appState::AppState>, Path(restoreId): Path<String>, Json(newPwd): Json<String>) -> impl IntoResponse {
    let dbPool = appState.dbState.getConnection().unwrap();
    let redisConn = appState.redisState.getConnection().unwrap();

    let mail = Mail::get("restoreId", &restoreId, redisConn).await;

    if let Err(err) = mail {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{:?}: {}", err.kind(), err.detail().unwrap_or("No further detail provided"))));
    }

    let mail = mail.unwrap();

    //We do this so that we can access the enum's values
    if let Mail::PwdRestore(mut Usuario, _restoreId) = mail {
        Usuario.contrasenna = newPwd;
        let _ = Usuario.generatePwd();
        let res = Usuario.actualizarUsuario(dbPool).await;

        return match res {
            Ok(_r) => Ok(String::from("Password has been updated")),
            Err(_err) => Err((StatusCode::INTERNAL_SERVER_ERROR, String::from("There has been an error while activating the user. Contact us for further information")))
        };
    }

    //This will not happen.
    return Ok(String::from(""));
}