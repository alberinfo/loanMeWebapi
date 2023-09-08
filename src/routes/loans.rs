#![allow(non_snake_case)]
#![allow(clippy::needless_return)]

use axum::{response::IntoResponse, Json, extract::State, http::StatusCode};

use crate::models::InputTypes::InputPrestamo;
use crate::models::Prestamo::*;
use crate::models::usuario::Usuario;
use crate::services::appState;

pub async fn getLoanOffers(State(appState): State<appState::AppState>) -> impl IntoResponse {
    let dbPool = appState.dbState.getConnection().unwrap();
    let result: Result<Vec<Prestamo>, LoanError> = Prestamo::getAllLoanOffers(dbPool).await;

    let defaultUser = Usuario { id: 0, email: String::from(""), nombrecompleto: String::from(""), nombreusuario: String::from(""), contrasenna: String::from(""), idwallet: None, tipousuario: None, habilitado: false };
    //let loaners = vec![defaultUser; result.]

    //We know getAllLoanOffers only returns Ok or Err(sqlx::Error)
    if let Err(err) = result {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string()));
    }

    let result = result.unwrap();
    return Ok(Json(result));
}

pub async fn getLoanRequests(State(appState): State<appState::AppState>) -> impl IntoResponse {
    let dbPool = appState.dbState.getConnection().unwrap();
    let result: Result<Vec<Prestamo>, LoanError> = Prestamo::getAllLoanRequests(dbPool).await;

    let defaultUser = Usuario { id: 0, email: String::from(""), nombrecompleto: String::from(""), nombreusuario: String::from(""), contrasenna: String::from(""), idwallet: None, tipousuario: None, habilitado: false };
    //let loaners = vec![defaultUser; result.]


    //We know getAllLoanRequests only returns Ok or Err(sqlx::Error)
    if let Err(err) = result {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string()));
    }

    let result = result.unwrap();
    return Ok(Json(result));
}

pub async fn getLoanById(State(appState): State<appState::AppState>, Json(LoanId): Json<i64>) -> impl IntoResponse {
    let dbPool = appState.dbState.getConnection().unwrap();

    let result = Prestamo::getLoanById(LoanId, dbPool).await;
    
    if let Err(err) = result {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string()));
    }

    let result = result.unwrap();
    return Ok(Json(result));
}

pub async fn createLoanOffer(State(appState): State<appState::AppState>, Json(payload): Json<InputPrestamo>) -> impl IntoResponse {
    let dbPool = appState.dbState.getConnection().unwrap();
    
    let loan = payload.Loan;
    let user = payload.Usuario;

    let result = loan.createLoanOffer(user, dbPool).await;

    return match result {
        Ok(()) => Ok(()),
        Err(r) => match r {
            LoanError::DbError(ref _err) => Err((StatusCode::INTERNAL_SERVER_ERROR, r.to_string())),
            LoanError::InvalidUserType { ref found } => Err((StatusCode::BAD_REQUEST, r.to_string())),
            LoanError::UserUnauthorized { ref expected, ref found} => Err((StatusCode::FORBIDDEN, r.to_string()))
        }
    }
}

pub async fn createLoanRequest(State(appState): State<appState::AppState>, Json(payload): Json<InputPrestamo>) -> impl IntoResponse {
    let dbPool = appState.dbState.getConnection().unwrap();
    
    let loan = payload.Loan;
    let user = payload.Usuario;

    let result = loan.createLoanRequest(user, dbPool).await;

    return match result {
        Ok(()) => Ok(()),
        Err(r) => match r {
            LoanError::DbError(ref _err) => Err((StatusCode::INTERNAL_SERVER_ERROR, r.to_string())),
            LoanError::InvalidUserType { ref found } => Err((StatusCode::BAD_REQUEST, r.to_string())),
            LoanError::UserUnauthorized { ref expected, ref found} => Err((StatusCode::FORBIDDEN, r.to_string()))
        }
    }
}