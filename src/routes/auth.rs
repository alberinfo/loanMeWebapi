#![allow(non_snake_case)]
#![allow(clippy::needless_return)]

use axum::{http::{StatusCode, Request, header}, response::{IntoResponse, Response}, Json, extract::{State, Path}, middleware::Next};

use crate::{services::appState, models::{InputTypes::InputPerfilCrediticio, mail::{self, Mail}}};
use crate::models::{usuario::Usuario, usuario::UserError, session::Session};

pub async fn validationLayer(State(mut appState): State<appState::AppState>, req: Request<axum::body::Body>, next: Next<axum::body::Body>) -> Response {
    let redisConnection = appState.redisState.getConnection().unwrap();

    let current_path = &req.uri().path().to_string();

    let skip_paths = vec!["/auth/register", "/auth/login", "/auth/confirmUser", "/profile/requestRestorePwd", "/profile/restorePwd"]; //Añadir caminos a medida que sea necesario.
    for skip_path in skip_paths {
        if current_path.starts_with(skip_path) {
            return next.run(req).await;
        }
    }

    let auth_header = req.headers().get(axum::http::header::AUTHORIZATION).and_then(|header| header.to_str().ok());
    if auth_header.is_none() {
        return (StatusCode::BAD_REQUEST, String::from("AUTHORIZATION Header is empty")).into_response();
    }
    
    let sessionId = auth_header.unwrap().to_string(); 

    if !(Session::verifySessionById(&sessionId, redisConnection).await) {
        return (StatusCode::UNAUTHORIZED, String::from("AUTHORIZATION Header is invalid")).into_response();
    }

    let ttl = Session::getTTL(&sessionId, redisConnection).await;
    if let Err(err) = ttl {
        return (StatusCode::INTERNAL_SERVER_ERROR, format!("Redis error {:?}\n{}", err.kind(), err.detail().unwrap_or("no further detail was provided"))).into_response();
    }
    if ttl.unwrap() <= 0 {
        return (StatusCode::UNAUTHORIZED, String::from("Session has already expired")).into_response();
    }

    let session = Session {
        username: Session::getSessionUserById(&sessionId, redisConnection).await.unwrap(),
        id: sessionId,
        creationDate: None
    };
    let _ = session.refreshSession(redisConnection).await; //Make sure user's session does not timeout while hes active

    return next.run(req).await;
}

pub async fn register(State(mut appState): State<appState::AppState>, Json(payload): Json<InputPerfilCrediticio>) -> impl IntoResponse {
    let dbPool = appState.dbState.getConnection().unwrap();
    let redisConnection = appState.redisState.getConnection().unwrap();
    let mailingPool = appState.mailingState.getConnection().unwrap();

    if payload.perfil.is_none() {
        return Err((StatusCode::BAD_REQUEST, String::from("Credit history has to be provided")));
    }

    let mut usuario = payload.Usuario;
    let mut PerfilCrediticio = payload.perfil.unwrap();

    if usuario.tipoUsuario.is_none() {
        return Err((StatusCode::BAD_REQUEST, String::from("Field TipoUsuario has to be provided.")));
    }

    let _ = usuario.generatePwd().await;

    let res = usuario.crearUsuario(dbPool).await;

    match res {
        Ok(r) => match r.rows_affected() {
            0 => return Err((StatusCode::BAD_REQUEST, String::from("There was an error while creating the user"))),
            1 => {},
            _ => return Err((StatusCode::INTERNAL_SERVER_ERROR, String::from("This should not have happened.")))
        },

        //Err(r) => return Err((StatusCode::INTERNAL_SERVER_ERROR, r.to_string()))
        Err(r) => match r {
            UserError::MultithreadError(err) => return Err((StatusCode::INTERNAL_SERVER_ERROR, String::from("There was an error while creating the user"))),
            UserError::DbError(err) => match err {
                sqlx::Error::Database(DbError) => return Err((StatusCode::BAD_REQUEST, DbError.message().to_string())), //we assume that the error is a UNIQUE related error, thus user-side error.
                _ => return Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))
            }
        }
    }


    PerfilCrediticio.fkUsuario = Usuario::getUserId(&usuario.nombreUsuario, dbPool).await.unwrap();
    let res = PerfilCrediticio.save(dbPool).await;

    match res {
        Ok(r) => match r.rows_affected() {
            0 => {
                let _ = usuario.eliminarUsuario(dbPool).await;
                return Err((StatusCode::BAD_REQUEST, String::from("There was an error while creating the user")))
            },
            1 => {},
            _ => {
                let _ = usuario.eliminarUsuario(dbPool).await;
                return Err((StatusCode::INTERNAL_SERVER_ERROR, String::from("This should not have happened.")))
            }
        },

        Err(r) => return Err((StatusCode::INTERNAL_SERVER_ERROR, r.to_string()))
    }



    let mut confirmationMail = mail::Mail::SignupConfirm(usuario.clone(), String::from(""));
    let saveRes = confirmationMail.save(redisConnection).await;

    if let Err(_err) = saveRes {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, String::from("There was an error while saving your data. Contact us for further information")));
    }

    let sendRes = confirmationMail.send(mailingPool).await;

    return match sendRes {
        Ok(_r) => Ok(String::from("Done")),
        Err(err) => Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))
    };
}

pub async fn confirmUser(State(mut appState): State<appState::AppState>, Path(confirmationId): Path<String>) -> impl IntoResponse{
    let dbPool = appState.dbState.getConnection().unwrap();
    let redisConnection = appState.redisState.getConnection().unwrap();

    let mail = Mail::get("confirmationId", &confirmationId, redisConnection).await;

    if let Err(err) = mail {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{:?}: {}", err.kind(), err.detail().unwrap_or("No further detail provided"))));
    }

    let mail = mail.unwrap();

    //We do this so that we can access the enum's values
    if let Mail::SignupConfirm(Usuario, _confirmationId) = mail {
        let res = Usuario.habilitarUsuario(dbPool).await;

        return match res {
            Ok(_r) => Ok(String::from("User has been enabled")),
            Err(_err) => Err((StatusCode::INTERNAL_SERVER_ERROR, String::from("There has been an error while activating the user. Contact us for further information")))
        };
    }

    //This will not happen.
    return Err((StatusCode::BAD_REQUEST, String::from("the id does not correspond to a Signup confirmation or does not exist.")));
}

pub async fn login(State(mut appState): State<appState::AppState>, Json(payload): Json<InputPerfilCrediticio>) -> impl IntoResponse {
    let dbPool = appState.dbState.getConnection().unwrap();
    let redisConnection = appState.redisState.getConnection().unwrap();

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

    let usuario: Usuario = usuario.unwrap();

    if !usuario.habilitado {
        return Err((StatusCode::FORBIDDEN, String::from("User should confirm their email before logging in")))
    }

    //usuario.hashContrasenna currently contains the PHC
    let validPwd = payload.Usuario.validatePwd(usuario.contrasenna).await;

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

    let mut oldSession = Session {
        username: usuario.nombreUsuario.clone(),
        id: String::from(""),
        creationDate: None
    };

    let userHasActiveSession = Session::verifySessionByUsername(&usuario.nombreUsuario, redisConnection).await;

    //If the user already has an active session, close it.
    if userHasActiveSession {
        oldSession.id = Session::getSessionIdByUsername(&usuario.nombreUsuario, redisConnection).await.unwrap();
        let res = oldSession.deleteSession(redisConnection).await;
        if let Err(err) = res {
            return Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{:?}: {}", err.kind(), err.detail().unwrap_or("No further detail provided"))));
        }
    }

    let nuevaSession = Session::new(usuario.nombreUsuario).await;
    let res = nuevaSession.createSession(redisConnection).await;
    return match res {
        Ok(_) => Ok(Json(nuevaSession)),
        Err(err) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{:?}: {}", err.kind(), err.detail().unwrap_or("No further detail provided"))))
    };
}

pub async fn logout(State(mut appState): State<appState::AppState>, headers: header::HeaderMap) -> impl IntoResponse {
    let redisConnection = appState.redisState.getConnection().unwrap();
    
    let session = Session {
        username: String::from(""),
        id: headers.get(axum::http::header::AUTHORIZATION).and_then(|header| header.to_str().ok()).unwrap().to_string(), //in auth.rs we already confirmed header is Some(value)
        creationDate: None
    };

    //We dont need to check if the header exists, we already did that in auth.rs
    let res = session.deleteSession(redisConnection).await;
    return match res {
        Ok(_) => Ok(String::from("Done")),
        Err(err) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{:?}: {}", err.kind(), err.detail().unwrap_or("No further detail provided"))))
    };
}