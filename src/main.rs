#![allow(non_snake_case)]

use axum::{routing::*, middleware};
use axum_server::tls_rustls::RustlsConfig;
use std::{error::Error, net::SocketAddr};
use loanMeWebapi::routes::*;
use loanMeWebapi::services::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt().with_max_level(tracing::Level::DEBUG).init(); //Initialize logging

    dotenvy::dotenv()?; //load environment vars

    let mut pgState = db::dbState::default();
    pgState.connect().await?;
    pgState.migrateDb().await?; //migrates the database if its empty
    

    let mut rdState = redisServer::redisState::default();
    rdState.connect().await?;

    let appState = appState::AppState {
        dbState: pgState,
        redisState: rdState
    };

    //All routes nested under /auth (i.e /auth/login)
    let auth: Router = axum::Router::new()
        .route("/registro", post(auth::registro))
        .route("/login", post(auth::login))
        .route("/logout", post(auth::logout))
        .layer(middleware::from_fn_with_state(appState.clone(), auth::validationLayer))
        .with_state(appState);

    //Al routes nested under /api (i.e, /auth/*)
    let api: Router = axum::Router::new()
        .nest("/auth", auth);

    //All routes nested under / (i.e, /api/*)
    let app: Router = axum::Router::new()
        .nest("/api", api)
        .fallback(endpoints::pageNotFound);

    let addr: SocketAddr = SocketAddr::from(([127,0,0,1], 4433));
    let config: RustlsConfig = RustlsConfig::from_pem_file(
        std::env::var("TLS_CERT_PATH").unwrap(),
        std::env::var("TLS_KEY_PATH").unwrap()
    )
    .await
    .unwrap();
    
    axum_server::bind_rustls(addr, config)
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}