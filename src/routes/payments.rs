#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::needless_return)]

use axum::{response::IntoResponse, extract::State, http::{header, StatusCode}, Json};
use strum::IntoEnumIterator;
use crate::{models::{PrestamoTxn::AcceptedBlockchains, InputTypes::InputTxn, session::Session, usuario::Usuario, Prestamo::LoanError}, services::appState};

pub async fn getAcceptedCurrencies() -> impl IntoResponse {
    let acceptedCurrencies = AcceptedBlockchains::iter().collect::<Vec<_>>();

    let ret = serde_json::to_string(&acceptedCurrencies);

    if let Err(_r) = ret {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, "There was an error while fetching the data. Try again later"));
    }

    return Ok(ret.unwrap());
}

pub async fn addTxn(State(mut appState): State<appState::AppState>, headers: header::HeaderMap, Json(payload): Json<InputTxn>) -> impl IntoResponse {
    let dbPool = appState.dbState.getConnection().unwrap();
    let redisConn = appState.redisState.getConnection().unwrap();

    let txn = payload.txn;
    
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

    let res = txn.addTxn(&user, dbPool).await;

    return match res {
        Ok(_) => Ok("Done"),
        Err(r) => match r {
            LoanError::DbError(ref _err) => Err((StatusCode::INTERNAL_SERVER_ERROR, r.to_string())),
            LoanError::InvalidDate | LoanError::InvalidUser => Err((StatusCode::BAD_REQUEST, r.to_string())),
            LoanError::InvalidUserType { .. } => Err((StatusCode::BAD_REQUEST, r.to_string())),
            LoanError::UserUnauthorized { .. } => Err((StatusCode::FORBIDDEN, r.to_string()))
        }
    }
}