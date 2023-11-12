#![allow(non_snake_case)]
#![allow(clippy::needless_return)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

use std::collections::HashMap;

use axum::{http::{StatusCode, header}, response::IntoResponse, Json, extract::{State, Path}};
use futures::StreamExt;

use crate::{services::appState, models::{InputTypes::InputPerfilCrediticio, session::Session, usuario::{Usuario, UserError}, PerfilCrediticio::PerfilCrediticio, mail::{Mail, MailError}, Prestamo::{Prestamo, LoanError}, PrestamoPropuesta::{PrestamoPropuesta, PropuestaItem}}};

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

    let mut response = InputPerfilCrediticio {
        Usuario: user.clone(),
        perfil: Some(perfilCrediticio),
        extra: HashMap::new()
    };

    let allLoansFromUser = Prestamo::getAllLoansFromUser(user.id, dbPool).await;

    if let Err(r) = allLoansFromUser {
        return match r {
            LoanError::DbError(ref _err) => Err((StatusCode::INTERNAL_SERVER_ERROR, r.to_string())),
            LoanError::InvalidDate | LoanError::InvalidUser => Err((StatusCode::BAD_REQUEST, r.to_string())),
            LoanError::InvalidUserType { .. } => Err((StatusCode::BAD_REQUEST, r.to_string())),
            LoanError::UserUnauthorized { ..} => Err((StatusCode::FORBIDDEN, r.to_string()))
        }
    }

    response.extra.insert(String::from("loans"), serde_json::to_value(allLoansFromUser.unwrap()).unwrap());

    let allLoanProposals = PrestamoPropuesta::getAllLoanProposalsForUser(user.id, dbPool).await;

    if let Err(r) = allLoanProposals {
        return match r {
            LoanError::DbError(ref _err) => Err((StatusCode::INTERNAL_SERVER_ERROR, r.to_string())),
            LoanError::InvalidDate | LoanError::InvalidUser => Err((StatusCode::BAD_REQUEST, r.to_string())),
            LoanError::InvalidUserType { .. } => Err((StatusCode::BAD_REQUEST, r.to_string())),
            LoanError::UserUnauthorized { .. } => Err((StatusCode::FORBIDDEN, r.to_string()))
        }
    }

    let allLoanProposals = allLoanProposals.unwrap();

    let futures: futures::stream::FuturesOrdered<_> = allLoanProposals.into_iter().map(|LoanProposal: PrestamoPropuesta| async move {
        let PropuestaItem = PropuestaItem {
            fkPrestamo: LoanProposal.fkPrestamo,
            usuario: Usuario::buscarUsuarioById(LoanProposal.fkUsuario, dbPool).await.unwrap()
        };

        PropuestaItem
    }).collect();

    let allLoanProposals: Vec<PropuestaItem> = futures.collect().await;

    response.extra.insert(String::from("loanProposals"), serde_json::to_value(allLoanProposals).unwrap());

    return Ok(Json(response));
}

pub async fn changePwd(State(appState): State<appState::AppState>, Json(payload): Json<InputPerfilCrediticio>) -> impl IntoResponse {
    let dbPool = appState.dbState.getConnection().unwrap();

    let usuario = Usuario::buscarUsuario(&payload.Usuario.nombreUsuario, dbPool).await;
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

    let validPwd = payload.Usuario.validatePwd(usuario.contrasenna.clone()).await;

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

    if payloadPerfil.historialCrediticio.is_some() { credit.historialCrediticio = payloadPerfil.historialCrediticio; }
    if payloadPerfil.extractoBancario.is_some() { credit.extractoBancario = payloadPerfil.extractoBancario; }
    if payloadPerfil.comprobanteDeIngreso.is_some() { credit.comprobanteDeIngreso = payloadPerfil.comprobanteDeIngreso; }
    if payloadPerfil.descripcionFinanciera.is_some() { credit.descripcionFinanciera = payloadPerfil.descripcionFinanciera; }

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
    let mut mail = Mail::PwdRestore(user, String::new());

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
        let _ = Usuario.generatePwd().await;
        let res = Usuario.actualizarUsuario(dbPool).await;

        return match res {
            Ok(_r) => Ok(String::from("Password has been updated")),
            Err(_err) => Err((StatusCode::INTERNAL_SERVER_ERROR, String::from("There has been an error while activating the user. Contact us for further information")))
        };
    }

    //This will not happen.
    return Ok(String::new());
}