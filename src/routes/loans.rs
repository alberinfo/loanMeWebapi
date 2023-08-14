#![allow(non_snake_case)]
#![allow(clippy::needless_return)]

use axum::{response::IntoResponse, Json, extract::State, http::StatusCode};

use crate::models::Prestamo::*;
use crate::models::usuario::Usuario;
use crate::services::appState;

pub async fn getLoanOffers(State(appState): State<appState::AppState>) -> impl IntoResponse {
    let dbPool = appState.dbState.getConnection().unwrap();
    let result: Result<Vec<Prestamo>, LoanError> = Prestamo::getAllLoanOffers(dbPool).await;

    let defaultUser = Usuario { id: 0, email: String::from(""), nombrecompleto: String::from(""), nombreusuario: String::from(""), contrasenna: String::from(""), idwallet: None, tipousuario: None };
    //let loaners = vec![defaultUser; result.]

    return match result {
        Ok(offers) => Ok(Json(offers)),
        Err(r) => match r {
            LoanError::DbError(ref _err) => Err((StatusCode::INTERNAL_SERVER_ERROR, r.to_string())),
            LoanError::InvalidUserType { ref found } => Err((StatusCode::BAD_REQUEST, r.to_string())),
            LoanError::UserUnauthorized { ref expected, ref found} => Err((StatusCode::FORBIDDEN, r.to_string()))
        }
    }
}