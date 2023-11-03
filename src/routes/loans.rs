#![allow(non_snake_case)]
#![allow(clippy::needless_return)]

use axum::http::header;
use axum::{response::IntoResponse, Json, extract::State, http::StatusCode};
use futures::StreamExt;

use crate::models::InputTypes::{InputPrestamo, InputTxn};
use crate::models::PrestamoTxn::PrestamoTxn;
use crate::models::{Prestamo::*, PrestamoTxn::*};
use crate::models::session::Session;
use crate::models::usuario::Usuario;
use crate::models::mail::{self, Mail};
use crate::services::appState;

pub async fn getLoanOffers(State(appState): State<appState::AppState>) -> impl IntoResponse {
    let dbPool = appState.dbState.getConnection().unwrap();
    let result: Result<Vec<Prestamo>, LoanError> = Prestamo::getAllLoanOffers(dbPool).await;

    //We know getAllLoanOffers only returns Ok or Err(sqlx::Error)
    if let Err(err) = result {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string()));
    }

    let LoanOffers: Vec<Prestamo> = result.unwrap();
    let Futures: futures::stream::FuturesOrdered<_> = LoanOffers.into_iter().map(|LoanOffer: Prestamo| async move {
        let fkPrestamista = LoanOffer.fkPrestamista.unwrap();

        let loanItem = LoanItem {
            loan: LoanOffer,
            txns: Vec::new(),
            user: Usuario::buscarUsuarioById(fkPrestamista, dbPool).await.unwrap()
        };

        loanItem

        //(LoanOffer, Usuario::buscarUsuarioById(fkPrestamista, dbPool).await.unwrap())
    }).collect();
    //let LoanOffersWithLoaners: Vec<(Prestamo, Usuario)> = Futures.collect().await;
    let LoanOffersWithLoaners: Vec<LoanItem> = Futures.collect().await;

    return Ok(Json(LoanOffersWithLoaners));
}

pub async fn getLoanRequests(State(appState): State<appState::AppState>) -> impl IntoResponse {
    let dbPool = appState.dbState.getConnection().unwrap();
    let result: Result<Vec<Prestamo>, LoanError> = Prestamo::getAllLoanRequests(dbPool).await;

    //We know getAllLoanRequests only returns Ok or Err(sqlx::Error)
    if let Err(err) = result {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string()));
    }

    let LoanRequests: Vec<Prestamo> = result.unwrap();

    let Futures: futures::stream::FuturesOrdered<_> = LoanRequests.into_iter().map(|LoanRequest: Prestamo| async move {
        let fkPrestatario = LoanRequest.fkPrestatario.unwrap();
        
        let loanItem = LoanItem {
            loan: LoanRequest,
            txns: Vec::new(),
            user: Usuario::buscarUsuarioById(fkPrestatario, dbPool).await.unwrap()
        };

        loanItem
        
        //(LoanOffer, Usuario::buscarUsuarioById(fkPrestatario, dbPool).await.unwrap())
    }).collect();
    //let LoanRequestsWithLoanees: Vec<(Prestamo, Usuario)> = Futures.collect().await;
    let LoanRequestsWithLoanees: Vec<LoanItem> = Futures.collect().await;
    
    return Ok(Json(LoanRequestsWithLoanees));
}

pub async fn getLoanById(State(appState): State<appState::AppState>, Json(LoanId): Json<i64>) -> impl IntoResponse {
    let dbPool = appState.dbState.getConnection().unwrap();

    let result = Prestamo::getLoanById(LoanId, dbPool).await;
    
    if let Err(err) = result {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string()));
    }

    let result = result.unwrap();

    let fkUsuario = result.fkPrestamista.unwrap_or(result.fkPrestatario.unwrap());
    let LoanWithUser = LoanItem {
        loan: result,
        txns: PrestamoTxn::getAllTxns(LoanId, dbPool).await.unwrap(),
        user: Usuario::buscarUsuarioById(fkUsuario, dbPool).await.unwrap()
    };

    return Ok(Json(LoanWithUser));
}

pub async fn createLoanOffer(State(mut appState): State<appState::AppState>, headers: header::HeaderMap, Json(payload): Json<InputPrestamo>) -> impl IntoResponse {
    let dbPool = appState.dbState.getConnection().unwrap();
    let redisConn = appState.redisState.getConnection().unwrap();

    let mut loan = payload.Loan;
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

    let result = loan.createLoanOffer(user, dbPool).await;

    return match result {
        Ok(()) => Ok("Done"),
        Err(r) => match r {
            LoanError::DbError(ref _err) => Err((StatusCode::INTERNAL_SERVER_ERROR, r.to_string())),
            LoanError::InvalidDate | LoanError::InvalidUser => Err((StatusCode::BAD_REQUEST, r.to_string())),
            LoanError::InvalidUserType { ref found } => Err((StatusCode::BAD_REQUEST, r.to_string())),
            LoanError::UserUnauthorized { ref expected, ref found} => Err((StatusCode::FORBIDDEN, r.to_string()))
        }
    }
}

pub async fn createLoanRequest(State(mut appState): State<appState::AppState>, headers: header::HeaderMap, Json(payload): Json<InputPrestamo>) -> impl IntoResponse {
    let dbPool = appState.dbState.getConnection().unwrap();
    let redisConn = appState.redisState.getConnection().unwrap();

    let mut loan = payload.Loan;
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

    let result = loan.createLoanRequest(user, dbPool).await;

    return match result {
        Ok(()) => Ok("Done"),
        Err(r) => match r {
            LoanError::DbError(ref _err) => Err((StatusCode::INTERNAL_SERVER_ERROR, r.to_string())),
            LoanError::InvalidDate | LoanError::InvalidUser => Err((StatusCode::BAD_REQUEST, r.to_string())),
            LoanError::InvalidUserType { ref found } => Err((StatusCode::BAD_REQUEST, r.to_string())),
            LoanError::UserUnauthorized { ref expected, ref found} => Err((StatusCode::FORBIDDEN, r.to_string()))
        }
    }
}

pub async fn proposeCompleteLoan(State(mut appState): State<appState::AppState>, headers: header::HeaderMap, Json(LoanId): Json<i64>) -> impl IntoResponse {
    let dbPool = appState.dbState.getConnection().unwrap();
    let redisConn = appState.redisState.getConnection().unwrap();
    let mailingPool = appState.mailingState.getConnection().unwrap();

    let sessionId = headers.get(axum::http::header::AUTHORIZATION).and_then(|header| header.to_str().ok()).unwrap().to_string(); //in auth.rs we already confirmed header is Some(value)
    let res = Session::getSessionUserById(&sessionId, redisConn).await;
    if let Err(err) = res {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{:?}: {}", err.kind(), err.detail().unwrap_or("No further detail provided"))))
    }

    let res = Usuario::buscarUsuario(&res.unwrap(), dbPool).await;
    if let Err(err) = res {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))
    }

    let user = res.unwrap(); //User proposing completion

    let _ = Prestamo::proposeCompleteLoan(LoanId, &user, dbPool);

    let loan = Prestamo::getLoanById(LoanId, dbPool).await;

    if let Err(r) = loan {
        return match r {
            LoanError::DbError(ref _err) => Err((StatusCode::INTERNAL_SERVER_ERROR, r.to_string())),
            LoanError::InvalidDate | LoanError::InvalidUser => Err((StatusCode::BAD_REQUEST, r.to_string())),
            LoanError::InvalidUserType { ref found } => Err((StatusCode::BAD_REQUEST, r.to_string())),
            LoanError::UserUnauthorized { ref expected, ref found} => Err((StatusCode::FORBIDDEN, r.to_string()))
        }
    }

    let loan = loan.unwrap();

    let res = Usuario::buscarUsuarioById(loan.fkPrestamista.unwrap_or(loan.fkPrestatario.unwrap()), dbPool).await; //If fkPrestamista is None, then fkPrestatario surely is Some, and viceversa

    if let Err(err) = res {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string()));
    }

    let loanCreator = res.unwrap();

    let mail = mail::Mail::LoanProposal(loanCreator, user, loan.id);
    let sendRes = mail.send(mailingPool).await;

    return match sendRes {
        Ok(_r) => Ok(String::from("Done")),
        Err(err) => Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))
    };;
}


pub async fn completeLoan(State(mut appState): State<appState::AppState>, headers: header::HeaderMap, Json(completionProposal): Json<PrestamoPropuesta>) -> impl IntoResponse {
    let dbPool = appState.dbState.getConnection().unwrap();
    let redisConn = appState.redisState.getConnection().unwrap();
    let mailingPool = appState.mailingState.getConnection().unwrap();

    let res = Usuario::buscarUsuarioById(completionProposal.fkUsuario, dbPool).await;
    if let Err(err) = res {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))
    }

    let user = res.unwrap();
    
    let res = Prestamo::completeLoan(completionProposal.fkPrestamo, &user, dbPool).await;

    if let Err(r) = res {
        return match r {
            LoanError::DbError(ref _err) => Err((StatusCode::INTERNAL_SERVER_ERROR, r.to_string())),
            LoanError::InvalidDate | LoanError::InvalidUser => Err((StatusCode::BAD_REQUEST, r.to_string())),
            LoanError::InvalidUserType { ref found } => Err((StatusCode::BAD_REQUEST, r.to_string())),
            LoanError::UserUnauthorized { ref expected, ref found} => Err((StatusCode::FORBIDDEN, r.to_string()))
        }
    }

    let res = Usuario::buscarUsuarioById(completionProposal.fkUsuario, dbPool).await; //If fkPrestamista is None, then fkPrestatario surely is Some, and viceversa

    if let Err(err) = res {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string()));
    }

    let loanCompleter = res.unwrap();


    let mail = mail::Mail::LoanProposalAccepted(loanCompleter, completionProposal.fkPrestamo);
    let sendRes = mail.send(mailingPool).await;

    return match sendRes {
        Ok(_r) => Ok(String::from("Done")),
        Err(err) => Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))
    };;
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
            LoanError::InvalidUserType { ref found } => Err((StatusCode::BAD_REQUEST, r.to_string())),
            LoanError::UserUnauthorized { ref expected, ref found} => Err((StatusCode::FORBIDDEN, r.to_string()))
        }
    }
}