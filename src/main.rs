#![allow(non_snake_case)]

use axum::{routing::*, middleware};
use axum_server::tls_rustls::RustlsConfig;
use std::{error::Error, net::SocketAddr};
use loanMeWebapi::routes::*;
use loanMeWebapi::services::*;
use tower_http::cors::CorsLayer;

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

    let loans: Router<appState::AppState, _> = axum::Router::new();
        //.route("/getLoanOffers", get(loans::))

    let profile: Router<appState::AppState, _> = axum::Router::new()
        .route("/changepwd", patch(profile::changePwd))
        .route("/changecredit", patch(profile::changeCredit));

    //All routes nested under /auth (i.e /auth/login)
    let auth: Router<appState::AppState, _> = axum::Router::new()
        .route("/register", post(auth::register))
        .route("/login", post(auth::login))
        .route("/logout", post(auth::logout));

    //Al routes nested under /api (i.e, /auth/*)
    let api: Router = axum::Router::new()
        .nest("/auth", auth)
        .nest("/profile", profile)
        .nest("/loans", loans)
        .layer(middleware::from_fn_with_state(appState.clone(), auth::validationLayer))
        .with_state(appState);

    //All routes nested under / (i.e, /api/*)
    let app: Router = axum::Router::new()
        .nest("/api", api)
        .layer(CorsLayer::permissive())
        .fallback(endpoints::pageNotFound);

    let addr: SocketAddr = SocketAddr::from(([0,0,0,0], 4433));
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

    /*axum::Server::bind(&"0.0.0.0:4433".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();*/

    Ok(())
}