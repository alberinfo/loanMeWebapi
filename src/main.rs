use axum::{routing::*, extract::Path, http::StatusCode, response::IntoResponse, Json, extract::State};
use axum_server::tls_rustls::RustlsConfig;
use std::{net::SocketAddr};

mod db;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_max_level(tracing::Level::DEBUG).init(); //Initialize logging

    dotenvy::dotenv(); //load environment vars

    //let dbPool = db::getDbConnection().await;

    //All routes nested under /v0
    let v0: Router = axum::Router::new()
        .route("/Users", get(getAllUsuarios));
        //.route("/Alumnos/:id", get(getAlumnoById))
        //.route("/Alumnos", post(insertAlumno))
        //.with_state(dbPool);

    //Al routes nested under /api (i.e, /v0/*)
    let api: Router = axum::Router::new()
        .nest("/v0", v0);

    //All routes nested under / (i.e, /api/*)
    let app: Router = axum::Router::new()
        .nest("/api", api)
        .fallback(pageNotFound);

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
}

async fn pageNotFound() -> impl IntoResponse {
    return (StatusCode::NOT_FOUND, "Page not found!");
}

async fn getAllUsuarios(/*State(dbPool): State<sqlx::PgPool>*/) -> Result<String, (StatusCode, String)> {
    return Ok("LMAO".to_string());
}