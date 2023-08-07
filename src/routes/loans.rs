#![allow(non_snake_case)]
#![allow(clippy::needless_return)]

use axum::{response::IntoResponse, Json, extract::State};

use crate::services::appState;

pub async fn getLoanOffers(State(appState): State<appState::AppState>) -> impl IntoResponse {
    
}