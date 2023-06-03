#![allow(non_snake_case)]
#![allow(clippy::needless_return)]

use axum::{http::{StatusCode, Request, header}, response::{IntoResponse, Response}, Json, extract::State, middleware::Next};

use crate::services::appState;
use crate::models::{usuario::Usuario, session::Session};

pub async fn validationLayer(State(mut appState): State<appState::AppState>, req: Request<axum::body::Body>, next: Next<axum::body::Body>) -> Response {
    let redisConnection = appState.redisState.getConnection().unwrap();

    let current_path = &req.uri().path().to_string();

    let skip_paths = vec!["/registro", "/login"]; //Añadir caminos a medida que sea necesario.
    for skip_path in skip_paths {
        if current_path.ends_with(skip_path) {
            return next.run(req).await;
        }
    }

    let auth_header = req.headers().get(axum::http::header::AUTHORIZATION).and_then(|header| header.to_str().ok());
    if auth_header.is_none() {
        return (StatusCode::BAD_REQUEST, String::from("AUTHORIZATION Header is empty")).into_response();
    }
    
    let session = Session {
        username: String::from(""),
        id: auth_header.unwrap().to_string(),
        creationDate: None
    };
    if !(session.verifySession(redisConnection).await) {
        return (StatusCode::UNAUTHORIZED, String::from("AUTHORIZATION Header is invalid")).into_response();
    }

    let ttl = session.getTTL(redisConnection).await;
    if ttl.is_err() {
        let err = ttl.err().unwrap();
        return (StatusCode::INTERNAL_SERVER_ERROR, format!("Redis error {:?}\n{}", err.kind(), err.detail().unwrap_or("no further detail was provided"))).into_response();
    }
    if ttl.unwrap() <= 0 {
        return (StatusCode::UNAUTHORIZED, String::from("Session has already expired")).into_response();
    }

    let _ = session.refreshSession(redisConnection).await; //Make sure user's session does not timeout while hes active

    return next.run(req).await;
}

pub async fn registro(State(appState): State<appState::AppState>, Json(mut payload): Json<Usuario>) -> impl IntoResponse {
    let dbState = appState.dbState;

    payload.hashcontrasenna = payload.generatePwd().await;
    let res = payload.guardarUsuario(dbState.dbPool.as_ref().unwrap()).await;

    return match res {
        Ok(r) => match r.rows_affected() {
            0 => Err((StatusCode::BAD_REQUEST, String::from("There was an error while creating the user"))),
            1 => Ok(String::from("Done")),
            _ => Err((StatusCode::INTERNAL_SERVER_ERROR, String::from("This should not have happened.")))
        },

        Err(r) => Err((StatusCode::INTERNAL_SERVER_ERROR, r.to_string()))
    };
}

pub async fn login(State(mut appState): State<appState::AppState>, Json(payload): Json<Usuario>) -> impl IntoResponse {
    let dbPool = appState.dbState.getConnection().unwrap();
    let redisConnection = appState.redisState.getConnection().unwrap();
    
    let usuario = payload.buscarUsuario(dbPool).await;

    if usuario.is_err() {
        match usuario.unwrap_err() {
            //no se encontro el usuario
            sqlx::Error::RowNotFound => return Err((StatusCode::BAD_REQUEST, String::from("User does not exist"))),
            x => return Err((StatusCode::INTERNAL_SERVER_ERROR, x.to_string()))
        }
    }

    let usuario: Usuario = usuario.unwrap();

    //usuario.hashContrasenna currently contains the PHC
    let validPwd = payload.validatePwd(usuario.hashcontrasenna).await;

    if !validPwd {
        return Err((StatusCode::UNAUTHORIZED, String::from("Wrong password")));
    }

    let nuevaSession = Session::new(usuario.nombreusuario).await;
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