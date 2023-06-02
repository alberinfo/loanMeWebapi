#![allow(non_snake_case)]

use axum::{routing::*, middleware};
use axum_server::tls_rustls::RustlsConfig;
use std::{error::Error, net::SocketAddr};
use loanMeWebapi::routes::*;
use loanMeWebapi::services::*;
use redis::RedisError;

//mod lib;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt().with_max_level(tracing::Level::DEBUG).init(); //Initialize logging

    let _ = dotenvy::dotenv(); //load environment vars

    let dbPoolResult = db::getDbConnection().await;

    if dbPoolResult.is_err() {
        eprintln!("Error while connecting to postgres");
        return Err(dbPoolResult.unwrap_err().to_string().into());
    }

    let dbPool: sqlx::Pool<sqlx::Postgres> = dbPoolResult.unwrap();
    let tableCount = db::getTableCount(&dbPool).await;
    if tableCount == 0 || tableCount == 1 { //assuming tablecount = 0 means db was just created, and = 1 means just _sqlx_migrations exists
        let res = sqlx::migrate!("./migrations").run(&dbPool).await;
        if res.is_err() {
            let err = res.unwrap_err();
            eprintln!("There was an error while migrating the database", );
            return Err(err.source().unwrap().to_string().into());
        }
    }

    let redisPoolResult: Result<redis::aio::ConnectionManager, RedisError> = redisServer::getRedisConnection().await;

    if redisPoolResult.is_err() {
        let err = redisPoolResult.err().unwrap();
        eprintln!("Error while connecting to redis ({:?})", err.kind());
        return Err(err.detail().unwrap_or("no further detail was provided").into());
    }

    let redisPool = redisPoolResult.unwrap();

    //All routes nested under /v0
    let v0: Router = axum::Router::new()
        .route("/registrarUsuario", post(endpoints::registrarUsuario))
        .route("/loginUsuario", post(endpoints::loginUsuario))
        .route("/logoutUsuario", post(endpoints::logoutUsuario))
        .layer(middleware::from_fn_with_state((dbPool.clone(), redisPool.clone()), auth::validationLayer))
        .with_state((dbPool, redisPool));

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