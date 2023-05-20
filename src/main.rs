#![allow(non_snake_case)]

use axum::{routing::*, middleware};
use axum_server::tls_rustls::RustlsConfig;
use std::{error::Error, net::SocketAddr};

mod endpoints;
mod db;
mod logic;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt().with_max_level(tracing::Level::DEBUG).init(); //Initialize logging

    let _ = dotenvy::dotenv(); //load environment vars

    let dbPool = db::getDbConnection().await;

    if db::getTableCount(&dbPool).await == 0 {
        sqlx::migrate!("./migrations").run(&dbPool).await?;
    }
    
    //All routes nested under /v0
    let v0: Router = axum::Router::new()
        .route("/registrarUsuario", get(endpoints::registrarUsuario))
        .route("/loginUsuario", get(endpoints::loginUsuario))
        //.route("/Alumnos/:id", get(getAlumnoById))
        //.route("/Alumnos", post(insertAlumno))
        .layer(middleware::from_fn_with_state(dbPool.clone(), endpoints::validateCredentialsLayer))
        .with_state(dbPool);

    //Al routes nested under /api (i.e, /v0/*)
    let api: Router = axum::Router::new()
        .nest("/v0", v0);

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