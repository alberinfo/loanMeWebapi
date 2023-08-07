#![allow(non_snake_case)]
#![allow(clippy::needless_return)]

use axum::http::StatusCode;
use axum::{response::IntoResponse, Json, extract::State};

use crate::models::Prestamo;
use crate::models::usuario::Usuario;
use crate::services::appState;

pub async fn getLoanOffers(State(appState): State<appState::AppState>) -> impl IntoResponse {
    let dbPool = appState.dbState.getConnection().unwrap();
    let result: sqlx::Result<Vec<Prestamo::Prestamo>> = Prestamo::Prestamo::getAllLoanOffers(dbPool).await;

    let defaultUser = Usuario { id: 0, email: String::from(""), nombrecompleto: String::from(""), nombreusuario: String::from(""), contrasenna: String::from(""), idwallet: None, tipousuario: None };
    //let loaners = vec![defaultUser; result.]

    return match result {
        Ok(offers) => Ok(axum::Json(offers)),
        Err(r) => Err((StatusCode::INTERNAL_SERVER_ERROR, r.to_string()))
    }
}